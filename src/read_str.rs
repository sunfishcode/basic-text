use layered_io::{ReadLayered, Status};
use std::io;

/// Add a convenience and optimizing method for reading into `str`.
pub trait ReadStr: ReadLayered {
    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    fn read_str(&mut self, buf: &mut str) -> io::Result<(usize, Status)>;

    /// Like `read_exact` but produces the result in a `str`.
    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        default_read_exact_str(self, buf)
    }
}

/// Default implementation of `ReadStr::read_exact_str`.
pub fn default_read_exact_str<Inner: ReadStr + ?Sized>(
    inner: &mut Inner,
    mut buf: &mut str,
) -> io::Result<()> {
    while !buf.is_empty() {
        match inner.read_str(buf) {
            Ok((size, status)) => {
                buf = buf.split_at_mut(size).1;
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
