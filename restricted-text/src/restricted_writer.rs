use crate::{restricted_output::RestrictedOutput, RestrictedStr, WriteRestricted};
use layered_io::{Bufferable, WriteLayered};
#[cfg(not(windows))]
use unsafe_io::os::posish::{AsRawFd, RawFd};
use std::{
    fmt,
    io::{self, Write},
    str,
};
#[cfg(windows)]
use unsafe_io::os::windows::{AsRawHandleOrSocket, RawHandleOrSocket};
use unsafe_io::OwnsRaw;
use utf8_io::WriteStr;
#[cfg(test)]
use utf8_io::Utf8Writer;
use basic_text::{TextStr, WriteText};

/// A `WriteLayered` implementation which translates to an output
/// `WriteLayered` producing a valid Restricted Text stream from an arbitrary
/// byte sequence.
///
/// `write` is not guaranteed to perform a single operation, because short
/// writes could produce invalid UTF-8, so `write` will retry as needed.
pub struct RestrictedWriter<Inner> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// Text translation state.
    pub(crate) output: RestrictedOutput,
}

impl<Inner: WriteStr + WriteLayered> RestrictedWriter<Inner> {
    /// Construct a new instance of `RestrictedWriter` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            output: RestrictedOutput::new(),
        }
    }

    /// Like `new`, but writes a U+FEFF (BOM) to the beginning of the output
    /// stream for compatibility with consumers that require that to determine
    /// the text encoding.
    #[inline]
    pub fn with_bom_compatibility(mut inner: Inner) -> io::Result<Self> {
        let output = RestrictedOutput::with_bom_compatibility(&mut inner)?;
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
            output: RestrictedOutput::with_crlf_compatibility(),
        }
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn close_into_inner(self) -> io::Result<Inner> {
        RestrictedOutput::close_into_inner(self)
    }

    /// Return the underlying stream object.
    #[inline]
    pub fn abandon_into_inner(self) -> Inner {
        RestrictedOutput::abandon_into_inner(self)
    }
}

impl<Inner: WriteStr + WriteLayered> WriteLayered for RestrictedWriter<Inner> {
    #[inline]
    fn close(&mut self) -> io::Result<()> {
        RestrictedOutput::close(self)
    }
}

impl<Inner: WriteStr + WriteLayered> WriteStr for RestrictedWriter<Inner> {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        RestrictedOutput::write_str(self, s)
    }
}

impl<Inner: WriteStr + WriteLayered> WriteText for RestrictedWriter<Inner> {
    #[inline]
    fn write_text(&mut self, s: &TextStr) -> io::Result<()> {
        RestrictedOutput::write_text(self, s)
    }
}

impl<Inner: WriteText + WriteLayered> WriteRestricted for RestrictedWriter<Inner> {
    #[inline]
    fn write_restricted(&mut self, s: &RestrictedStr) -> io::Result<()> {
        RestrictedOutput::write_restricted(self, s)
    }
}

impl<Inner: WriteStr + WriteLayered> Bufferable for RestrictedWriter<Inner> {
    #[inline]
    fn abandon(&mut self) {
        RestrictedOutput::abandon(self)
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        RestrictedOutput::suggested_buffer_size(self)
    }
}

impl<Inner: WriteStr + WriteLayered> Write for RestrictedWriter<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        RestrictedOutput::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        RestrictedOutput::flush(self)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_write_vectored(&self) -> bool {
        RestrictedOutput::is_write_vectored(self)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        RestrictedOutput::write_vectored(self, bufs)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        RestrictedOutput::write_all_vectored(self, bufs)
    }
}

#[cfg(not(windows))]
impl<Inner: WriteStr + WriteLayered + AsRawFd> AsRawFd for RestrictedWriter<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: WriteStr + WriteLayered + AsRawHandleOrSocket> AsRawHandleOrSocket
    for RestrictedWriter<Inner>
{
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

// Safety: `RestrictedWriter` implements `OwnsRaw` if `Inner` does.
unsafe impl<Inner: OwnsRaw> OwnsRaw for RestrictedWriter<Inner> {}

impl<Inner: fmt::Debug> fmt::Debug for RestrictedWriter<Inner> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("RestrictedWriter");
        b.field("inner", &self.inner);
        b.finish()
    }
}

#[cfg(test)]
fn translate_via_layered_writer(bytes: &[u8]) -> io::Result<String> {
    let mut writer = RestrictedWriter::new(TextWriter::new(Utf8Writer::new(layered_io::LayeredWriter::new(
        Vec::<u8>::new(),
    ))));
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
