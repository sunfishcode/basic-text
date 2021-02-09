//! Output for `RestrictedWriter` and the writer half of `RestrictedDuplexer`.

use crate::{
    RestrictedDuplexer, RestrictedStr, RestrictedWriter,
};
#[cfg(can_vector)]
use layered_io::default_is_write_vectored;
#[cfg(write_all_vectored)]
use layered_io::default_write_all_vectored;
use layered_io::{default_write_vectored, HalfDuplexLayered, WriteLayered};
use std::{
    io::{self, Write},
    str,
    mem::replace,
};
use utf8_io::{ReadStrLayered, WriteStr};
use basic_text::TextStr;

pub(crate) trait RestrictedWriterInternals<Inner: WriteStr + WriteLayered>: Write {
    fn impl_(&mut self) -> &mut RestrictedOutput;
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
    fn into_inner(self) -> Inner;
    fn write_str(&mut self, s: &str) -> io::Result<()>;
}

impl<Inner: WriteStr + WriteLayered> RestrictedWriterInternals<Inner> for RestrictedWriter<Inner> {
    fn impl_(&mut self) -> &mut RestrictedOutput {
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

    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.inner.write_str(s)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr + WriteLayered> RestrictedWriterInternals<Inner>
    for RestrictedDuplexer<Inner>
{
    fn impl_(&mut self) -> &mut RestrictedOutput {
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

    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.inner.write_str(s)
    }
}

pub(crate) struct RestrictedOutput {
    /// Temporary staging buffer.
    buffer: String,
}

impl RestrictedOutput {
    /// Construct a new instance of `RestrictedOutput`.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    pub(crate) fn close_into_inner<Inner: WriteStr + WriteLayered>(
        mut internals: impl RestrictedWriterInternals<Inner>,
    ) -> io::Result<Inner> {
        Ok(internals.into_inner())
    }

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    pub(crate) fn abandon_into_inner<Inner: WriteStr + WriteLayered>(
        mut internals: impl RestrictedWriterInternals<Inner>,
    ) -> Inner {
        Self::reset_state(&mut internals);

        internals.into_inner()
    }

    fn normal_write_str<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn normal_write_text<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
        s: &TextStr,
    ) -> io::Result<()> {
        let impl_ = internals.impl_();
        let s = s.as_ref(); // TODO: Avoid doing this.

        impl_.buffer.push_str(s);

        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn normal_write_restricted<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
        s: &RestrictedStr,
    ) -> io::Result<()> {
        let impl_ = internals.impl_();
        let s = s.as_ref(); // TODO: Avoid doing this.

        impl_.buffer.push_text(s);

        // Write to the underlying stream.
        Self::write_buffer(internals)
    }

    fn write_buffer<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
    ) -> io::Result<()> {
        let buffer = replace(&mut internals.impl_().buffer, String::new());
        match internals.write_str(&buffer) {
            Ok(()) => (),
            Err(e) => {
                Self::reset_state(internals);
                return Err(e);
            }
        }
        internals.impl_().buffer = buffer;

        // Reset the temporary buffer.
        internals.impl_().buffer.clear();

        Ok(())
    }

    pub(crate) fn close<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.inner_mut().close()
    }

    pub(crate) fn abandon<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
    ) {
        internals.inner_mut().abandon();

        Self::reset_state(internals);
    }

    pub(crate) fn suggested_buffer_size<Inner: WriteStr + WriteLayered>(
        internals: &impl RestrictedWriterInternals<Inner>,
    ) -> usize {
        internals.inner().suggested_buffer_size()
    }

    pub(crate) fn write_text<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
        s: &TextStr,
    ) -> io::Result<()> {
        Self::normal_write_text(internals, s)
    }

    pub(crate) fn write_str<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
        s: &str,
    ) -> io::Result<()> {
        Self::normal_write_str(internals, s)
    }

    pub(crate) fn write<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
        buf: &[u8],
    ) -> io::Result<usize> {
        match str::from_utf8(buf) {
            Ok(s) => Self::write_str(internals, s).map(|()| buf.len()),
            // Safety: See the example code here:
            // https://doc.rust-lang.org/std/str/struct.Utf8Error.html#examples
            Err(error) if error.valid_up_to() != 0 => Self::write_str(internals, unsafe {
                str::from_utf8_unchecked(&buf[..error.valid_up_to()])
            })
            .map(|()| error.valid_up_to()),
            Err(error) => {
                Self::reset_state(internals);
                Err(io::Error::new(io::ErrorKind::Other, error))
            }
        }
    }

    #[inline]
    pub(crate) fn flush<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
    ) -> io::Result<()> {
        internals.inner_mut().flush()
    }

    #[inline]
    pub(crate) fn write_vectored<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
        bufs: &[io::IoSlice<'_>],
    ) -> io::Result<usize> {
        default_write_vectored(internals, bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    pub(crate) fn is_write_vectored<Inner: WriteStr + WriteLayered>(
        internals: &impl RestrictedWriterInternals<Inner>,
    ) -> bool {
        default_is_write_vectored(internals)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    pub(crate) fn write_all_vectored<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
        bufs: &mut [io::IoSlice<'_>],
    ) -> io::Result<()> {
        default_write_all_vectored(internals, bufs)
    }

    fn reset_state<Inner: WriteStr + WriteLayered>(
        internals: &mut impl RestrictedWriterInternals<Inner>,
    ) {
    }
}
