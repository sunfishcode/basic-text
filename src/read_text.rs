use crate::{default_read_to_text_string, TextString, TextSubstr};
use layered_io::Status;
use std::io;
use utf8_io::{ReadStr, ReadStrLayered};

/// Add a convenience method for reading Basic Text content.
pub trait ReadText: ReadStr {
    /// Like `read_str` but for reading Basic Text content. Note that the
    /// resulting data may not be a Basic Text string, as it may be eg. a
    /// portion of a stream that starts with a non-starter.
    fn read_text_substr(&mut self, buf: &mut TextSubstr) -> io::Result<usize>;

    /// Like `read_exact_str` but for reading Basic Text content. As with
    /// `read_text`, the resulting string may not be a Basic Text string.
    #[inline]
    fn read_exact_text_substr(&mut self, buf: &mut TextSubstr) -> io::Result<()> {
        default_read_exact_text_substr(self, buf)
    }

    /// Like `read_to_string`, but reads into a `TextString`.
    #[inline]
    fn read_to_text_string(&mut self, buf: &mut TextString) -> io::Result<usize> {
        default_read_to_text_string(self, buf)
    }
}

/// Extend the `ReadLayered` trait with `read_text_with_status`, a method for
/// reading text data.
pub trait ReadTextLayered: ReadStrLayered {
    /// Like `read_str_with_status` but for reading Basic Text data. Note that
    /// the resulting data may not be a Basic Text string, as it may be eg. a
    /// portion of a stream that starts with a non-starter.
    ///
    /// `buf` must be at least `NORMALIZATION_BUFFER_SIZE` bytes long, so that
    /// any valid normalized sequence can be read.
    fn read_text_substr_with_status(&mut self, buf: &mut TextSubstr)
        -> io::Result<(usize, Status)>;

    /// Like `read_exact_str_using_status` but for reading Basic Text content.
    /// As with `read_text`, the resulting string may not be a Basic Text
    /// string.
    ///
    /// Also, like `ReadText::read_exact_text`, but uses
    /// `read_text_with_status` to avoid performing an extra `read` at the
    /// end.
    #[inline]
    fn read_exact_text_substr_using_status(&mut self, buf: &mut TextSubstr) -> io::Result<Status> {
        default_read_exact_text_substr_using_status(self, buf)
    }
}

/// Default implementation of [`ReadText::read_exact_text_substr`].
pub fn default_read_exact_text_substr<Inner: ReadText + ?Sized>(
    inner: &mut Inner,
    mut buf: &mut TextSubstr,
) -> io::Result<()> {
    while !buf.is_empty() {
        match inner.read_text_substr(buf) {
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

/// Default implementation of
/// [`ReadTextLayered::read_exact_text_substr_using_status`].
pub fn default_read_exact_text_substr_using_status<Inner: ReadTextLayered + ?Sized>(
    inner: &mut Inner,
    mut buf: &mut TextSubstr,
) -> io::Result<Status> {
    let mut result_status = Status::active();

    while !buf.is_empty() {
        match inner.read_text_substr_with_status(buf) {
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
