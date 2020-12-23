use crate::TextStr;
use io_ext::{ReadExt, Status};
use std::io;

/// Add a convenience method for reading into `TextStr`.
pub trait ReadText: ReadExt {
    /// Like `read_with_status` but produces the result in a `TextStr`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    fn read_text(&mut self, buf: &mut TextStr) -> io::Result<(usize, Status)>;

    /// Like `read_exact` but produces the result in a `TextStr`.
    fn read_exact_text(&mut self, buf: &mut TextStr) -> io::Result<()> {
        default_read_exact_text(self, buf)
    }
}

/// Default implementation of `ReadText::read_exact_text`.
pub fn default_read_exact_text<Inner: ReadText + ?Sized>(
    inner: &mut Inner,
    mut buf: &mut TextStr,
) -> io::Result<()> {
    while !buf.is_empty() {
        match inner.read_text(buf) {
            Ok((size, status)) => {
                let t = buf;
                buf = t.split_at_mut(size).1;
                if status.is_end() {
                    break;
                }
            }
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
