use crate::TextStr;
use io_ext::WriteExt;
use std::io;

/// Add a convenience method for reading into `TextStr`.
pub trait WriteText: WriteExt {
    /// Like `WriteExt::write_str` but writes from a `TextStr`.
    #[inline]
    fn write_text(&mut self, buf: &TextStr) -> io::Result<()> {
        default_write_text(self, buf)
    }
}

/// Default implementation of `WriteText::read_exact_str`.
#[inline]
pub fn default_write_text<Inner: WriteExt + ?Sized>(
    inner: &mut Inner,
    buf: &TextStr,
) -> io::Result<()> {
    inner.write_str(buf.as_ref())
}
