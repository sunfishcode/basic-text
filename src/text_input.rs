//! Shared implementation for `TextReader` and the reader half of
//! `TextDuplexer`.

use crate::{TextDuplexer, TextReader, TextSubstr};
use basic_text_internals::unicode::{
    BEL, BOM, CAN, CGJ, DEL, ESC, LS, MAX_UTF8_SIZE, NEL, NORMALIZATION_BUFFER_SIZE, PS,
};
use basic_text_internals::unicode_normalization::char::is_public_assigned;
use basic_text_internals::unicode_normalization::{
    is_nfc_stream_safe_quick, IsNormalized, Recompositions, Replacements, StreamSafe,
    UnicodeNormalization,
};
use basic_text_internals::{
    is_basic_text_end, is_basic_text_start, replace, IsolateUnassigned, PreNormalization,
};
use layered_io::{default_read, HalfDuplexLayered, Status, WriteLayered};
use std::cmp::max;
use std::collections::{vec_deque, VecDeque};
use std::mem::take;
use std::{io, str};
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
    /// TODO: This is awkward; what we really want here is a streaming
    /// stream-safe and NFC translator.
    queue: VecDeque<char>,

    /// An iterator over the chars in `self.queue`.
    ssnfc_iter:
        Recompositions<StreamSafe<Replacements<IsolateUnassigned<vec_deque::IntoIter<char>>>>>,

    /// The number of characters in the queue which are already verified to be
    /// Stream-Safe NFC and can skip normalization.
    quick: usize,

    /// When we can't fit all the data from an underlying read in our buffer,
    /// we buffer it up. Remember the status value so we can replay that too.
    pending_status: Status,

    /// At the beginning of a stream or after a push, expect a
    /// normalization-form starter.
    expect_starter: bool,

    /// For ignoring BOM at the start of a stream.
    at_start: bool,

    /// NEL compatibility mode.
    nel_compatibility: bool,

    /// LSPS compatibility mode.
    lsps_compatibility: bool,

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
            ssnfc_iter: VecDeque::<char>::new()
                .into_iter()
                .isolate_unassigned()
                .cjk_compat_variants()
                .stream_safe()
                .nfc(),
            quick: 0,
            pending_status: Status::active(),
            expect_starter: true,
            at_start: true,
            nel_compatibility: false,
            lsps_compatibility: false,
            state: State::Ground(true),
        }
    }

    /// Construct a new instance of `TextInput` in NEL compatibility mode.
    #[inline]
    pub(crate) fn with_nel_compatibility() -> Self {
        let mut result = Self::new();
        result.nel_compatibility = true;
        result
    }

    /// Construct a new instance of `TextInput` in LSPS compatibility mode.
    #[inline]
    pub(crate) fn with_lsps_compatibility() -> Self {
        let mut result = Self::new();
        result.lsps_compatibility = true;
        result
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

    #[inline]
    pub(crate) fn read_text_substr<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut TextSubstr,
    ) -> io::Result<usize> {
        // Safety: This is a Basic Text stream so we can read directly into a
        // `TextSubstr`.
        internals.read(unsafe { buf.as_bytes_mut() })
    }

    #[inline]
    pub(crate) fn read_exact_text_substr<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut TextSubstr,
    ) -> io::Result<()> {
        // Safety: This is a Basic Text stream so we can read directly into a
        // `TextSubstr`.
        internals.read_exact(unsafe { buf.as_bytes_mut() })
    }

    #[inline]
    pub(crate) fn read_text_substr_with_status<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut TextSubstr,
    ) -> io::Result<(usize, Status)> {
        // Safety: This is a Basic Text stream so we can read directly into a
        // `TextSubstr`.
        internals.read_with_status(unsafe { buf.as_bytes_mut() })
    }

    #[inline]
    pub(crate) fn read_exact_text_substr_using_status<Inner: ReadStrLayered>(
        internals: &mut impl TextReaderInternals<Inner>,
        buf: &mut TextSubstr,
    ) -> io::Result<Status> {
        // Safety: This is a Basic Text stream so we can read directly into a
        // `TextSubstr`.
        internals.read_exact_using_status(unsafe { buf.as_bytes_mut() })
    }

    fn queue_next(&mut self) -> Option<char> {
        let quick = self.quick;
        if quick != 0 {
            self.quick = quick - 1;
            self.queue.pop_front()
        } else {
            match self.ssnfc_iter.next() {
                Some(c) => Some(c),
                None => {
                    let last_boundary = self
                        .queue
                        .iter()
                        .rev()
                        .position(|c| matches!(*c, '\n' | CGJ))?;
                    let index = self.queue.len() - last_boundary;
                    if is_nfc_stream_safe_quick(self.queue.iter().take(index).copied())
                        == IsNormalized::Yes
                        && self
                            .queue
                            .iter()
                            .take(index)
                            .copied()
                            .all(is_public_assigned)
                    {
                        self.quick = index - 1;
                        self.queue.pop_front()
                    } else {
                        let tmp = self.queue.drain(..index).collect::<VecDeque<char>>();
                        self.ssnfc_iter = tmp
                            .into_iter()
                            .isolate_unassigned()
                            .cjk_compat_variants()
                            .stream_safe()
                            .nfc();
                        self.ssnfc_iter.next()
                    }
                }
            }
        }
    }

    fn process_raw_string(&mut self) {
        let mut chars = self.raw_string.chars();

        // If we're at the start of a stream, skip over a leading BOM.
        if take(&mut self.at_start) && self.raw_string.starts_with(BOM) {
            chars.next();
        }

        for c in chars {
            loop {
                match (self.state, c) {
                    (State::Ground(_), c) => match c {
                        '\n' => {
                            self.queue.push_back('\n');
                            self.expect_starter = false;
                            self.state = State::Ground(true);
                        }
                        '\r' => self.state = State::Cr,
                        '\x0c' => self.state = State::Ff,
                        ESC => self.state = State::Esc,
                        mut c => {
                            self.state = State::Ground(false);
                            if (self.nel_compatibility && c == NEL)
                                || (self.lsps_compatibility && matches!(c, LS | PS))
                            {
                                c = '\n';
                                self.state = State::Ground(true);
                            }
                            let pos = self.queue.len();
                            replace(c, &mut self.queue);

                            // Prepend a CGJ if needed to guard a non-starter.
                            if take(&mut self.expect_starter)
                                && !self
                                    .queue
                                    .get(pos)
                                    .copied()
                                    .map(is_basic_text_start)
                                    .unwrap_or(true)
                            {
                                self.queue.insert(pos, CGJ);
                            }
                        }
                    },

                    (State::Cr, c) => {
                        self.queue.push_back('\n');
                        self.expect_starter = false;
                        self.state = State::Ground(true);
                        if c != '\n' {
                            continue;
                        }
                    }

                    (State::Ff, c) => {
                        if c != '\x0c' {
                            if c != '\n' && c != '\r' {
                                self.queue.push_back(' ');
                            }
                            self.expect_starter = false;
                            self.state = State::Ground(false);
                            continue;
                        }
                    }

                    (State::Esc, '[') => self.state = State::CsiStart,
                    (State::Esc, ']') => self.state = State::Osc,
                    (State::Esc, ESC) => self.state = State::Esc,
                    (State::Esc, c) if matches!(c, '@'..='~' | CAN) => {
                        self.state = State::Ground(false);
                    }
                    (State::Esc, _) => {
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
        while let Some(c) = internals.impl_().queue_next() {
            nread += c.encode_utf8(&mut buf[nread..]).len();
            if buf.len() - nread < MAX_UTF8_SIZE {
                // Write out single-byte codepoints to preserve UTF-8 validity.
                clear_to_char_boundary(&mut buf[nread..]);
                return Ok((nread, Status::active()));
            }
        }
        if internals.impl_().pending_status != Status::active() {
            internals.impl_().pending_status = Status::active();
            internals.impl_().expect_starter = true;

            // Write out single-byte codepoints to preserve UTF-8 validity.
            clear_to_char_boundary(&mut buf[nread..]);

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
                State::Ff => {
                    internals.impl_().queue.push_back(' ');
                    internals.impl_().expect_starter = false;
                    internals.impl_().state = State::Ground(false);
                }
                State::Esc | State::CsiStart | State::Csi | State::Osc | State::Linux => {
                    internals.impl_().state = State::Ground(false);
                }
            }

            // If the stream ends in a non-ending char, append a CGJ.
            if let Some(last) = internals.impl_().queue.back() {
                if !is_basic_text_end(*last) {
                    internals.impl_().queue.push_back(CGJ);
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

        // Write out single-byte codepoints to preserve UTF-8 validity.
        clear_to_char_boundary(&mut buf[nread..]);

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

#[inline]
fn clear_to_char_boundary(buf: &mut [u8]) {
    for b in buf {
        if is_char_boundary(*b) {
            break;
        }
        *b = b'\0';
    }
}

#[test]
fn test_clear_to_char_boundary() {
    let mut buf = vec![7, 7, 0x80, 0x80, 7];
    clear_to_char_boundary(&mut buf[2..]);
    assert_eq!(buf, vec![7, 7, 0, 0, 7]);
}

#[inline]
const fn is_char_boundary(b: u8) -> bool {
    b as i8 >= -0x40
}

#[test]
fn test_is_char_boundary() {
    assert!(is_char_boundary(b'\0'));
    assert!(is_char_boundary(b'A'));
    assert!(is_char_boundary(0x7f));
    assert!(!is_char_boundary(0x80));
    assert!(!is_char_boundary(0xbf));
    assert!(is_char_boundary(0xc0));
    assert!(is_char_boundary(0xcf));
    assert!(is_char_boundary(0xff));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    // Default state. Boolean is true iff we just saw a '\n'.
    Ground(bool),

    // After a '\r'.
    Cr,

    // After a '\x0c'.
    Ff,

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
