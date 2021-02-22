use crate::{text_output::TextOutput, TextStr, WriteText};
use layered_io::{Bufferable, WriteLayered};
use std::{
    fmt,
    io::{self, Write},
    str,
};
#[cfg(feature = "terminal-io")]
use terminal_io::{Terminal, TerminalColorSupport, WriteTerminal};
#[cfg(not(windows))]
use unsafe_io::os::posish::{AsRawFd, RawFd};
#[cfg(windows)]
use unsafe_io::os::windows::{AsRawHandleOrSocket, RawHandleOrSocket};
use unsafe_io::OwnsRaw;
use utf8_io::WriteStr;

/// A `WriteLayered` implementation which translates to an output
/// `WriteLayered` producing a valid Basic Text stream from an arbitrary
/// byte sequence.
///
/// `write` is not guaranteed to perform a single operation, because short
/// writes could produce invalid UTF-8, so `write` will retry as needed.
pub struct TextWriter<Inner> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// Text translation state.
    pub(crate) output: TextOutput,
}

impl<Inner: WriteStr + WriteLayered> TextWriter<Inner> {
    /// Construct a new instance of `TextWriter` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            output: TextOutput::new(),
        }
    }

    /// Like `new`, but writes a U+FEFF (BOM) to the beginning of the output
    /// stream for compatibility with consumers that require that to determine
    /// the text encoding.
    #[inline]
    pub fn with_bom_compatibility(mut inner: Inner) -> io::Result<Self> {
        let output = TextOutput::with_bom_compatibility(&mut inner)?;
        Ok(Self { inner, output })
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
            inner,
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

#[cfg(feature = "terminal-io")]
impl<Inner: WriteStr + WriteLayered + WriteTerminal> TextWriter<Inner> {
    /// Construct a new instance of `TextWriter` wrapping `inner` that
    /// optionally permits "ANSI"-style color escape sequences of the form
    /// `ESC [ ... m` on output.
    #[inline]
    pub fn with_ansi_color_output(inner: Inner) -> Self {
        let ansi_color = inner.color_support() != TerminalColorSupport::Monochrome;
        Self {
            inner,
            output: TextOutput::with_ansi_color(ansi_color),
        }
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: WriteStr + WriteLayered + WriteTerminal> Terminal for TextWriter<Inner> {}

#[cfg(feature = "terminal-io")]
impl<Inner: WriteStr + WriteLayered + WriteTerminal> WriteTerminal for TextWriter<Inner> {
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

impl<Inner: WriteStr + WriteLayered> WriteLayered for TextWriter<Inner> {
    #[inline]
    fn close(&mut self) -> io::Result<()> {
        TextOutput::close(self)
    }
}

impl<Inner: WriteStr + WriteLayered> WriteStr for TextWriter<Inner> {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        TextOutput::write_str(self, s)
    }
}

impl<Inner: WriteStr + WriteLayered> WriteText for TextWriter<Inner> {
    #[inline]
    fn write_text(&mut self, s: &TextStr) -> io::Result<()> {
        TextOutput::write_text(self, s)
    }
}

impl<Inner: WriteStr + WriteLayered> Bufferable for TextWriter<Inner> {
    #[inline]
    fn abandon(&mut self) {
        TextOutput::abandon(self)
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        TextOutput::suggested_buffer_size(self)
    }
}

impl<Inner: WriteStr + WriteLayered> Write for TextWriter<Inner> {
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
impl<Inner: WriteStr + WriteLayered + AsRawFd> AsRawFd for TextWriter<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: WriteStr + WriteLayered + AsRawHandleOrSocket> AsRawHandleOrSocket
    for TextWriter<Inner>
{
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

// Safety: `TextWriter` implements `OwnsRaw` if `Inner` does.
unsafe impl<Inner: OwnsRaw> OwnsRaw for TextWriter<Inner> {}

impl<Inner: fmt::Debug> fmt::Debug for TextWriter<Inner> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("TextWriter");
        b.field("inner", &self.inner);
        b.finish()
    }
}

#[cfg(test)]
fn translate_via_layered_writer(bytes: &[u8]) -> io::Result<String> {
    let mut writer = TextWriter::new(utf8_io::Utf8Writer::new(layered_io::LayeredWriter::new(
        Vec::<u8>::new(),
    )));
    writer.write_all(bytes)?;
    let inner = writer.close_into_inner()?.close_into_inner()?;
    Ok(String::from_utf8(inner.get_ref().to_vec()).unwrap())
}

#[cfg(test)]
fn test(bytes: &[u8], s: &str) {
    assert_eq!(translate_via_layered_writer(bytes).unwrap(), s);
}

#[cfg(test)]
fn test_error(bytes: &[u8]) {
    translate_via_layered_writer(bytes).unwrap_err();
}

#[test]
fn test_empty_string() {
    test(b"", "");
}

#[test]
fn test_no_newline() {
    test_error(b"hello");
}

#[test]
fn test_nl() {
    test(b"\n", "\n");
    test(b"\nhello\nworld\n", "\nhello\nworld\n");
}

#[test]
fn test_bom() {
    test_error("\u{feff}".as_bytes());
    test_error("\u{feff}\n".as_bytes());
    test_error("\u{feff}hello\u{feff}world\u{feff}".as_bytes());
    test_error("\u{feff}hello\u{feff}world\u{feff}\n".as_bytes());
    test_error("\u{feff}hello world\n".as_bytes());
    test_error("hello\u{feff}world\n".as_bytes());
    test_error("hello world\u{feff}".as_bytes());
    test_error("hello world\u{feff}\n".as_bytes());
}

#[test]
fn test_crlf() {
    test_error(b"\r\n");
    test_error(b"\r\nhello\r\nworld\r");
    test_error(b"\r\nhello\r\nworld\r\n");
    test_error(b"\r\nhello world\n");
    test_error(b"hello\r\nworld\n");
    test_error(b"hello world\r");
    test_error(b"hello world\r\n");
}

#[test]
fn test_cr_plain() {
    test_error(b"\r");
    test_error(b"\rhello\rworld\r");
    test_error(b"\rhello\rworld\r\n");
    test_error(b"\rhello world\n");
    test_error(b"hello\rworld\n");
    test_error(b"hello world\r");
    test_error(b"hello world\r\n");
}

#[test]
fn test_ff() {
    test_error(b"\x0c");
    test_error(b"\x0c\n");
    test_error(b"\x0chello\x0cworld\x0c");
    test_error(b"\x0chello\x0cworld\x0c\n");
    test_error(b"\x0chello world\n");
    test_error(b"hello\x0cworld\n");
    test_error(b"hello world\x0c");
    test_error(b"hello world\x0c\n");
}

#[test]
fn test_del() {
    test_error(b"\x7f");
    test_error(b"\x7f\n");
    test_error(b"\x7fhello\x7fworld\x7f");
    test_error(b"\x7fhello\x7fworld\x7f\n");
    test_error(b"\x7fhello world\n");
    test_error(b"hello\x7fworld\n");
    test_error(b"hello world\x7f");
    test_error(b"hello world\x7f\n");
}

#[test]
fn test_non_text_c0() {
    test_error(b"\x00");
    test_error(b"\x01");
    test_error(b"\x02");
    test_error(b"\x03");
    test_error(b"\x04");
    test_error(b"\x05");
    test_error(b"\x06");
    test_error(b"\x07");
    test_error(b"\x08");
    test_error(b"\x0b");
    test_error(b"\x0e");
    test_error(b"\x0f");
    test_error(b"\x10");
    test_error(b"\x11");
    test_error(b"\x12");
    test_error(b"\x13");
    test_error(b"\x14");
    test_error(b"\x15");
    test_error(b"\x16");
    test_error(b"\x17");
    test_error(b"\x18");
    test_error(b"\x19");
    test_error(b"\x1a");
    test_error(b"\x1b");
    test_error(b"\x1c");
    test_error(b"\x1d");
    test_error(b"\x1e");
    test_error(b"\x1f");

    test_error(b"\x00\n");
    test_error(b"\x01\n");
    test_error(b"\x02\n");
    test_error(b"\x03\n");
    test_error(b"\x04\n");
    test_error(b"\x05\n");
    test_error(b"\x06\n");
    test_error(b"\x07\n");
    test_error(b"\x08\n");
    test_error(b"\x0b\n");
    test_error(b"\x0e\n");
    test_error(b"\x0f\n");
    test_error(b"\x10\n");
    test_error(b"\x11\n");
    test_error(b"\x12\n");
    test_error(b"\x13\n");
    test_error(b"\x14\n");
    test_error(b"\x15\n");
    test_error(b"\x16\n");
    test_error(b"\x17\n");
    test_error(b"\x18\n");
    test_error(b"\x19\n");
    test_error(b"\x1a\n");
    test_error(b"\x1b\n");
    test_error(b"\x1c\n");
    test_error(b"\x1d\n");
    test_error(b"\x1e\n");
    test_error(b"\x1f\n");
}

#[test]
fn test_c1() {
    test_error("\u{80}".as_bytes());
    test_error("\u{81}".as_bytes());
    test_error("\u{82}".as_bytes());
    test_error("\u{83}".as_bytes());
    test_error("\u{84}".as_bytes());
    test_error("\u{85}".as_bytes());
    test_error("\u{86}".as_bytes());
    test_error("\u{87}".as_bytes());
    test_error("\u{88}".as_bytes());
    test_error("\u{89}".as_bytes());
    test_error("\u{8a}".as_bytes());
    test_error("\u{8b}".as_bytes());
    test_error("\u{8c}".as_bytes());
    test_error("\u{8d}".as_bytes());
    test_error("\u{8e}".as_bytes());
    test_error("\u{8f}".as_bytes());
    test_error("\u{90}".as_bytes());
    test_error("\u{91}".as_bytes());
    test_error("\u{92}".as_bytes());
    test_error("\u{93}".as_bytes());
    test_error("\u{94}".as_bytes());
    test_error("\u{95}".as_bytes());
    test_error("\u{96}".as_bytes());
    test_error("\u{97}".as_bytes());
    test_error("\u{98}".as_bytes());
    test_error("\u{99}".as_bytes());
    test_error("\u{9a}".as_bytes());
    test_error("\u{9b}".as_bytes());
    test_error("\u{9c}".as_bytes());
    test_error("\u{9d}".as_bytes());
    test_error("\u{9e}".as_bytes());
    test_error("\u{9f}".as_bytes());

    test_error("\u{80}\n".as_bytes());
    test_error("\u{81}\n".as_bytes());
    test_error("\u{82}\n".as_bytes());
    test_error("\u{83}\n".as_bytes());
    test_error("\u{84}\n".as_bytes());
    test_error("\u{85}\n".as_bytes());
    test_error("\u{86}\n".as_bytes());
    test_error("\u{87}\n".as_bytes());
    test_error("\u{88}\n".as_bytes());
    test_error("\u{89}\n".as_bytes());
    test_error("\u{8a}\n".as_bytes());
    test_error("\u{8b}\n".as_bytes());
    test_error("\u{8c}\n".as_bytes());
    test_error("\u{8d}\n".as_bytes());
    test_error("\u{8e}\n".as_bytes());
    test_error("\u{8f}\n".as_bytes());
    test_error("\u{90}\n".as_bytes());
    test_error("\u{91}\n".as_bytes());
    test_error("\u{92}\n".as_bytes());
    test_error("\u{93}\n".as_bytes());
    test_error("\u{94}\n".as_bytes());
    test_error("\u{95}\n".as_bytes());
    test_error("\u{96}\n".as_bytes());
    test_error("\u{97}\n".as_bytes());
    test_error("\u{98}\n".as_bytes());
    test_error("\u{99}\n".as_bytes());
    test_error("\u{9a}\n".as_bytes());
    test_error("\u{9b}\n".as_bytes());
    test_error("\u{9c}\n".as_bytes());
    test_error("\u{9d}\n".as_bytes());
    test_error("\u{9e}\n".as_bytes());
    test_error("\u{9f}\n".as_bytes());
}

#[test]
fn test_nfc() {
    test_error("\u{212b}".as_bytes());
    test_error("\u{212b}\n".as_bytes());
    test("\u{c5}\n".as_bytes(), "\u{c5}\n");
    test("\u{41}\u{30a}\n".as_bytes(), "\u{c5}\n");
}

#[test]
fn test_leading_nonstarters() {
    test_error("\u{30a}".as_bytes());
    test_error("\u{30a}\n".as_bytes());
}

#[test]
fn test_esc() {
    test_error(b"\x1b");
    test_error(b"\x1b\n");
    test_error(b"\x1b@");
    test_error(b"\x1b@\n");
    test_error(b"\x1b@hello\x1b@world\x1b@");
    test_error(b"\x1b@hello\x1b@world\x1b@\n");
}

#[test]
fn test_csi() {
    test_error(b"\x1b[");
    test_error(b"\x1b[\n");
    test_error(b"\x1b[@hello\x1b[@world\x1b[@");
    test_error(b"\x1b[@hello\x1b[@world\x1b[@\n");
    test_error(b"\x1b[+@hello\x1b[+@world\x1b[+@");
    test_error(b"\x1b[+@hello\x1b[+@world\x1b[+@\n");
}

#[test]
fn test_osc() {
    test_error(b"\x1b]");
    test_error(b"\x1b]\n");
    test_error(b"\x1b]\x07hello\x1b]\x07world\x1b]\x07");
    test_error(b"\x1b]\x07hello\x1b]\x07world\x1b]\x07\n");
    test_error(b"\x1b]message\x07hello\x1b]message\x07world\x1b]message\x07");
    test_error(b"\x1b]message\x07hello\x1b]message\x07world\x1b]message\x07\n");
    test_error(b"\x1b]mes\ns\tage\x07hello\x1b]mes\ns\tage\x07world\x1b]mes\ns\tage\x07");
    test_error(b"\x1b]mes\ns\tage\x07hello\x1b]mes\ns\tage\x07world\x1b]mes\ns\tage\x07\n");
}

#[test]
fn test_linux() {
    test_error(b"\x1b[[A");
    test_error(b"\x1b[[A\n");
    test_error(b"\x1b[[Ahello\x1b[[Aworld\x1b[[A");
    test_error(b"\x1b[[Ahello\x1b[[Aworld\x1b[[A\n");
}

#[test]
fn test_unassigned() {
    test_error("\u{1d455}".as_bytes());
    test_error("\u{1d455}\n".as_bytes());
}

// TODO: Test Stream-Safe
// TODO: test for nonstarter after push
