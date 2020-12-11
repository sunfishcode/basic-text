//! Text output for `TextWriter` and the writer half of `TextInteractor`.

use crate::{
    categorize::Categorize,
    unicode::{is_normalization_form_starter, BOM, ESC, MAX_UTF8_SIZE, SUB},
    TextInteractor, TextWriter, Utf8Interactor, Utf8Writer, WriteWrapper,
};
#[cfg(can_vector)]
use io_ext::default_is_write_vectored;
#[cfg(write_all_vectored)]
use io_ext::default_write_all_vectored;
use io_ext::{default_write_vectored, Bufferable, InteractExt, WriteExt};
use std::{
    cell::RefCell,
    io::{self, Write},
    mem,
    mem::replace,
    rc::Rc,
    str,
};
use unicode_normalization::UnicodeNormalization;

pub(crate) trait TextWriterInternals<Inner: WriteExt>: WriteExt {
    type Utf8Inner: io::Write + WriteExt + WriteWrapper<Inner>;
    fn impl_(&mut self) -> &mut TextOutput;
    fn utf8_inner(&self) -> &Self::Utf8Inner;
    fn utf8_inner_mut(&mut self) -> &mut Self::Utf8Inner;
    fn into_utf8_inner(self) -> Self::Utf8Inner;
}

impl<Inner: WriteExt> TextWriterInternals<Inner> for TextWriter<Inner> {
    type Utf8Inner = Utf8Writer<Inner>;

    fn impl_(&mut self) -> &mut TextOutput {
        &mut self.output
    }

    fn utf8_inner(&self) -> &Self::Utf8Inner {
        &self.inner
    }

    fn utf8_inner_mut(&mut self) -> &mut Self::Utf8Inner {
        &mut self.inner
    }

    fn into_utf8_inner(self) -> Self::Utf8Inner {
        self.inner
    }
}

impl<Inner: InteractExt> TextWriterInternals<Inner> for TextInteractor<Inner> {
    type Utf8Inner = Utf8Interactor<Inner>;

    fn impl_(&mut self) -> &mut TextOutput {
        &mut self.output
    }

    fn utf8_inner(&self) -> &Self::Utf8Inner {
        &self.inner
    }

    fn utf8_inner_mut(&mut self) -> &mut Self::Utf8Inner {
        &mut self.inner
    }

    fn into_utf8_inner(self) -> Self::Utf8Inner {
        self.inner
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
    pub(crate) fn new() -> Self {
        Self {
            buffer: String::new(),
            crlf_compatibility: false,
            expect_starter: true,
            ansi_color: false,
            state: State::Ground(true),
            escape_sequence: String::new(),
        }
    }

    /// Like `new`, but enables CRLF output mode, which translates "\n" to
    /// "\r\n" for compatibility with consumers that need that.
    ///
    /// Note: This is not often needed; even on Windows these days most
    /// things are ok with plain '\n' line endings, [including Windows Notepad].
    /// The main notable things that really need them are IETF RFCs, for example
    /// [RFC-5198].
    ///
    /// [including Windows Notepad]: https://devblogs.microsoft.com/commandline/extended-eol-in-notepad/
    /// [RFC-5198]: https://tools.ietf.org/html/rfc5198#appendix-C
    #[inline]
    pub(crate) fn with_crlf_compatibility() -> Self {
        let mut result = Self::new();
        result.crlf_compatibility = true;
        result
    }

    #[inline]
    pub(crate) fn with_bom_compatibility<Inner: WriteExt>(
        internals: &mut Inner,
    ) -> io::Result<Self> {
        let result = Self::new();

        let mut bom_bytes = [0_u8; MAX_UTF8_SIZE];
        let bom_len = BOM.encode_utf8(&mut bom_bytes).len();
        // Safety: `bom_bytes` is valid UTF-8 because we just encoded it.
        internals.write_str(unsafe { str::from_utf8_unchecked(&bom_bytes[..bom_len]) })?;

        // The BOM is not part of the logical content, so leave the stream in
        // Ground(true) mode, meaning we don't require a newline if nothing
        // else is written to the stream.

        Ok(result)
    }

    /// Construct a new instance of `TextOutput` that optionally permits
    /// "ANSI"-style color escape sequences of the form `ESC [ ... m`.
    #[cfg(feature = "terminal-support")]
    #[inline]
    pub(crate) fn with_ansi_color(ansi_color: bool) -> Self {
        let mut result = Self::new();
        result.ansi_color = ansi_color;
        result
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    pub(crate) fn close_into_inner<Inner: WriteExt>(
        mut internals: impl TextWriterInternals<Inner>,
    ) -> io::Result<Inner> {
        Self::check_nl(&mut internals)?;
        internals.into_utf8_inner().close_into_inner()
    }

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    pub(crate) fn abandon_into_inner<Inner: WriteExt>(
        mut internals: impl TextWriterInternals<Inner>,
    ) -> Inner {
        // Don't enforce a trailing newline.
        internals.impl_().state = State::Ground(true);

        internals.into_utf8_inner().abandon_into_inner()
    }

    fn normal_write_str<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        Self::state_machine(internals, s)?;

        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn crlf_write_str<Inner: WriteExt>(
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
                impl_.state = State::Ground(true);
                impl_.buffer.push_str("\r\n");
            }

            Self::state_machine(internals, slice)?;
        }

        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn write_buffer<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) -> io::Result<()> {
        if internals.impl_().expect_starter {
            internals.impl_().expect_starter = false;
            if let Some(c) = internals.impl_().buffer.chars().next() {
                if !is_normalization_form_starter(c) {
                    internals.abandon();
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "write data must begin with a Unicode Normalization Form starter",
                    ));
                }
            }
        }

        let buffer = replace(&mut internals.impl_().buffer, String::new());
        match internals.utf8_inner_mut().write_str(&buffer) {
            Ok(()) => (),
            Err(e) => {
                internals.abandon();
                return Err(e);
            }
        }
        internals.impl_().buffer = buffer;

        // Reset the temporary buffer.
        internals.impl_().buffer.clear();

        Ok(())
    }

    fn state_machine<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        let error = Rc::new(RefCell::new(None));
        let impl_ = internals.impl_();
        for c in Categorize::new(s.chars(), Rc::clone(&error))
            .svar()
            .stream_safe()
            .nfc()
        {
            match (&impl_.state, c) {
                // Recognize ANSI-style color escape sequences.
                (State::Ground(_), ESC) if impl_.ansi_color => {
                    impl_.state = State::Esc;
                    impl_.escape_sequence.clear();
                }
                (State::Esc, '[') => {
                    impl_.state = State::Csi;
                    impl_.escape_sequence.push('[');
                }
                (State::Csi, c) if matches!(c, ' '..='?') => impl_.escape_sequence.push(c),
                (State::Csi, 'm') => {
                    impl_.state = State::Ground(false);
                    impl_.buffer.push(ESC);
                    impl_.buffer.push_str(&impl_.escape_sequence);
                    impl_.buffer.push('m');
                }

                (State::Ground(_), '\n') => {
                    impl_.state = State::Ground(true);
                    impl_.buffer.push(c)
                }

                (State::Ground(_), SUB) => {
                    // SUB indicates an error sent through the NFC iterator
                    // chain, and the Rc<RefCell<Option<io::Error>>> holds the
                    // actual error.
                    internals.abandon();
                    return Err(mem::replace(&mut *error.borrow_mut(), None).unwrap());
                }

                (State::Ground(_), ESC) => {
                    impl_.state = State::Esc;
                }

                // Common case: in ground state and reading a normal char.
                (State::Ground(_), c) => {
                    impl_.state = State::Ground(false);
                    impl_.buffer.push(c)
                }

                // Escape sequence not recognized.
                (State::Esc, c) | (State::Csi, c) => {
                    internals.abandon();
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("unrecognized escape sequence, ending in {:?}", c),
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_nl<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) -> io::Result<()> {
        match internals.impl_().state {
            State::Ground(true) => Ok(()),
            State::Ground(false) => {
                internals.abandon();
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "output text stream must end with newline",
                ))
            }
            State::Esc | State::Csi => {
                internals.abandon();
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "incomplete escape sequence at end of output text stream",
                ))
            }
        }
    }

    pub(crate) fn end<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.impl_().expect_starter = true;
        Self::check_nl(internals)?;
        internals.utf8_inner_mut().end()
    }

    pub(crate) fn abandon<Inner: WriteExt>(internals: &mut impl TextWriterInternals<Inner>) {
        internals.utf8_inner_mut().abandon();

        // Don't enforce a trailing newline.
        internals.impl_().state = State::Ground(true);
    }

    pub(crate) fn suggested_buffer_size<Inner: WriteExt>(
        internals: &impl TextWriterInternals<Inner>,
    ) -> usize {
        internals.utf8_inner().suggested_buffer_size()
    }

    pub(crate) fn write_str<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        if internals.impl_().crlf_compatibility {
            Self::crlf_write_str(internals, s)
        } else {
            Self::normal_write_str(internals, s)
        }
    }

    pub(crate) fn write<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        buf: &[u8],
    ) -> io::Result<usize> {
        match str::from_utf8(buf) {
            Ok(s) => internals.write_str(s).map(|()| buf.len()),
            // Safety: See the example code here:
            // https://doc.rust-lang.org/std/str/struct.Utf8Error.html#examples
            Err(error) if error.valid_up_to() != 0 => Self::write_str(internals, unsafe {
                str::from_utf8_unchecked(&buf[..error.valid_up_to()])
            })
            .map(|()| error.valid_up_to()),
            Err(error) => {
                internals.abandon();
                Err(io::Error::new(io::ErrorKind::Other, error))
            }
        }
    }

    #[inline]
    pub(crate) fn flush<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.impl_().expect_starter = true;
        internals.utf8_inner_mut().flush()
    }

    #[inline]
    pub(crate) fn write_vectored<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        bufs: &[io::IoSlice<'_>],
    ) -> io::Result<usize> {
        default_write_vectored(internals, bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    pub(crate) fn is_write_vectored<Inner: WriteExt>(
        internals: &impl TextWriterInternals<Inner>,
    ) -> bool {
        default_is_write_vectored(internals)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    pub(crate) fn write_all_vectored<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        bufs: &mut [io::IoSlice<'_>],
    ) -> io::Result<()> {
        default_write_all_vectored(internals, bufs)
    }

    #[inline]
    pub(crate) fn newline<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        nl: bool,
    ) {
        if nl {
            internals.impl_().state = State::Ground(true);
        }
    }
}

impl Drop for TextOutput {
    fn drop(&mut self) {
        if let State::Ground(true) = self.state {
            // oll korrect
        } else {
            panic!("output text stream not ended with newline");
        }
    }
}

enum State {
    // Default state. Boolean is true iff we just saw a '\n'.
    Ground(bool),

    // After a '\x1b'.
    Esc,

    // Within a sequence started by "\x1b[".
    Csi,
}
