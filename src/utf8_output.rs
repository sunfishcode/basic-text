use crate::{Utf8ReaderWriter, Utf8Writer, WriteWrapper};
use io_ext::{ReadWriteExt, Status, WriteExt};
use std::{io, str};

pub(crate) trait Utf8WriterInternals<Inner: WriteExt>:
    WriteExt + WriteWrapper<Inner>
{
    fn impl_(&mut self) -> &mut Utf8Output;
    fn inner(&mut self) -> &mut Inner;
    fn into_inner(self) -> Inner;
}

impl<Inner: WriteExt> Utf8WriterInternals<Inner> for Utf8Writer<Inner> {
    fn impl_(&mut self) -> &mut Utf8Output {
        &mut self.impl_
    }

    fn inner(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }
}

impl<Inner: ReadWriteExt> Utf8WriterInternals<Inner> for Utf8ReaderWriter<Inner> {
    fn impl_(&mut self) -> &mut Utf8Output {
        &mut self.output
    }

    fn inner(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }
}

pub(crate) struct Utf8Output {}

impl Utf8Output {
    /// Construct a new instance of `Utf8Output`.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub(crate) fn close_into_inner<Inner: WriteExt>(
        mut internals: impl Utf8WriterInternals<Inner>,
    ) -> io::Result<Inner> {
        internals.flush_with_status(Status::End)?;
        Ok(internals.into_inner())
    }

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub(crate) fn abandon_into_inner<Inner: WriteExt>(
        internals: impl Utf8WriterInternals<Inner>,
    ) -> Inner {
        internals.into_inner()
    }

    #[inline]
    pub(crate) fn flush_with_status<Inner: WriteExt>(
        internals: &mut impl Utf8WriterInternals<Inner>,
        status: Status,
    ) -> io::Result<()> {
        internals.inner().flush_with_status(status)
    }

    #[inline]
    pub(crate) fn abandon<Inner: WriteExt>(internals: &mut impl Utf8WriterInternals<Inner>) {
        internals.inner().abandon()
    }

    #[inline]
    pub(crate) fn write_str<Inner: WriteExt>(
        internals: &mut impl Utf8WriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        internals.inner().write_str(s)
    }

    pub(crate) fn write<Inner: WriteExt>(
        internals: &mut impl Utf8WriterInternals<Inner>,
        buf: &[u8],
    ) -> io::Result<usize> {
        match str::from_utf8(buf) {
            Ok(s) => Self::write_str(internals, s).map(|_| buf.len()),
            Err(error) if error.valid_up_to() != 0 => internals
                .inner()
                .write_all(&buf[..error.valid_up_to()])
                .map(|_| error.valid_up_to()),
            Err(error) => {
                internals.inner().abandon();
                Err(io::Error::new(io::ErrorKind::Other, error))
            }
        }
    }

    #[inline]
    pub(crate) fn flush<Inner: WriteExt>(
        internals: &mut impl Utf8WriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.inner().flush()
    }
}
