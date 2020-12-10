//! Text input for `TextReader` and the reader half of `TextReaderWriter`.

use crate::{
    rc_char_queue::{RcCharQueue, RcCharQueueIter},
    replace_selected::ReplaceSelected,
    unicode::{
        is_normalization_form_starter, BEL, BOM, CAN, CGJ, DEL, ESC, FF, MAX_UTF8_SIZE, NEL,
        NORMALIZATION_BUFFER_LEN, NORMALIZATION_BUFFER_SIZE, REPL,
    },
    ReadStr, TextReader, TextReaderWriter, Utf8Reader, Utf8ReaderWriter,
};
use io_ext::{
    default_read, default_read_exact, default_read_to_end, default_read_to_string,
    default_read_vectored, ReadExt, ReadWriteExt, Status,
};
use std::{io, mem, str};
use unicode_normalization::{Recompositions, Replacements, StreamSafe, UnicodeNormalization};

pub(crate) trait TextReaderInternals<Inner: ReadExt>: ReadExt {
    type Utf8Inner: ReadStr;
    fn impl_(&mut self) -> &mut TextInput;
    fn inner(&self) -> &Self::Utf8Inner;
    fn inner_mut(&mut self) -> &mut Self::Utf8Inner;
}

impl<Inner: ReadExt> TextReaderInternals<Inner> for TextReader<Inner> {
    type Utf8Inner = Utf8Reader<Inner>;

    fn impl_(&mut self) -> &mut TextInput {
        &mut self.impl_
    }

    fn inner(&self) -> &Self::Utf8Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Utf8Inner {
        &mut self.inner
    }
}

impl<Inner: ReadWriteExt> TextReaderInternals<Inner> for TextReaderWriter<Inner> {
    type Utf8Inner = Utf8ReaderWriter<Inner>;

    fn impl_(&mut self) -> &mut TextInput {
        &mut self.input
    }

    fn inner(&self) -> &Self::Utf8Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Utf8Inner {
        &mut self.inner
    }
}

pub(crate) struct TextInput {
    /// Temporary storage for reading scalar values from the underlying stream.
    raw_string: String,

    /// A queue of scalar values which have been translated but not written to
    /// the output yet.
    /// TODO: This is awkward; what we really want here is a streaming stream-safe
    /// and NFC translator.
    queue: RcCharQueue,

    /// An iterator over the chars in `self.queue`.
    queue_iter: Option<Recompositions<StreamSafe<ReplaceSelected<Replacements<RcCharQueueIter>>>>>,

    /// When we can't fit all the data from an underlying read in our buffer,
    /// we buffer it up. Remember the status value so we can replay that too.
    pending_status: Status,

    /// At the beginning of a stream or after a push, expect a
    /// normalization-form starter.
    expect_starter: bool,

    /// For emitting BOM at the start of a stream.
    at_start: bool,

    /// Control-code and escape-sequence state machine.
    state: State,
}

impl TextInput {
    /// Construct a new instance of `TextInput`.
    #[inline]
    pub fn new() -> Self {
        let queue = RcCharQueue::new();
        Self {
            raw_string: String::new(),
            queue,
            queue_iter: None,
            pending_status: Status::active(),
            expect_starter: true,
            at_start: true,
            state: State::Ground(true),
        }
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[inline]
    pub fn read_str<Inner: ReadExt>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<(usize, Status)> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        Self::read_with_status(internals, unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_exact` but produces the result in a `str`.
    #[inline]
    pub fn read_exact_str<Inner: ReadExt>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<()> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        Self::read_exact(internals, unsafe { buf.as_bytes_mut() })
    }

    fn queue_next(&mut self, sequence_end: bool) -> Option<char> {
        if !sequence_end && self.queue.len() < NORMALIZATION_BUFFER_LEN {
            return None;
        }
        if self.queue_iter.is_none() {
            if self.queue.is_empty() {
                return None;
            }
            self.queue_iter = Some(
                ReplaceSelected::new(self.queue.iter().svar())
                    .stream_safe()
                    .nfc(),
            );
        }
        if let Some(c) = self.queue_iter.as_mut().unwrap().next() {
            return Some(c);
        }
        self.queue_iter = None;
        None
    }

    fn process_raw_string(&mut self) {
        for c in self.raw_string.chars() {
            let at_start = mem::replace(&mut self.at_start, false);
            loop {
                match (self.state, c) {
                    (State::Ground(_), BOM) if at_start => (),
                    (State::Ground(_), '\n') => {
                        self.queue.push('\n');
                        self.expect_starter = false;
                        self.state = State::Ground(true)
                    }
                    (State::Ground(_), '\t') => {
                        self.queue.push('\t');
                        self.expect_starter = false;
                        self.state = State::Ground(false)
                    }
                    (State::Ground(_), FF) | (State::Ground(_), NEL) => {
                        self.queue.push(' ');
                        self.expect_starter = false;
                        self.state = State::Ground(false)
                    }
                    (State::Ground(_), '\r') => self.state = State::Cr,
                    (State::Ground(_), ESC) => self.state = State::Esc,
                    (State::Ground(_), c)
                        if c.is_control() || matches!(c, '\u{2329}' | '\u{232a}') =>
                    {
                        self.queue.push(REPL);
                        self.expect_starter = false;
                        self.state = State::Ground(false);
                    }
                    (State::Ground(_), CGJ) => {
                        self.queue.push(CGJ);
                        self.expect_starter = false;
                        self.state = State::Ground(false)
                    }
                    (State::Ground(_), mut c) => {
                        if self.expect_starter {
                            self.expect_starter = false;
                            if !is_normalization_form_starter(c) {
                                c = REPL;
                            }
                        }
                        assert!(c != CGJ);
                        self.queue.push(c);
                        self.state = State::Ground(false)
                    }

                    (State::Cr, '\n') => {
                        self.queue.push('\n');
                        self.expect_starter = false;
                        self.state = State::Ground(true);
                    }
                    (State::Cr, _) => {
                        self.queue.push('\n');
                        self.expect_starter = false;
                        self.state = State::Ground(true);
                        continue;
                    }

                    (State::Esc, '[') => self.state = State::CsiStart,
                    (State::Esc, ']') => self.state = State::Osc,
                    (State::Esc, ESC) => self.state = State::Esc,
                    (State::Esc, c) if matches!(c, '@'..='~' | CAN) => {
                        self.state = State::Ground(false)
                    }
                    (State::Esc, _) => {
                        self.queue.push(REPL);
                        self.expect_starter = false;
                        self.state = State::Ground(false);
                        continue;
                    }

                    (State::CsiStart, '[') => self.state = State::Linux,
                    (State::CsiStart, c) | (State::Csi, c) if matches!(c, ' '..='?') => {
                        self.state = State::Csi
                    }
                    (State::CsiStart, c) | (State::Csi, c) if matches!(c, '@'..='~') => {
                        self.state = State::Ground(false)
                    }
                    (State::CsiStart, CAN) | (State::Csi, CAN) => self.state = State::Ground(false),
                    (State::CsiStart, _) | (State::Csi, _) => {
                        self.state = State::Ground(false);
                        continue;
                    }

                    (State::Osc, BEL) | (State::Osc, CAN) => self.state = State::Ground(false),
                    (State::Osc, ESC) => self.state = State::Esc,
                    (State::Osc, _) => (),

                    (State::Linux, c) if matches!(c, '\0'..=DEL) => {
                        self.state = State::Ground(false)
                    }
                    (State::Linux, _) => {
                        self.state = State::Ground(false);
                        continue;
                    }
                }
                break;
            }
        }
    }

    pub(crate) fn read_with_status<Inner: ReadExt>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<(usize, Status)> {
        if buf.len() < NORMALIZATION_BUFFER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer for text input must be at least NORMALIZATION_BUFFER_SIZE bytes",
            ));
        }

        let mut nread = 0;

        loop {
            match internals.impl_().queue_next(false) {
                Some(c) => nread += c.encode_utf8(&mut buf[nread..]).len(),
                None => break,
            }
            if buf.len() - nread < MAX_UTF8_SIZE {
                return Ok((nread, Status::active()));
            }
        }
        if internals.impl_().pending_status != Status::active() {
            internals.impl_().pending_status = Status::active();
            internals.impl_().expect_starter = true;
            return Ok((nread, internals.impl_().pending_status));
        }

        let mut raw_bytes =
            mem::replace(&mut internals.impl_().raw_string, String::new()).into_bytes();
        raw_bytes.resize(4096, 0_u8);
        let (size, status) = internals.inner_mut().read_with_status(&mut raw_bytes)?;
        raw_bytes.resize(size, 0);
        // Safety: This is a UTF-8 stream so we can read directly into a `String`.
        internals.impl_().raw_string = unsafe { String::from_utf8_unchecked(raw_bytes) };

        internals.impl_().process_raw_string();

        if status != Status::active() {
            match internals.impl_().state {
                State::Ground(_) => {}
                State::Cr => {
                    internals.impl_().queue.push('\n');
                    internals.impl_().expect_starter = false;
                    internals.impl_().state = State::Ground(true);
                }
                State::Esc => {
                    internals.impl_().queue.push(REPL);
                    internals.impl_().expect_starter = false;
                    internals.impl_().state = State::Ground(false);
                }
                State::CsiStart | State::Csi | State::Osc | State::Linux => {
                    internals.impl_().state = State::Ground(false);
                }
            }

            if status.is_end() && internals.impl_().state != State::Ground(true) {
                internals.impl_().queue.push('\n');
                internals.impl_().expect_starter = false;
                internals.impl_().state = State::Ground(true);
            }
        }

        loop {
            match internals.impl_().queue_next(status != Status::active()) {
                Some(c) => nread += c.encode_utf8(&mut buf[nread..]).len(),
                None => break,
            }
            if buf.len() - nread < MAX_UTF8_SIZE {
                break;
            }
        }

        Ok((
            nread,
            if internals.impl_().queue_iter.is_none() {
                if status != Status::active() {
                    internals.impl_().expect_starter = true;
                }
                status
            } else {
                internals.impl_().pending_status = status;
                Status::active()
            },
        ))
    }

    pub(crate) fn minimum_buffer_size<Inner: ReadExt>(
        _internals: &impl TextReaderInternals<Inner>,
    ) -> usize {
        NORMALIZATION_BUFFER_SIZE
    }

    #[inline]
    pub(crate) fn read<Inner: ReadExt>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        default_read(internals, buf)
    }

    #[inline]
    pub(crate) fn read_vectored<Inner: ReadExt>(
        internals: &mut impl TextReaderInternals<Inner>,
        bufs: &mut [io::IoSliceMut<'_>],
    ) -> io::Result<usize> {
        default_read_vectored(internals, bufs)
    }

    #[cfg(feature = "nightly")]
    #[inline]
    pub(crate) fn is_read_vectored<Inner: ReadExt>(
        internals: &impl TextReaderInternals<Inner>,
    ) -> bool {
        default_is_read_vectored(internals)
    }

    #[inline]
    pub(crate) fn read_to_end<Inner: ReadExt>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut Vec<u8>,
    ) -> io::Result<usize> {
        default_read_to_end(internals, buf)
    }

    #[inline]
    pub(crate) fn read_to_string<Inner: ReadExt>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut String,
    ) -> io::Result<usize> {
        default_read_to_string(internals, buf)
    }

    #[inline]
    pub(crate) fn read_exact<Inner: ReadExt>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<()> {
        default_read_exact(internals, buf)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    // Default state. Boolean is true iff we just saw a '\n'.
    Ground(bool),

    // After a '\r'.
    Cr,

    // After a '\x1b'.
    Esc,

    // Immediately after a "\x1b[".
    CsiStart,

    // Within a sequence started by "\x1b[".
    Csi,

    // Within a sequence started by "\x1b]".
    Osc,

    // After a "\x1b[[".
    Linux,
}
