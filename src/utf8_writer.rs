use crate::utf8_output::Utf8Output;
use io_ext::{Status, WriteExt};
use std::{io, str};

/// Add methods for for finishing with a `WriteExt` and returning its
/// inner `WriteExt`.
pub trait WriteWrapper<Inner: WriteExt>: WriteExt {
    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    fn close_into_inner(self) -> io::Result<Inner>;

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    fn abandon_into_inner(self) -> Inner;
}

/// A `WriteExt` implementation which translates into an output `WriteExt`
/// producing a valid UTF-8 sequence from an arbitrary byte sequence from an
/// arbitrary byte sequence. Attempts to write invalid encodings are reported
/// as errors.
///
/// `write` is not guaranteed to perform a single operation, because short
/// writes could produce invalid UTF-8, so `write` will retry as needed.
pub struct Utf8Writer<Inner: WriteExt> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    pub(crate) output: Utf8Output,
}

impl<Inner: WriteExt> Utf8Writer<Inner> {
    /// Construct a new instance of `Utf8Writer` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            output: Utf8Output::new(),
        }
    }
}

impl<Inner: WriteExt> WriteExt for Utf8Writer<Inner> {
    #[inline]
    fn flush_with_status(&mut self, status: Status) -> io::Result<()> {
        Utf8Output::flush_with_status(self, status)
    }

    #[inline]
    fn abandon(&mut self) {
        Utf8Output::abandon(self)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        Utf8Output::write_str(self, s)
    }
}

impl<Inner: WriteExt> WriteWrapper<Inner> for Utf8Writer<Inner> {
    #[inline]
    fn close_into_inner(self) -> io::Result<Inner> {
        Utf8Output::close_into_inner(self)
    }

    #[inline]
    fn abandon_into_inner(self) -> Inner {
        Utf8Output::abandon_into_inner(self)
    }
}

impl<Inner: WriteExt> io::Write for Utf8Writer<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Utf8Output::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Utf8Output::flush(self)
    }
}
