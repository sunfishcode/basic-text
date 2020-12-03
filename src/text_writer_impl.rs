use crate::{
    unicode::{is_normalization_form_starter, BOM, MAX_UTF8_SIZE},
    TextReaderWriter, TextWriter, Utf8ReaderWriter, Utf8Writer, WriteWrapper,
};
use io_ext::{default_flush, ReadWriteExt, Status, WriteExt};
use std::{io, mem::replace, str};
use unicode_normalization::UnicodeNormalization;

pub(crate) trait TextWriterInternals<Inner: WriteExt>: WriteExt {
    type Utf8Inner: io::Write + WriteExt + WriteWrapper<Inner>;
    fn impl_(&mut self) -> &mut TextWriterImpl;
    fn utf8_inner(&mut self) -> &mut Self::Utf8Inner;
    fn into_utf8_inner(self) -> Self::Utf8Inner;
}

impl<Inner: WriteExt> TextWriterInternals<Inner> for TextWriter<Inner> {
    type Utf8Inner = Utf8Writer<Inner>;

    fn impl_(&mut self) -> &mut TextWriterImpl {
        &mut self.impl_
    }

    fn utf8_inner(&mut self) -> &mut Self::Utf8Inner {
        &mut self.inner
    }

    fn into_utf8_inner(self) -> Self::Utf8Inner {
        self.inner
    }
}

impl<Inner: ReadWriteExt> TextWriterInternals<Inner> for TextReaderWriter<Inner> {
    type Utf8Inner = Utf8ReaderWriter<Inner>;

    fn impl_(&mut self) -> &mut TextWriterImpl {
        &mut self.writer_impl
    }

    fn utf8_inner(&mut self) -> &mut Self::Utf8Inner {
        &mut self.inner
    }

    fn into_utf8_inner(self) -> Self::Utf8Inner {
        self.inner
    }
}

pub(crate) struct TextWriterImpl {
    /// Temporary staging buffer.
    buffer: String,

    /// True if the last byte written was a '\n'.
    nl: NlGuard,

    /// When enabled, "\n" is replaced by "\r\n".
    crlf_compatibility: bool,

    /// At the beginning of a stream or after a push, expect a
    /// normalization-form starter.
    expect_starter: bool,
}

impl TextWriterImpl {
    /// Construct a new instance of `TextWriterImpl`.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            buffer: String::new(),
            nl: NlGuard(true),
            crlf_compatibility: false,
            expect_starter: true,
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
        Self {
            buffer: String::new(),
            nl: NlGuard(true),
            crlf_compatibility: true,
            expect_starter: true,
        }
    }

    #[inline]
    pub(crate) fn write_bom<Inner: WriteExt>(inner: &mut Inner) -> io::Result<()> {
        let mut bom_bytes = [0_u8; MAX_UTF8_SIZE];
        let bom_len = BOM.encode_utf8(&mut bom_bytes).len();
        inner.write_str(unsafe { str::from_utf8_unchecked(&bom_bytes[..bom_len]) })
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    pub(crate) fn close_into_inner<Inner: WriteExt>(
        mut internals: impl TextWriterInternals<Inner>,
    ) -> io::Result<Inner> {
        Self::check_nl(&mut internals, Status::End)?;
        internals.into_utf8_inner().close_into_inner()
    }

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    pub(crate) fn abandon_into_inner<Inner: WriteExt>(
        internals: impl TextWriterInternals<Inner>,
    ) -> Inner {
        internals.into_utf8_inner().abandon_into_inner()
    }

    fn normal_write_str<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        internals
            .impl_()
            .buffer
            .extend(s.chars().stream_safe().nfc());

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
                internals.impl_().buffer.push_str("\r\n");
            }
            internals
                .impl_()
                .buffer
                .extend(slice.chars().stream_safe().nfc());
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

        if internals
            .impl_()
            .buffer
            .chars()
            .any(|c| (c.is_control() && c != '\n' && c != '\t') || c == BOM)
        {
            internals.abandon();
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid Unicode scalar value written to text stream",
            ));
        }

        let buffer = replace(&mut internals.impl_().buffer, String::new());
        match internals.utf8_inner().write_str(&buffer) {
            Ok(()) => (),
            Err(e) => {
                internals.abandon();
                return Err(e);
            }
        }
        internals.impl_().buffer = buffer;

        if let Some(last) = internals.impl_().buffer.as_bytes().last().copied() {
            Self::newline(internals, last == b'\n');
        }

        // Reset the temporary buffer.
        internals.impl_().buffer.clear();

        Ok(())
    }

    fn check_nl<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        status: Status,
    ) -> io::Result<()> {
        match status {
            Status::End => {
                if !internals.impl_().nl.0 {
                    internals.abandon();
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "output text stream must end with newline",
                    ));
                }
            }
            Status::Open(_) => (),
        }
        Ok(())
    }

    pub(crate) fn flush_with_status<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        status: Status,
    ) -> io::Result<()> {
        if status != Status::active() {
            internals.impl_().expect_starter = true;
        }
        Self::check_nl(internals, status)?;
        internals.utf8_inner().flush_with_status(status)
    }

    pub(crate) fn abandon<Inner: WriteExt>(internals: &mut impl TextWriterInternals<Inner>) {
        internals.utf8_inner().abandon();

        // Don't enforce a trailing newline.
        internals.impl_().nl.0 = true;
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
            Ok(s) => internals.write_str(s).map(|_| buf.len()),
            Err(error) if error.valid_up_to() != 0 => Self::write_str(internals, unsafe {
                str::from_utf8_unchecked(&buf[..error.valid_up_to()])
            })
            .map(|_| buf.len()),
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
        default_flush(internals)
    }

    #[inline]
    pub(crate) fn newline<Inner: WriteExt>(
        internals: &mut impl TextWriterInternals<Inner>,
        nl: bool,
    ) {
        internals.impl_().nl.0 = nl;
    }
}

struct NlGuard(bool);

impl Drop for NlGuard {
    fn drop(&mut self) {
        if !self.0 {
            panic!("output text stream not ended with newline");
        }
    }
}
