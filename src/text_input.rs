//! Shared implementation for `TextReader` and the reader half of
//! `TextDuplexer`.

use crate::{TextDuplexer, TextReader, TextStr};
use basic_text_internals::{
    is_basic_text_end, is_basic_text_start,
    unicode::{
        BEL, BOM, CAN, CGJ, DEL, ESC, FF, MAX_UTF8_SIZE, NEL, NORMALIZATION_BUFFER_SIZE, REPL,
    },
    ReplaceSelected,
};
use layered_io::{default_read, HalfDuplexLayered, Status, WriteLayered};
use std::{
    cmp::max,
    collections::{vec_deque, VecDeque},
    io::{self, copy, repeat, Cursor, Read},
    mem::take,
    str,
};
use unicode_normalization::{Recompositions, Replacements, StreamSafe, UnicodeNormalization};
use utf8_io::{ReadStrLayered, WriteStr};

/// Abstract over `TextReader` and the reader half of `TextDuplexer`.
pub(crate) trait TextReaderInternals<Inner: ReadStrLayered>: ReadStrLayered {
    fn impl_(&mut self) -> &mut TextInput;
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
    fn into_inner(self) -> Inner;
}

impl<Inner: ReadStrLayered> TextReaderInternals<Inner> for TextReader<Inner> {
    fn impl_(&mut self) -> &mut TextInput {
        &mut self.input
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr + WriteLayered> TextReaderInternals<Inner>
    for TextDuplexer<Inner>
{
    fn impl_(&mut self) -> &mut TextInput {
        &mut self.input
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }
}

pub(crate) struct TextInput {
    /// Temporary storage for reading scalar values from the underlying stream.
    raw_string: String,

    /// A queue of scalar values which have been translated but not written to
    /// the output yet.
    /// TODO: This is awkward; what we really want here is a streaming stream-safe
    /// and NFC translator.
    queue: VecDeque<char>,

    /// An iterator over the chars in `self.queue`.
    queue_iter:
        Recompositions<StreamSafe<ReplaceSelected<Replacements<vec_deque::IntoIter<char>>>>>,

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
    pub(crate) fn new() -> Self {
        let queue = VecDeque::new();
        Self {
            raw_string: String::new(),
            queue,
            queue_iter: ReplaceSelected::new(
                VecDeque::<char>::new().into_iter().cjk_compat_variants(),
            )
            .stream_safe()
            .nfc(),
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
    pub(crate) fn read_str<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<usize> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_exact` but produces the result in a `str`.
    #[inline]
    pub(crate) fn read_exact_str<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<()> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read_exact(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[inline]
    pub(crate) fn read_str_with_status<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<(usize, Status)> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        let (size, status) = internals.read_with_status(unsafe { buf.as_bytes_mut() })?;

        debug_assert!(buf.is_char_boundary(size));

        Ok((size, status))
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[inline]
    pub(crate) fn read_exact_str_using_status<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<Status> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read_exact_using_status(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_with_status` but produces the result in a `TextStr`. Be sure
    /// to check the `size` field of the return value to see how many bytes
    /// were written.
    #[inline]
    pub(crate) fn read_text<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut TextStr,
    ) -> io::Result<usize> {
        // Safety: This is a UTF-8 stream so we can read directly into a
        // `TextStr`.
        internals.read(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_exact` but produces the result in a `TextStr`.
    #[inline]
    pub(crate) fn read_exact_text<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut TextStr,
    ) -> io::Result<()> {
        // Safety: This is a Text stream so we can read directly into a
        // `TextStr`.
        internals.read_exact(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_with_status` but produces the result in a `TextStr`. Be sure
    /// to check the `size` field of the return value to see how many bytes
    /// were written.
    #[inline]
    pub(crate) fn read_text_with_status<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut TextStr,
    ) -> io::Result<(usize, Status)> {
        // Safety: This is a text stream so we can read directly into a `TextStr`.
        let (size, status) = internals.read_with_status(unsafe { buf.as_bytes_mut() })?;

        // TODO
        //debug_assert!(buf.is_char_boundary(size));

        Ok((size, status))
    }

    /// Like `read_with_status` but produces the result in a `TextStr`. Be sure
    /// to check the `size` field of the return value to see how many bytes
    /// were written.
    #[inline]
    pub(crate) fn read_exact_text_using_status<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut TextStr,
    ) -> io::Result<Status> {
        // Safety: This is a text stream so we can read directly into a `str`.
        internals.read_exact_using_status(unsafe { buf.as_bytes_mut() })
    }

    fn queue_next(&mut self) -> Option<char> {
        match self.queue_iter.next() {
            Some(c) => Some(c),
            None => {
                let index = self.queue.iter().position(|c| matches!(*c, '\n' | CGJ))?;
                let tmp = self.queue.drain(0..=index).collect::<VecDeque<char>>();
                self.queue_iter = ReplaceSelected::new(tmp.into_iter().cjk_compat_variants())
                    .stream_safe()
                    .nfc();
                self.queue_iter.next()
            }
        }
    }

    fn process_raw_string(&mut self) {
        for c in self.raw_string.chars() {
            let at_start = take(&mut self.at_start);
            loop {
                match (self.state, c) {
                    (State::Ground(_), BOM) if at_start => (),
                    (State::Ground(_), '\n') => {
                        self.queue.push_back('\n');
                        self.expect_starter = false;
                        self.state = State::Ground(true);
                    }
                    (State::Ground(_), '\t') => {
                        self.queue.push_back('\t');
                        self.expect_starter = false;
                        self.state = State::Ground(false);
                    }
                    (State::Ground(_), FF) | (State::Ground(_), NEL) => {
                        self.queue.push_back(' ');
                        self.expect_starter = false;
                        self.state = State::Ground(false);
                    }
                    (State::Ground(_), '\r') => self.state = State::Cr,
                    (State::Ground(_), ESC) => self.state = State::Esc,
                    (State::Ground(_), c)
                        if c.is_control() || matches!(c, '\u{2329}' | '\u{232a}') =>
                    {
                        self.queue.push_back(REPL);
                        self.expect_starter = false;
                        self.state = State::Ground(false);
                    }
                    (State::Ground(_), mut c) => {
                        if self.expect_starter {
                            self.expect_starter = false;
                            if !is_basic_text_start(c) {
                                c = REPL;
                            }
                        }
                        self.queue.push_back(c);
                        self.state = State::Ground(false);
                    }

                    (State::Cr, '\n') => {
                        self.queue.push_back('\n');
                        self.expect_starter = false;
                        self.state = State::Ground(true);
                    }
                    (State::Cr, _) => {
                        self.queue.push_back('\n');
                        self.expect_starter = false;
                        self.state = State::Ground(true);
                        continue;
                    }

                    (State::Esc, '[') => self.state = State::CsiStart,
                    (State::Esc, ']') => self.state = State::Osc,
                    (State::Esc, ESC) => self.state = State::Esc,
                    (State::Esc, c) if matches!(c, '@'..='~' | CAN) => {
                        self.state = State::Ground(false);
                    }
                    (State::Esc, _) => {
                        self.queue.push_back(REPL);
                        self.expect_starter = false;
                        self.state = State::Ground(false);
                        continue;
                    }

                    (State::CsiStart, '[') => self.state = State::Linux,
                    (State::CsiStart, c) | (State::Csi, c) if matches!(c, ' '..='?') => {
                        self.state = State::Csi;
                    }
                    (State::CsiStart, c) | (State::Csi, c) if matches!(c, '@'..='~') => {
                        self.state = State::Ground(false);
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
                        self.state = State::Ground(false);
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

    pub(crate) fn read_with_status<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<(usize, Status)> {
        if buf.len() < NORMALIZATION_BUFFER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("buffer for text input is {} bytes; at least NORMALIZATION_BUFFER_SIZE bytes are required", buf.len())
            ));
        }

        let mut nread = 0;

        loop {
            match internals.impl_().queue_next() {
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

            // We may have overwritten part of a codepoint; overwrite the rest
            // of the buffer.
            // TODO: Use [`fill`] when it becomes available:
            // https://doc.rust-lang.org/std/primitive.slice.html#method.fill
            copy(
                &mut repeat(b'?').take((buf.len() - nread) as u64),
                &mut Cursor::new(&mut buf[nread..]),
            )
            .unwrap();

            return Ok((nread, internals.impl_().pending_status));
        }

        let mut raw_bytes = take(&mut internals.impl_().raw_string).into_bytes();
        raw_bytes.resize(4096, 0_u8);
        let (size, status) = internals.inner_mut().read_with_status(&mut raw_bytes)?;
        raw_bytes.resize(size, 0);
        // Safety: This is a UTF-8 stream so we can read into a `String`.
        internals.impl_().raw_string = unsafe { String::from_utf8_unchecked(raw_bytes) };

        internals.impl_().process_raw_string();

        if status != Status::active() {
            match internals.impl_().state {
                State::Ground(_) => {}
                State::Cr => {
                    internals.impl_().queue.push_back('\n');
                    internals.impl_().expect_starter = false;
                    internals.impl_().state = State::Ground(true);
                }
                State::Esc => {
                    internals.impl_().queue.push_back(REPL);
                    internals.impl_().expect_starter = false;
                    internals.impl_().state = State::Ground(false);
                }
                State::CsiStart | State::Csi | State::Osc | State::Linux => {
                    internals.impl_().state = State::Ground(false);
                }
            }

            // If the stream ends in a non-ending char, append a REPL.
            if let Some(last) = internals.impl_().queue.back() {
                if !is_basic_text_end(*last) {
                    internals.impl_().queue.push_back(REPL);
                }
            }

            if status.is_end() {
                // If the stream doesn't end in a newline, append one.
                if internals.impl_().state != State::Ground(true) {
                    internals.impl_().queue.push_back('\n');
                    internals.impl_().expect_starter = false;
                    internals.impl_().state = State::Ground(true);
                }
            }
        }

        let mut queue_empty = false;
        loop {
            match internals.impl_().queue_next() {
                Some(c) => nread += c.encode_utf8(&mut buf[nread..]).len(),
                None => {
                    queue_empty = true;
                    break;
                }
            }
            if buf.len() - nread < MAX_UTF8_SIZE {
                break;
            }
        }

        // We may have overwritten part of a codepoint; overwrite the rest
        // of the buffer.
        copy(
            &mut repeat(b'?').take((buf.len() - nread) as u64),
            &mut Cursor::new(&mut buf[nread..]),
        )
        .unwrap();

        Ok((
            nread,
            if queue_empty {
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

    #[inline]
    pub(crate) fn minimum_buffer_size<Inner: ReadStrLayered>(
        internals: &impl TextReaderInternals<Inner>,
    ) -> usize {
        max(
            NORMALIZATION_BUFFER_SIZE,
            internals.inner().minimum_buffer_size(),
        )
    }

    #[inline]
    pub(crate) fn abandon<Inner: ReadStrLayered>(internals: &mut impl TextReaderInternals<Inner>) {
        // Don't enforce a trailing newline.
        internals.impl_().state = State::Ground(true);

        assert!(internals.impl_().queue.is_empty());

        internals.inner_mut().abandon();
    }

    #[inline]
    pub(crate) fn suggested_buffer_size<Inner: ReadStrLayered>(
        internals: &impl TextReaderInternals<Inner>,
    ) -> usize {
        max(
            Self::minimum_buffer_size(internals),
            internals.inner().suggested_buffer_size(),
        )
    }

    #[inline]
    pub(crate) fn read<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        default_read(internals, buf)
    }

    #[inline]
    pub(crate) fn read_to_string<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut String,
    ) -> io::Result<usize> {
        // Safety: This is a UTF-8 stream so we can read into a `String`.
        internals.read_to_end(unsafe { buf.as_mut_vec() })
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
