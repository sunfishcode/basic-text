use crate::{
    text_input::TextInput, text_output::TextOutput, ReadStr, ReadText, TextStr, Utf8Interactor,
    WriteText, WriteWrapper,
};
use io_ext::{Bufferable, InteractExt, ReadExt, Status, WriteExt};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, RawFd};
use std::{
    cmp::max,
    fmt,
    io::{self, Read, Write},
    str,
};
#[cfg(feature = "terminal-support")]
use terminal_support::{
    InteractTerminal, ReadTerminal, Terminal, TerminalColorSupport, WriteTerminal,
};
#[cfg(windows)]
use unsafe_io::{AsRawHandleOrSocket, RawHandleOrSocket};

/// An `InteractExt` implementation which translates to and from an inner
/// `InteractExt` producing a valid Basic Text interactive stream from
/// an arbitrary interactive byte stream.
pub struct TextInteractor<Inner> {
    /// The wrapped byte stream.
    pub(crate) inner: Utf8Interactor<Inner>,

    /// Text translation state.
    pub(crate) input: TextInput,
    pub(crate) output: TextOutput,
}

impl<Inner: InteractExt> TextInteractor<Inner> {
    /// Construct a new instance of `TextReader` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner: Utf8Interactor::new(inner),
            input: TextInput::new(),
            output: TextOutput::new(),
        }
    }

    /// Like `new`, but writes a U+FEFF (BOM) to the beginning of the output
    /// stream for compatibility with consumers that require that to determine
    /// the text encoding.
    #[inline]
    pub fn with_bom_compatibility(mut inner: Inner) -> io::Result<Self> {
        let output = TextOutput::with_bom_compatibility(&mut inner)?;
        let utf8_interactor = Utf8Interactor::new(inner);
        Ok(Self {
            inner: utf8_interactor,
            input: TextInput::new(),
            output,
        })
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
    pub fn with_crlf_compatibility(inner: Inner) -> Self {
        Self {
            inner: Utf8Interactor::new(inner),
            input: TextInput::new(),
            output: TextOutput::with_crlf_compatibility(),
        }
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn close_into_inner(self) -> io::Result<Inner> {
        TextOutput::close_into_inner(self)
    }

    /// Return the underlying stream object.
    #[inline]
    pub fn abandon_into_inner(self) -> Inner {
        TextOutput::abandon_into_inner(self)
    }
}

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> TextInteractor<Inner> {
    /// Construct a new instance of `TextWriter` wrapping `inner` that
    /// optionally permits "ANSI"-style color escape sequences of the form
    /// `ESC [ ... m` on output.
    #[inline]
    pub fn with_ansi_color_output(inner: Inner) -> Self {
        let ansi_color = inner.color_support() != TerminalColorSupport::Monochrome;
        Self {
            inner: Utf8Interactor::new(inner),
            input: TextInput::new(),
            output: TextOutput::with_ansi_color(ansi_color),
        }
    }
}

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> Terminal for TextInteractor<Inner> {}

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> ReadTerminal for TextInteractor<Inner> {
    #[inline]
    fn is_line_by_line(&self) -> bool {
        self.inner.is_line_by_line()
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        self.inner.is_input_terminal()
    }
}

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> WriteTerminal for TextInteractor<Inner> {
    #[inline]
    fn color_support(&self) -> TerminalColorSupport {
        self.inner.color_support()
    }

    #[inline]
    fn color_preference(&self) -> bool {
        self.inner.color_preference()
    }

    #[inline]
    fn is_output_terminal(&self) -> bool {
        self.inner.is_output_terminal()
    }
}

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> InteractTerminal for TextInteractor<Inner> {}

impl<Inner: InteractExt> ReadExt for TextInteractor<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        let (size, status) = TextInput::read_with_status(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        if size != 0 && buf.get(size - 1).copied() == Some(b'\n') {
            TextOutput::newline(self, true);
        }

        Ok((size, status))
    }

    #[inline]
    fn minimum_buffer_size(&self) -> usize {
        TextInput::minimum_buffer_size(self)
    }
}

impl<Inner: InteractExt> Bufferable for TextInteractor<Inner> {
    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        max(
            TextInput::suggested_buffer_size(self),
            TextOutput::suggested_buffer_size(self),
        )
    }

    #[inline]
    fn abandon(&mut self) {
        TextInput::abandon(self);
        TextOutput::abandon(self);
    }
}

impl<Inner: InteractExt> ReadStr for TextInteractor<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        TextInput::read_str(self, buf)
    }

    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        TextInput::read_exact_str(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        TextOutput::newline(
            self,
            buf.as_bytes().get(buf.len() - 1).copied() == Some(b'\n'),
        );

        Ok(())
    }
}

impl<Inner: InteractExt> ReadText for TextInteractor<Inner> {
    #[inline]
    fn read_text(&mut self, buf: &mut TextStr) -> io::Result<(usize, Status)> {
        TextInput::read_text(self, buf)
    }

    #[inline]
    fn read_exact_text(&mut self, buf: &mut TextStr) -> io::Result<()> {
        TextInput::read_exact_text(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        TextOutput::newline(
            self,
            buf.as_bytes().get(buf.len() - 1).copied() == Some(b'\n'),
        );

        Ok(())
    }
}

impl<Inner: InteractExt> Read for TextInteractor<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        TextInput::read(self, buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        TextInput::read_vectored(self, bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_read_vectored(&self) -> bool {
        TextInput::is_read_vectored(self)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        TextInput::read_to_end(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        TextInput::read_to_string(self, buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        TextInput::read_exact(self, buf)
    }
}

impl<Inner: InteractExt> WriteExt for TextInteractor<Inner> {
    #[inline]
    fn close(&mut self) -> io::Result<()> {
        TextOutput::close(self)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        TextOutput::write_str(self, s)
    }
}

impl<Inner: InteractExt> WriteText for TextInteractor<Inner> {
    #[inline]
    fn write_text(&mut self, s: &TextStr) -> io::Result<()> {
        TextOutput::write_text(self, s)
    }
}

impl<Inner: InteractExt> InteractExt for TextInteractor<Inner> {}

impl<Inner: InteractExt> WriteWrapper<Inner> for TextInteractor<Inner> {
    #[inline]
    fn close_into_inner(self) -> io::Result<Inner> {
        TextOutput::close_into_inner(self)
    }

    #[inline]
    fn abandon_into_inner(self) -> Inner {
        TextOutput::abandon_into_inner(self)
    }
}

impl<Inner: InteractExt> Write for TextInteractor<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        TextOutput::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        TextOutput::flush(self)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_write_vectored(&self) -> bool {
        TextOutput::is_write_vectored(self)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        TextOutput::write_vectored(self, bufs)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        TextOutput::write_all_vectored(self, bufs)
    }
}

#[cfg(not(windows))]
impl<Inner: InteractExt + AsRawFd> AsRawFd for TextInteractor<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: InteractExt + AsRawHandleOrSocket> AsRawHandleOrSocket for TextInteractor<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

impl<Inner: fmt::Debug> fmt::Debug for TextInteractor<Inner> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("TextInteractor");
        b.field("inner", &self.inner);
        b.finish()
    }
}
