use crate::{
    text_input::TextInput, text_output::TextOutput, ReadText, ReadTextLayered, TextSubstr,
    WriteText,
};
use duplex::{Duplex, HalfDuplex};
use layered_io::{
    default_read_to_end, Bufferable, HalfDuplexLayered, LayeredDuplexer, ReadLayered, Status,
    WriteLayered,
};
use std::{
    cmp::max,
    fmt::{self, Debug, Formatter},
    io::{self, Read, Write},
    str,
};
#[cfg(feature = "terminal-io")]
use terminal_io::{DuplexTerminal, ReadTerminal, Terminal, TerminalColorSupport, WriteTerminal};
#[cfg(windows)]
use unsafe_io::os::windows::{
    AsHandleOrSocket, AsRawHandleOrSocket, BorrowedHandleOrSocket, RawHandleOrSocket,
};
use utf8_io::{ReadStr, ReadStrLayered, Utf8Duplexer, WriteStr};
#[cfg(not(windows))]
use {
    io_lifetimes::{AsFd, BorrowedFd},
    unsafe_io::os::posish::{AsRawFd, RawFd},
};

/// A [`HalfDuplex`] implementation which translates from an input `HalfDuplex`
/// implementation producing an arbitrary byte sequence into a valid Basic Text
/// stream.
pub struct TextDuplexer<Inner: HalfDuplex + ReadStr + WriteStr> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// Text translation state.
    pub(crate) input: TextInput,
    pub(crate) output: TextOutput,
}

impl<Inner: HalfDuplex> TextDuplexer<Utf8Duplexer<LayeredDuplexer<Inner>>> {
    /// Construct a new instance of `TextDuplexer` wrapping `inner`, which can be
    /// anything that implements [`HalfDuplex`].
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self::from_utf8(Utf8Duplexer::new(LayeredDuplexer::new(inner)))
    }

    /// Like `new`, but writes a U+FEFF (BOM) to the beginning of the output
    /// stream for compatibility with consumers that require that to determine
    /// the text encoding.
    #[inline]
    pub fn with_bom_compatibility(inner: Inner) -> io::Result<Self> {
        Self::from_utf8_with_bom_compatibility(Utf8Duplexer::new(LayeredDuplexer::new(inner)))
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
        Self::from_utf8_with_crlf_compatibility(Utf8Duplexer::new(LayeredDuplexer::new(inner)))
    }

    /// Like `new`, but replaces U+85 (NEL) with U+A instead of U+20.
    #[inline]
    pub fn with_nel_compatibility(inner: Inner) -> io::Result<Self> {
        Self::from_utf8_with_nel_compatibility(Utf8Duplexer::new(LayeredDuplexer::new(inner)))
    }

    /// Like `new`, but replaces U+2028 (LS) and U+2029 (PS) with U+A instead
    /// of U+20.
    #[inline]
    pub fn with_lsps_compatibility(inner: Inner) -> io::Result<Self> {
        Self::from_utf8_with_lsps_compatibility(Utf8Duplexer::new(LayeredDuplexer::new(inner)))
    }
}

impl<Inner: HalfDuplex + ReadStr + ReadLayered + ReadStrLayered + WriteStr + WriteLayered>
    TextDuplexer<Inner>
{
    /// Construct a new instance of `TextDuplexer` wrapping `inner`.
    #[inline]
    pub fn from_utf8(inner: Inner) -> Self {
        Self {
            inner,
            input: TextInput::new(),
            output: TextOutput::new(),
        }
    }

    /// Like `from_utf8`, but writes a U+FEFF (BOM) to the beginning of the output
    /// stream for compatibility with consumers that require that to determine
    /// the text encoding.
    #[inline]
    pub fn from_utf8_with_bom_compatibility(mut inner: Inner) -> io::Result<Self> {
        let output = TextOutput::with_bom_compatibility(&mut inner)?;
        Ok(Self {
            inner,
            input: TextInput::new(),
            output,
        })
    }

    /// Like `from_utf8`, but enables CRLF output mode, which translates "\n" to
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
    pub fn from_utf8_with_crlf_compatibility(inner: Inner) -> Self {
        Self {
            inner,
            input: TextInput::new(),
            output: TextOutput::with_crlf_compatibility(),
        }
    }

    /// Like `from_utf8`, but replaces U+85 (NEL) with U+A instead of U+20.
    #[inline]
    pub fn from_utf8_with_nel_compatibility(inner: Inner) -> io::Result<Self> {
        let input = TextInput::with_nel_compatibility();
        let output = TextOutput::new();
        Ok(Self {
            inner,
            input,
            output,
        })
    }

    /// Like `from_utf8`, but replaces U+2028 (LS) and U+2029 (PS) with U+A
    /// instead of U+20.
    #[inline]
    pub fn from_utf8_with_lsps_compatibility(inner: Inner) -> io::Result<Self> {
        let input = TextInput::with_lsps_compatibility();
        let output = TextOutput::new();
        Ok(Self {
            inner,
            input,
            output,
        })
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

#[cfg(feature = "terminal-io")]
impl<Inner: HalfDuplex + ReadStr + WriteStr + DuplexTerminal> TextDuplexer<Inner> {
    /// Construct a new instance of `TextWriter` wrapping `inner` that
    /// optionally permits "ANSI"-style color escape sequences of the form
    /// `ESC [ ... m` on output.
    #[inline]
    pub fn with_ansi_color_output(inner: Inner) -> Self {
        let ansi_color = inner.color_support() != TerminalColorSupport::Monochrome;
        Self {
            inner,
            input: TextInput::new(),
            output: TextOutput::with_ansi_color(ansi_color),
        }
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: HalfDuplex + ReadStr + WriteStr + DuplexTerminal> Terminal for TextDuplexer<Inner> {}

#[cfg(feature = "terminal-io")]
impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr + DuplexTerminal> ReadTerminal
    for TextDuplexer<Inner>
{
    #[inline]
    fn is_line_by_line(&self) -> bool {
        self.inner.is_line_by_line()
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        self.inner.is_input_terminal()
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr + DuplexTerminal> WriteTerminal
    for TextDuplexer<Inner>
{
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

#[cfg(feature = "terminal-io")]
impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr + DuplexTerminal> DuplexTerminal
    for TextDuplexer<Inner>
{
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> ReadLayered for TextDuplexer<Inner> {
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

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> Bufferable for TextDuplexer<Inner> {
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

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> ReadStr for TextDuplexer<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<usize> {
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

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> ReadStrLayered for TextDuplexer<Inner> {
    #[inline]
    fn read_str_with_status(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        TextInput::read_str_with_status(self, buf)
    }

    #[inline]
    fn read_exact_str_using_status(&mut self, buf: &mut str) -> io::Result<Status> {
        TextInput::read_exact_str_using_status(self, buf)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> ReadText for TextDuplexer<Inner> {
    #[inline]
    fn read_text_substr(&mut self, buf: &mut TextSubstr) -> io::Result<usize> {
        TextInput::read_text_substr(self, buf)
    }

    #[inline]
    fn read_exact_text_substr(&mut self, buf: &mut TextSubstr) -> io::Result<()> {
        TextInput::read_exact_text_substr(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        TextOutput::newline(
            self,
            buf.as_bytes().get(buf.len() - 1).copied() == Some(b'\n'),
        );

        Ok(())
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> ReadTextLayered for TextDuplexer<Inner> {
    #[inline]
    fn read_text_substr_with_status(
        &mut self,
        buf: &mut TextSubstr,
    ) -> io::Result<(usize, Status)> {
        TextInput::read_text_substr_with_status(self, buf)
    }

    #[inline]
    fn read_exact_text_substr_using_status(&mut self, buf: &mut TextSubstr) -> io::Result<Status> {
        TextInput::read_exact_text_substr_using_status(self, buf)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> Read for TextDuplexer<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        TextInput::read(self, buf)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        default_read_to_end(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        TextInput::read_to_string(self, buf)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> WriteLayered for TextDuplexer<Inner> {
    #[inline]
    fn close(&mut self) -> io::Result<()> {
        TextOutput::close(self)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> WriteStr for TextDuplexer<Inner> {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        TextOutput::write_str(self, s)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> WriteText for TextDuplexer<Inner> {
    #[inline]
    fn write_text_substr(&mut self, s: &TextSubstr) -> io::Result<()> {
        TextOutput::write_text_substr(self, s)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> Duplex for TextDuplexer<Inner> {}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> Write for TextDuplexer<Inner> {
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
impl<Inner: HalfDuplexLayered + ReadStr + WriteStr + AsRawFd> AsRawFd for TextDuplexer<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(not(windows))]
impl<Inner: HalfDuplexLayered + ReadStr + WriteStr + AsFd> AsFd for TextDuplexer<Inner> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

#[cfg(windows)]
impl<Inner: HalfDuplexLayered + ReadStr + WriteStr + AsRawHandleOrSocket> AsRawHandleOrSocket
    for TextDuplexer<Inner>
{
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl<Inner: HalfDuplexLayered + ReadStr + WriteStr + AsHandleOrSocket> AsHandleOrSocket
    for TextDuplexer<Inner>
{
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.inner.as_handle_or_socket()
    }
}

impl<Inner: HalfDuplexLayered + ReadStr + WriteStr + Debug> Debug for TextDuplexer<Inner> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("TextDuplexer");
        b.field("inner", &self.inner);
        b.finish()
    }
}
