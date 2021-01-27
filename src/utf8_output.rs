use crate::{default_write_str, Utf8Interactor, Utf8Writer};
use interact_trait::InteractExt;
use io_ext::WriteExt;
use std::{io, str};

pub(crate) trait Utf8WriterInternals<Inner: WriteExt>: WriteExt {
    fn impl_(&mut self) -> &mut Utf8Output;
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
    fn into_inner(self) -> Inner;
}

impl<Inner: WriteExt> Utf8WriterInternals<Inner> for Utf8Writer<Inner> {
    fn impl_(&mut self) -> &mut Utf8Output {
        &mut self.output
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }
}

impl<Inner: InteractExt> Utf8WriterInternals<Inner> for Utf8Interactor<Inner> {
    fn impl_(&mut self) -> &mut Utf8Output {
        &mut self.output
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
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
    pub(crate) const fn new() -> Self {
        Self {}
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub(crate) fn close_into_inner<Inner: WriteExt>(
        mut internals: impl Utf8WriterInternals<Inner>,
    ) -> io::Result<Inner> {
        internals.flush()?;
        Ok(internals.into_inner())
    }

    /// Return the underlying stream object.
    #[inline]
    pub(crate) fn abandon_into_inner<Inner: WriteExt>(
        internals: impl Utf8WriterInternals<Inner>,
    ) -> Inner {
        internals.into_inner()
    }

    #[inline]
    pub(crate) fn close<Inner: WriteExt>(
        internals: &mut impl Utf8WriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.inner_mut().close()
    }

    #[inline]
    pub(crate) fn abandon<Inner: WriteExt>(internals: &mut impl Utf8WriterInternals<Inner>) {
        internals.inner_mut().abandon()
    }

    #[inline]
    pub(crate) fn suggested_buffer_size<Inner: WriteExt>(
        internals: &impl Utf8WriterInternals<Inner>,
    ) -> usize {
        internals.inner().suggested_buffer_size()
    }

    #[inline]
    pub(crate) fn write_str<Inner: WriteExt>(
        internals: &mut impl Utf8WriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        default_write_str(internals.inner_mut(), s)
    }

    pub(crate) fn write<Inner: WriteExt>(
        internals: &mut impl Utf8WriterInternals<Inner>,
        buf: &[u8],
    ) -> io::Result<usize> {
        match str::from_utf8(buf) {
            Ok(s) => Self::write_str(internals, s).map(|_| buf.len()),
            Err(error) if error.valid_up_to() != 0 => internals
                .inner_mut()
                .write_all(&buf[..error.valid_up_to()])
                .map(|_| error.valid_up_to()),
            Err(error) => {
                internals.inner_mut().abandon();
                Err(io::Error::new(io::ErrorKind::Other, error))
            }
        }
    }

    #[inline]
    pub(crate) fn flush<Inner: WriteExt>(
        internals: &mut impl Utf8WriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.inner_mut().flush()
    }
}
