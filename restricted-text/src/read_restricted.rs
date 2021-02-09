use crate::RestrictedStr;
use layered_io::Status;
use std::io;
use basic_text::{ReadText, ReadTextLayered};

/// Add a convenience method for reading into `RestrictedStr`.
pub trait ReadRestricted: ReadText {
    /// Like `read` but produces the result in a `RestrictedStr`. Be sure to check
    /// the `size` field of the return value to see how many bytes were
    /// written.
    fn read_restricted(&mut self, buf: &mut RestrictedStr) -> io::Result<usize>;

    /// Like `read_exact` but produces the result in a `RestrictedStr`.
    #[inline]
    fn read_exact_restricted(&mut self, buf: &mut RestrictedStr) -> io::Result<()> {
        default_read_exact_restricted(self, buf)
    }
}

/// Extend the `ReadTextLayered` trait with `read_restricted_with_status`, a method for
/// reading restricted data.
pub trait ReadRestrictedLayered: ReadTextLayered {
    /// Like `read_with_status` but produces the result in a `RestrictedStr`. Be sure to
    /// check the return value to see how many bytes were written.
    ///
    /// `buf` must be at least `NORMALIZATION_BUFFER_SIZE` bytes long, so that any
    /// valid normalized sequence can be read.
    fn read_restricted_with_status(&mut self, buf: &mut RestrictedStr) -> io::Result<(usize, Status)>;

    /// Like `read_exact` but produces the result in a `RestrictedStr`.
    ///
    /// Also, like `ReadRestricted::read_exact_restricted`, but uses `read_restricted_with_status`
    /// to avoid performing an extra `read` at the end.
    #[inline]
    fn read_exact_restricted_using_status(&mut self, buf: &mut RestrictedStr) -> io::Result<Status> {
        default_read_exact_restricted_using_status(self, buf)
    }
}

/// Default implementation of `ReadRestricted::read_exact_restricted`.
pub fn default_read_exact_restricted<Inner: ReadRestricted + ?Sized>(
    inner: &mut Inner,
    mut buf: &mut RestrictedStr,
) -> io::Result<()> {
    while !buf.is_empty() {
        match inner.read_restricted(buf) {
            Ok(0) => break,
            Ok(size) => buf = buf.split_at_mut(size).1,
            Err(e) => return Err(e),
        }
    }

    if buf.is_empty() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "failed to fill whole buffer",
        ))
    }
}

/// Default implementation of [`ReadRestrictedLayered::read_exact_restricted_using_status`].
pub fn default_read_exact_restricted_using_status<Inner: ReadRestrictedLayered + ?Sized>(
    inner: &mut Inner,
    mut buf: &mut RestrictedStr,
) -> io::Result<Status> {
    let mut result_status = Status::active();

    while !buf.is_empty() {
        match inner.read_restricted_with_status(buf) {
            Ok((size, status)) => {
                buf = buf.split_at_mut(size).1;
                if status.is_end() {
                    result_status = status;
                    break;
                }
            }
            Err(e) => return Err(e),
        }
    }

    if buf.is_empty() {
        Ok(result_status)
    } else {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "failed to fill whole buffer",
        ))
    }
}
