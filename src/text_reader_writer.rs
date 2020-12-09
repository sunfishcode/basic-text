use crate::{
    text_input::TextInput, text_output::TextOutput, ReadStr, Utf8ReaderWriter, WriteWrapper,
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

    pub(crate) input: TextInput,
    pub(crate) output: TextOutput,
}

impl<Inner: ReadWriteExt> TextReaderWriter<Inner> {
    /// Construct a new instance of `TextReader` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner: Utf8ReaderWriter::new(inner),
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
        let utf8_reader_writer = Utf8ReaderWriter::new(inner);
        Ok(Self {
            inner: utf8_reader_writer,
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
            inner: Utf8ReaderWriter::new(inner),
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

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn abandon_into_inner(self) -> Inner {
        TextOutput::abandon_into_inner(self)
    }
}

impl<Inner: ReadWriteExt> ReadExt for TextReaderWriter<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        let (size, status) = TextInput::read_with_status(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        if size != 0 {
            TextOutput::newline(self, buf.get(size - 1).copied() == Some(b'\n'));
        }

        Ok((size, status))
    }

    #[inline]
    fn minimum_buffer_size(&self) -> usize {
        TextInput::minimum_buffer_size(self)
    }
}

impl<Inner: ReadWriteExt> ReadStr for TextReaderWriter<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        let (size, status) = TextInput::read_str(self, buf)?;

        // If the input ended with a newline, don't require the output to have
        // ended with a newline.
        if size != 0 {
            TextOutput::newline(self, buf.as_bytes().get(size - 1).copied() == Some(b'\n'));
        }

        Ok((size, status))
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
        TextInput::is_read_vectored(self)
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
        TextOutput::flush_with_status(self, status)
    }

    #[inline]
    fn abandon(&mut self) {
        TextOutput::abandon(self)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        TextOutput::write_str(self, s)
    }
}

impl<Inner: ReadWriteExt> ReadWriteExt for TextReaderWriter<Inner> {}

impl<Inner: ReadWriteExt> WriteWrapper<Inner> for TextReaderWriter<Inner> {
    #[inline]
    fn close_into_inner(self) -> io::Result<Inner> {
        TextOutput::close_into_inner(self)
    }

    #[inline]
    fn abandon_into_inner(self) -> Inner {
        TextOutput::abandon_into_inner(self)
    }
}

impl<Inner: ReadWriteExt> io::Write for TextReaderWriter<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        TextOutput::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        TextOutput::flush(self)
    }
}
