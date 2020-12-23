use io_ext::WriteExt;
use std::io;

/// Add methods for for finishing with a `WriteExt` and returning its
/// inner `WriteExt`.
pub trait WriteWrapper<Inner>: WriteExt {
    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    fn close_into_inner(self) -> io::Result<Inner>;

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    fn abandon_into_inner(self) -> Inner;
}
