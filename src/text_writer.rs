use crate::{text_writer_impl::TextWriterImpl, Utf8Writer, WriteWrapper};
use io_ext::{Status, WriteExt};
use std::{io, str};

/// A `WriteExt` implementation which translates to an output `WriteExt`
/// producing a valid plain text stream from an arbitrary byte sequence.
///
/// `write` is not guaranteed to perform a single operation, because short
/// writes could produce invalid UTF-8, so `write` will retry as needed.
pub struct TextWriter<Inner: WriteExt> {
    /// The wrapped byte stream.
    pub(crate) inner: Utf8Writer<Inner>,

    /// Temporary staging buffer.
    pub(crate) impl_: TextWriterImpl,
}

impl<Inner: WriteExt> TextWriter<Inner> {
    /// Construct a new instance of `TextWriter` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner: Utf8Writer::new(inner),
            impl_: TextWriterImpl::new(),
        }
    }

    /// Like `new`, but writes a U+FEFF (BOM) to the beginning of the output
    /// stream for compatibility with consumers that require that to determine
    /// the text encoding.
    #[inline]
    pub fn with_bom_compatibility(mut inner: Inner) -> io::Result<Self> {
        let impl_ = TextWriterImpl::with_bom_compatibility(&mut inner)?;
        Ok(Self {
            inner: Utf8Writer::new(inner),
            impl_,
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
            inner: Utf8Writer::new(inner),
            impl_: TextWriterImpl::with_crlf_compatibility(),
        }
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn close_into_inner(self) -> io::Result<Inner> {
        TextWriterImpl::close_into_inner(self)
    }

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn abandon_into_inner(self) -> Inner {
        TextWriterImpl::abandon_into_inner(self)
    }
}

impl<Inner: WriteExt> WriteExt for TextWriter<Inner> {
    #[inline]
    fn flush_with_status(&mut self, status: Status) -> io::Result<()> {
        TextWriterImpl::flush_with_status(self, status)
    }

    #[inline]
    fn abandon(&mut self) {
        TextWriterImpl::abandon(self)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        TextWriterImpl::write_str(self, s)
    }
}

impl<Inner: WriteExt> WriteWrapper<Inner> for TextWriter<Inner> {
    #[inline]
    fn close_into_inner(self) -> io::Result<Inner> {
        TextWriterImpl::close_into_inner(self)
    }

    #[inline]
    fn abandon_into_inner(self) -> Inner {
        TextWriterImpl::abandon_into_inner(self)
    }
}

impl<Inner: WriteExt> io::Write for TextWriter<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        TextWriterImpl::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        TextWriterImpl::flush(self)
    }
}

struct NlGuard(bool);

impl Drop for NlGuard {
    #[inline]
    fn drop(&mut self) {
        if !self.0 {
            panic!("output text stream not ended with newline");
        }
    }
}

#[cfg(test)]
fn translate_via_std_writer(bytes: &[u8]) -> io::Result<String> {
    use std::io::Write;
    let mut writer = TextWriter::new(io_ext_adapters::StdWriter::new(Vec::<u8>::new()));
    writer.write_all(bytes)?;
    let inner = writer.close_into_inner()?;
    Ok(String::from_utf8(inner.get_ref().to_vec()).unwrap())
}

#[cfg(test)]
fn test(bytes: &[u8], s: &str) {
    assert_eq!(translate_via_std_writer(bytes).unwrap(), s);
}

#[cfg(test)]
fn test_error(bytes: &[u8]) {
    assert!(translate_via_std_writer(bytes).is_err());
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
    test_error("\u{feff}hello\u{feff}world\u{feff}".as_bytes());
    test_error("\u{feff}hello world".as_bytes());
    test_error("hello\u{feff}world".as_bytes());
    test_error("hello world\u{feff}".as_bytes());
}

#[test]
fn test_crlf() {
    test_error(b"\r\n");
    test_error(b"\r\nhello\r\nworld\r\n");
    test_error(b"\r\nhello world");
    test_error(b"hello\r\nworld");
    test_error(b"hello world\r\n");
}

#[test]
fn test_cr_plain() {
    test_error(b"\r");
    test_error(b"\rhello\rworld\r");
    test_error(b"\rhello world");
    test_error(b"hello\rworld");
    test_error(b"hello world\r");
}

#[test]
fn test_ff() {
    test_error(b"\x0c");
    test_error(b"\x0chello\x0cworld\x0c");
    test_error(b"\x0chello world");
    test_error(b"hello\x0cworld");
    test_error(b"hello world\x0c");
}

#[test]
fn test_del() {
    test_error(b"\x7f");
    test_error(b"\x7fhello\x7fworld\x7f");
    test_error(b"\x7fhello world");
    test_error(b"hello\x7fworld");
    test_error(b"hello world\x7f");
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
}

#[test]
fn test_nfc() {
    test_error("\u{212b}\n".as_bytes());
    test("\u{c5}\n".as_bytes(), "\u{c5}\n");
    test("\u{41}\u{30a}\n".as_bytes(), "\u{c5}\n");
}

#[test]
fn test_leading_nonstarters() {
    test_error("\u{30a}".as_bytes());
}

#[test]
fn test_esc() {
    test_error(b"\x1b");
    test_error(b"\x1b@");
    test_error(b"\x1b@hello\x1b@world\x1b@");
}

#[test]
fn test_csi() {
    test_error(b"\x1b[");
    test_error(b"\x1b[@hello\x1b[@world\x1b[@");
    test_error(b"\x1b[+@hello\x1b[+@world\x1b[+@");
}

#[test]
fn test_osc() {
    test_error(b"\x1b]");
    test_error(b"\x1b]\x07hello\x1b]\x07world\x1b]\x07");
    test_error(b"\x1b]message\x07hello\x1b]message\x07world\x1b]message\x07");
    test_error(b"\x1b]mes\ns\tage\x07hello\x1b]mes\ns\tage\x07world\x1b]mes\ns\tage\x07");
}

#[test]
fn test_linux() {
    test_error(b"\x1b[[A");
    test_error(b"\x1b[[Ahello\x1b[[Aworld\x1b[[A");
}

// TODO: Test Stream-Safe
// TODO: test for nonstarter after push
