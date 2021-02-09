use crate::RestrictedStr;
use std::io;
use basic_text::WriteText;

/// Add a convenience method for reading into `RestrictedStr`.
pub trait WriteRestricted: WriteText {
    /// Like `WriteText::write_text` but writes from a `RestrictedStr`.
    #[inline]
    fn write_restricted(&mut self, buf: &RestrictedStr) -> io::Result<()> {
        default_write_restricted(self, buf)
    }
}

// There is no `WriteRestrictedLayered` because none of the `WriteLayered` functions
// need to be augmented to handle strings.

/// Default implementation of `WriteRestricted::read_restricted_str`.
#[inline]
pub fn default_write_restricted<Inner: WriteText + ?Sized>(
    inner: &mut Inner,
    buf: &RestrictedStr,
) -> io::Result<()> {
    inner.write_text(buf.as_ref())
}
