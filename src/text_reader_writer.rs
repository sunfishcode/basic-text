use crate::{
    text_reader_impl::TextReaderImpl, text_writer_impl::TextWriterImpl, ReadStr, Utf8ReaderWriter,
    WriteWrapper,
};
use io_ext::{
    default_read, default_read_exact, default_read_to_end, default_read_to_string,
    default_read_vectored, ReadExt, ReadWriteExt, Status, WriteExt,
};
use std::{io, str};

/// The combination of `TextReader` and `TextWriter`.
pub struct TextReaderWriter<Inner: ReadWriteExt> {
    /// The wrapped byte stream.
    pub(crate) inner: Utf8ReaderWriter<Inner>,

    pub(crate) reader_impl: TextReaderImpl,
    pub(crate) writer_impl: TextWriterImpl,
}

impl<Inner: ReadWriteExt> TextReaderWriter<Inner> {
    /// Construct a new instance of `TextReader` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner: Utf8ReaderWriter::new(inner),
            reader_impl: TextReaderImpl::new(),
            writer_impl: TextWriterImpl::new(),
        }
    }

    /// Like `new`, but writes a U+FEFF (BOM) to the beginning of the output
    /// stream for compatibility with consumers that require that to determine
    /// the text encoding.
    #[inline]
    pub fn with_bom_compatibility(mut inner: Inner) -> io::Result<Self> {
        let writer_impl = TextWriterImpl::with_bom_compatibility(&mut inner)?;
        let utf8_reader_writer = Utf8ReaderWriter::new(inner);
        Ok(Self {
            inner: utf8_reader_writer,
            reader_impl: TextReaderImpl::new(),
            writer_impl,
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
            inner: Utf8ReaderWriter::new(inner),
            reader_impl: TextReaderImpl::new(),
            writer_impl: TextWriterImpl::with_crlf_compatibility(),
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

impl<Inner: ReadWriteExt> ReadExt for TextReaderWriter<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        let size_and_status = TextReaderImpl::read_with_status(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        if size_and_status.0 != 0 {
            TextWriterImpl::newline(self, buf.get(size_and_status.0 - 1).copied() == Some(b'\n'));
        }

        Ok(size_and_status)
    }
}

impl<Inner: ReadWriteExt> ReadStr for TextReaderWriter<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        let size_and_status = TextReaderImpl::read_str(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        if size_and_status.0 != 0 {
            TextWriterImpl::newline(
                self,
                buf.as_bytes().get(size_and_status.0 - 1).copied() == Some(b'\n'),
            );
        }

        Ok(size_and_status)
    }

    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        TextReaderImpl::read_exact_str(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        TextWriterImpl::newline(
            self,
            buf.as_bytes().get(buf.len() - 1).copied() == Some(b'\n'),
        );

        Ok(())
    }
}

impl<Inner: ReadWriteExt> io::Read for TextReaderWriter<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        default_read(self, buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        default_read_vectored(self, bufs)
    }

    #[cfg(feature = "nightly")]
    #[inline]
    fn is_read_vectored(&self) -> bool {
        TextReaderImpl::is_read_vectored(self)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        default_read_to_end(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        default_read_to_string(self, buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        default_read_exact(self, buf)
    }
}

impl<Inner: ReadWriteExt> WriteExt for TextReaderWriter<Inner> {
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

impl<Inner: ReadWriteExt> ReadWriteExt for TextReaderWriter<Inner> {}

impl<Inner: ReadWriteExt> WriteWrapper<Inner> for TextReaderWriter<Inner> {
    #[inline]
    fn close_into_inner(self) -> io::Result<Inner> {
        TextWriterImpl::close_into_inner(self)
    }

    #[inline]
    fn abandon_into_inner(self) -> Inner {
        TextWriterImpl::abandon_into_inner(self)
    }
}

impl<Inner: ReadWriteExt> io::Write for TextReaderWriter<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        TextWriterImpl::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        TextWriterImpl::flush(self)
    }
}
