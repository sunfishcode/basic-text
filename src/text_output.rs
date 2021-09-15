//! Shared implementation for `TextWriter` and the writer half of
//! `TextDuplexer`.

use crate::{TextDuplexer, TextSubstr, TextWriter};
use basic_text_internals::unicode::{BOM, ESC, MAX_UTF8_SIZE, SUB};
use basic_text_internals::unicode_normalization::char::is_public_assigned;
use basic_text_internals::unicode_normalization::{
    is_nfc_stream_safe_quick, IsNormalized, UnicodeNormalization,
};
use basic_text_internals::{
    is_basic_text_end, is_basic_text_start, BasicTextError, PreNormalization,
};
#[cfg(can_vector)]
use layered_io::default_is_write_vectored;
#[cfg(write_all_vectored)]
use layered_io::default_write_all_vectored;
use layered_io::{default_write_vectored, HalfDuplexLayered, WriteLayered};
use std::cell::RefCell;
use std::io::{self, Write};
use std::mem::take;
use std::rc::Rc;
use std::str;
use utf8_io::{ReadStrLayered, WriteStr};

/// Abstract over `TextWriter` and the writer half of `TextDuplexer`.
pub(crate) trait TextWriterInternals<Inner: WriteStr + WriteLayered>: Write {
    fn impl_(&mut self) -> &mut TextOutput;
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
    fn into_inner(self) -> Inner;
    fn write_str(&mut self, s: &str) -> io::Result<()>;
}

impl<Inner: WriteStr + WriteLayered> TextWriterInternals<Inner> for TextWriter<Inner> {
    fn impl_(&mut self) -> &mut TextOutput {
        &mut self.output
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

    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.inner.write_str(s)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr + WriteLayered> TextWriterInternals<Inner>
    for TextDuplexer<Inner>
{
    fn impl_(&mut self) -> &mut TextOutput {
        &mut self.output
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

    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.inner.write_str(s)
    }
}

pub(crate) struct TextOutput {
    /// Temporary staging buffer.
    buffer: String,

    /// When enabled, "\n" is replaced by "\r\n".
    crlf_compatibility: bool,

    /// At the beginning of a stream or after a push, expect a
    /// normalization-form starter.
    expect_starter: bool,

    /// Are `ESC [ ... m`-style color sequences enabled?
    ansi_color: bool,

    /// Control-code and escape-sequence state machine.
    state: State,

    /// An in-progress escape sequence.
    escape_sequence: String,
}

impl TextOutput {
    /// Construct a new instance of `TextOutput`.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            buffer: String::new(),
            crlf_compatibility: false,
            expect_starter: true,
            ansi_color: false,
            state: State::Ground(Ground::Newline),
            escape_sequence: String::new(),
        }
    }

    /// Like `new`, but enables CRLF output mode, which translates "\n" to
    /// "\r\n" for compatibility with consumers that need that.
    ///
    /// Note: This is not often needed; even on Windows these days most
    /// things are ok with plain '\n' line endings, [including Windows
    /// Notepad]. The main notable things that really need them are IETF
    /// RFCs, for example [RFC-5198].
    ///
    /// [including Windows Notepad]: https://devblogs.microsoft.com/commandline/extended-eol-in-notepad/
    /// [RFC-5198]: https://tools.ietf.org/html/rfc5198#appendix-C
    #[inline]
    pub(crate) const fn with_crlf_compatibility() -> Self {
        let mut result = Self::new();
        result.crlf_compatibility = true;
        result
    }

    #[inline]
    pub(crate) fn with_bom_compatibility<Inner: WriteStr + WriteLayered>(
        inner: &mut Inner,
    ) -> io::Result<Self> {
        let result = Self::new();

        let mut bom_bytes = [0_u8; MAX_UTF8_SIZE];
        let bom_len = BOM.encode_utf8(&mut bom_bytes).len();
        // Safety: `bom_bytes` is valid UTF-8 because we just encoded it.
        inner.write_all(&bom_bytes[..bom_len])?;

        // The BOM is not part of the logical content, so leave the stream in
        // Ground(Ground::Newline) mode, meaning we don't require a newline if
        // nothing else is written to the stream.

        Ok(result)
    }

    /// Construct a new instance of `TextOutput` that optionally permits
    /// "ANSI"-style color escape sequences of the form `ESC [ ... m`.
    #[cfg(feature = "terminal-io")]
    #[inline]
    pub(crate) fn with_ansi_color(ansi_color: bool) -> Self {
        let mut result = Self::new();
        result.ansi_color = ansi_color;
        result
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    pub(crate) fn close_into_inner<Inner: WriteStr + WriteLayered>(
        mut internals: impl TextWriterInternals<Inner>,
    ) -> io::Result<Inner> {
        Self::check_nl(&mut internals)?;
        Ok(internals.into_inner())
    }

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    pub(crate) fn abandon_into_inner<Inner: WriteStr + WriteLayered>(
        mut internals: impl TextWriterInternals<Inner>,
    ) -> Inner {
        Self::reset_state(&mut internals);

        internals.into_inner()
    }

    fn normal_write_str<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        Self::state_machine(internals, s)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn crlf_write_str<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        // Translate "\n" into "\r\n".
        let mut first = true;
        for slice in s.split('\n') {
            if first {
                first = false;
            } else {
                let impl_ = internals.impl_();
                impl_.state = State::Ground(Ground::Newline);
                impl_.buffer.push_str("\r\n");
            }

            Self::state_machine(internals, slice)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        }

        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn normal_write_text<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &TextSubstr,
    ) -> io::Result<()> {
        let impl_ = internals.impl_();
        let s = s.as_ref(); // TODO: Avoid doing this.

        impl_.buffer.push_str(s);
        let ground = match s.chars().next_back() {
            None | Some('\n') => Ground::Newline,
            Some(c) if !is_basic_text_end(c) => Ground::ZwjOrPrepend,
            _ => Ground::Other,
        };
        impl_.state = State::Ground(ground);

        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn crlf_write_text<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &TextSubstr,
    ) -> io::Result<()> {
        let s: &str = s.as_ref(); // TODO: Avoid doing this.

        // Translate "\n" into "\r\n".
        let mut first = true;
        for slice in s.split('\n') {
            let impl_ = internals.impl_();
            if first {
                first = false;
            } else {
                impl_.state = State::Ground(Ground::Newline);
                impl_.buffer.push_str("\r\n");
            }

            impl_.buffer.push_str(slice);
            let ground = match slice.chars().next_back() {
                None | Some('\n') => Ground::Newline,
                Some(c) if !is_basic_text_end(c) => Ground::ZwjOrPrepend,
                _ => Ground::Other,
            };
            impl_.state = State::Ground(ground);
        }

        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn write_buffer<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) -> io::Result<()> {
        if internals.impl_().expect_starter {
            internals.impl_().expect_starter = false;
            if let Some(c) = internals.impl_().buffer.chars().next() {
                if !is_basic_text_start(c) {
                    Self::prepare_failure(internals);
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "write data must begin with a Unicode Normalization Form starter",
                    ));
                }
            }
        }

        let buffer = take(&mut internals.impl_().buffer);
        match internals.write_str(&buffer) {
            Ok(()) => (),
            Err(err) => {
                Self::prepare_failure(internals);
                return Err(err);
            }
        }
        internals.impl_().buffer = buffer;

        // Reset the temporary buffer.
        internals.impl_().buffer.clear();

        Ok(())
    }

    fn state_machine<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
    ) -> Result<(), BasicTextError> {
        let error = Rc::new(RefCell::new(None));

        if is_nfc_stream_safe_quick(s.chars()) == IsNormalized::Yes
            && s.chars().all(is_public_assigned)
        {
            // Fast path: Data is already Stream-Safe NFC and assigned. Just
            // check for errors.
            for c in s.chars().categorize(Rc::clone(&error)) {
                Self::state_machine_char(internals, c, &error)?;
            }
        } else {
            Self::state_machine_slow_path(internals, s, &error)?;
        }

        Ok(())
    }

    #[cold]
    fn state_machine_slow_path<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
        error: &Rc<RefCell<Option<BasicTextError>>>,
    ) -> Result<(), BasicTextError> {
        // Slow path: Compute Stream-Safe NFC, isolate unassigned scalar
        // values, and check for errors.
        for c in s
            .chars()
            .categorize(Rc::clone(error))
            .isolate_unassigned()
            .cjk_compat_variants()
            .stream_safe()
            .nfc()
        {
            Self::state_machine_char(internals, c, error)?;
        }

        Ok(())
    }

    fn state_machine_char<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        c: char,
        error: &Rc<RefCell<Option<BasicTextError>>>,
    ) -> Result<(), BasicTextError> {
        let impl_ = internals.impl_();
        match (&impl_.state, c) {
            // Recognize ANSI-style color escape sequences.
            (State::Ground(_), ESC) if impl_.ansi_color => {
                impl_.state = State::Esc;
                impl_.escape_sequence.clear();
                impl_.escape_sequence.push(ESC);
            }
            (State::Esc, '[') => {
                impl_.state = State::Csi;
                impl_.escape_sequence.push('[');
            }
            (State::Csi, c) if matches!(c, ' '..='?') => impl_.escape_sequence.push(c),
            (State::Csi, 'm') => {
                impl_.escape_sequence.push('m');
                impl_.buffer.push_str(&impl_.escape_sequence);
                impl_.state = State::Ground(Ground::Other);
            }

            (State::Ground(_), '\n') => {
                impl_.state = State::Ground(Ground::Newline);
                impl_.buffer.push(c);
            }

            (State::Ground(_), SUB) => {
                // SUB indicates an error sent through the NFC iterator
                // chain, and the Rc<RefCell<Option<BasicTextError>>> holds the
                // actual error.
                Self::prepare_failure(internals);
                return Err(take(&mut *error.borrow_mut()).unwrap());
            }

            (State::Ground(_), ESC) => {
                Self::prepare_failure(internals);
                return Err(BasicTextError::Escape);
            }

            // Common case: in ground state and reading a normal char.
            (State::Ground(_), c) => {
                let ground = if !is_basic_text_end(c) {
                    Ground::ZwjOrPrepend
                } else {
                    Ground::Other
                };
                impl_.state = State::Ground(ground);
                impl_.buffer.push(c);
            }

            // Escape sequence not recognized.
            (State::Esc, _) | (State::Csi, _) => {
                Self::prepare_failure(internals);
                return Err(BasicTextError::UnrecognizedEscape);
            }
        }
        Ok(())
    }

    fn check_nl<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) -> io::Result<()> {
        match internals.impl_().state {
            State::Ground(Ground::Newline) => Ok(()),
            State::Ground(Ground::ZwjOrPrepend) => {
                Self::prepare_failure(internals);
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "strict text stream ended after a ZWJ or Prepend",
                ))
            }
            State::Ground(Ground::Other) => {
                Self::prepare_failure(internals);
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "strict text stream must end with newline",
                ))
            }
            State::Esc | State::Csi => {
                Self::prepare_failure(internals);
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "incomplete escape sequence at end of strict text stream",
                ))
            }
        }
    }

    pub(crate) fn close<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.impl_().expect_starter = true;
        Self::check_nl(internals)?;
        internals.inner_mut().close()
    }

    pub(crate) fn abandon<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) {
        internals.inner_mut().abandon();

        Self::reset_state(internals);
    }

    pub(crate) fn suggested_buffer_size<Inner: WriteStr + WriteLayered>(
        internals: &impl TextWriterInternals<Inner>,
    ) -> usize {
        internals.inner().suggested_buffer_size()
    }

    pub(crate) fn write_text_substr<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &TextSubstr,
    ) -> io::Result<()> {
        if internals.impl_().crlf_compatibility {
            Self::crlf_write_text(internals, s)
        } else {
            Self::normal_write_text(internals, s)
        }
    }

    pub(crate) fn write_str<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        if internals.impl_().crlf_compatibility {
            Self::crlf_write_str(internals, s)
        } else {
            Self::normal_write_str(internals, s)
        }
    }

    pub(crate) fn write<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        buf: &[u8],
    ) -> io::Result<usize> {
        match str::from_utf8(buf) {
            Ok(s) => Self::write_str(internals, s).map(|()| buf.len()),
            // Safety: See the example code here:
            // https://doc.rust-lang.org/std/str/struct.Utf8Error.html#examples
            Err(error) if error.valid_up_to() != 0 => Self::write_str(internals, unsafe {
                str::from_utf8_unchecked(&buf[..error.valid_up_to()])
            })
            .map(|()| error.valid_up_to()),
            Err(error) => {
                Self::prepare_failure(internals);
                Err(io::Error::new(io::ErrorKind::InvalidData, error))
            }
        }
    }

    #[inline]
    pub(crate) fn flush<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) -> io::Result<()> {
        match internals.impl_().state {
            State::Ground(Ground::ZwjOrPrepend) => {
                Self::prepare_failure(internals);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "strict text stream flushed after a ZWJ or Prepend",
                ));
            }
            State::Ground(_) => (),
            State::Esc | State::Csi => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "strict text stream flushed while an escape sequence was in progress",
                ))
            }
        }
        internals.impl_().expect_starter = true;
        internals.inner_mut().flush()
    }

    #[inline]
    pub(crate) fn write_vectored<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        bufs: &[io::IoSlice<'_>],
    ) -> io::Result<usize> {
        default_write_vectored(internals, bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    pub(crate) fn is_write_vectored<Inner: WriteStr + WriteLayered>(
        internals: &impl TextWriterInternals<Inner>,
    ) -> bool {
        default_is_write_vectored(internals)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    pub(crate) fn write_all_vectored<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        bufs: &mut [io::IoSlice<'_>],
    ) -> io::Result<()> {
        default_write_all_vectored(internals, bufs)
    }

    #[inline]
    pub(crate) fn newline<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
        nl: bool,
    ) {
        if nl {
            internals.impl_().state = State::Ground(Ground::Newline);
        }
    }

    fn reset_state<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) {
        // Don't enforce a trailing newline.
        internals.impl_().state = State::Ground(Ground::Newline);
    }

    fn prepare_failure<Inner: WriteStr + WriteLayered>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) {
        Self::reset_state(internals);
        internals.inner_mut().abandon();
    }
}

impl Drop for TextOutput {
    fn drop(&mut self) {
        if let State::Ground(Ground::Newline) = self.state {
            // oll korrect
        } else {
            panic!("strict text stream not ended with newline");
        }
    }
}

enum Ground {
    // We just saw a '\n'.
    Newline,
    // We just saw a ZWJ or a Prepend.
    ZwjOrPrepend,
    // Otherwise.
    Other,
}

enum State {
    // Default state.
    Ground(Ground),

    // After a '\x1b'.
    Esc,

    // Within a sequence started by "\x1b[".
    Csi,
}
