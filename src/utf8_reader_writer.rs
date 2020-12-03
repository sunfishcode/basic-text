use crate::{
    utf8_reader_impl::Utf8ReaderImpl, utf8_writer_impl::Utf8WriterImpl, ReadStr, WriteWrapper,
};
use io_ext::{ReadExt, ReadWriteExt, Status, WriteExt};
use std::{io, str};

/// The combination of `Utf8Reader` and `Utf8Writer`.
pub struct Utf8ReaderWriter<Inner: ReadWriteExt> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    pub(crate) reader_impl: Utf8ReaderImpl,
    pub(crate) writer_impl: Utf8WriterImpl,
}

impl<Inner: ReadWriteExt> Utf8ReaderWriter<Inner> {
    /// Construct a new instance of `Utf8ReaderWriter` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            reader_impl: Utf8ReaderImpl::new(),
            writer_impl: Utf8WriterImpl::new(),
        }
    }
}

impl<Inner: ReadWriteExt> ReadStr for Utf8ReaderWriter<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        Utf8ReaderImpl::read_str(self, buf)
    }

    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        Utf8ReaderImpl::read_exact_str(self, buf)
    }
}

impl<Inner: ReadWriteExt> ReadExt for Utf8ReaderWriter<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        Utf8ReaderImpl::read_with_status(self, buf)
    }
}

impl<Inner: ReadWriteExt> io::Read for Utf8ReaderWriter<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Utf8ReaderImpl::read(self, buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        Utf8ReaderImpl::read_vectored(self, bufs)
    }

    #[cfg(feature = "nightly")]
    #[inline]
    fn is_read_vectored(&self) -> bool {
        Utf8ReaderImpl::is_vectored(self)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        Utf8ReaderImpl::read_to_end(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        Utf8ReaderImpl::read_to_string(self, buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        Utf8ReaderImpl::read_exact(self, buf)
    }
}

impl<Inner: ReadWriteExt> WriteExt for Utf8ReaderWriter<Inner> {
    #[inline]
    fn flush_with_status(&mut self, status: Status) -> io::Result<()> {
        Utf8WriterImpl::flush_with_status(self, status)
    }

    #[inline]
    fn abandon(&mut self) {
        Utf8WriterImpl::abandon(self)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        Utf8WriterImpl::write_str(self, s)
    }
}

impl<Inner: ReadWriteExt> ReadWriteExt for Utf8ReaderWriter<Inner> {}

impl<Inner: ReadWriteExt> WriteWrapper<Inner> for Utf8ReaderWriter<Inner> {
    #[inline]
    fn close_into_inner(self) -> io::Result<Inner> {
        Utf8WriterImpl::close_into_inner(self)
    }

    #[inline]
    fn abandon_into_inner(self) -> Inner {
        Utf8WriterImpl::abandon_into_inner(self)
    }
}

impl<Inner: ReadWriteExt> io::Write for Utf8ReaderWriter<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Utf8WriterImpl::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Utf8WriterImpl::flush(self)
    }
}
