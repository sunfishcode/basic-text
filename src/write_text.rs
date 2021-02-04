use crate::TextStr;
use std::io;
use utf8_io::WriteStr;

/// Add a convenience method for reading into `TextStr`.
pub trait WriteText: WriteStr {
    /// Like `WriteStr::write_str` but writes from a `TextStr`.
    #[inline]
    fn write_text(&mut self, buf: &TextStr) -> io::Result<()> {
        default_write_text(self, buf)
    }
}

// There is no `WriteTextLayered` because none of the `WriteLayered` functions
// need to be augmented to handle strings.

/// Default implementation of `WriteText::read_exact_str`.
#[inline]
pub fn default_write_text<Inner: WriteStr + ?Sized>(
    inner: &mut Inner,
    buf: &TextStr,
) -> io::Result<()> {
    inner.write_str(buf.as_ref())
}
