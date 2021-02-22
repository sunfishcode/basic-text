use crate::{restricted_input::RestrictedInput, restricted_output::RestrictedOutput, ReadRestricted, RestrictedStr, WriteRestricted};
use duplex::{Duplex, HalfDuplex};
use layered_io::{
    default_read_to_end, Bufferable, HalfDuplexLayered, ReadLayered, Status, WriteLayered,
};
#[cfg(not(windows))]
use unsafe_io::os::posish::{AsRawFd, RawFd};
use std::{
    cmp::max,
    fmt,
    io::{self, Read, Write},
    str,
};
#[cfg(windows)]
use unsafe_io::os::windows::{AsRawHandleOrSocket, RawHandleOrSocket};
use unsafe_io::OwnsRaw;
use utf8_io::{ReadStr, ReadStrLayered, WriteStr};
use basic_text::{ReadText, ReadTextLayered, WriteText, TextStr};

/// A [`HalfDuplex`] implementation which translates from an input `HalfDuplex`
/// implementation producing an arbitrary byte sequence into a valid Restricted Text
/// stream.
pub struct RestrictedDuplexer<Inner: HalfDuplex + ReadStr + WriteStr> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// Text translation state.
    pub(crate) input: RestrictedInput,
    pub(crate) output: RestrictedOutput,
}

impl<Inner: HalfDuplex + ReadStr + ReadLayered + ReadStrLayered + WriteStr + WriteLayered>
    RestrictedDuplexer<Inner>
{
    /// Construct a new instance of `RestrictedDuplexer` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            input: RestrictedInput::new(),
            output: RestrictedOutput::new(),
        }
    }

    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn close_into_inner(self) -> io::Result<Inner> {
        RestrictedOutput::close_into_inner(self)
    }

    /// Return the underlying stream object.
    #[inline]
    pub fn abandon_into_inner(self) -> Inner {
        RestrictedOutput::abandon_into_inner(self)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> ReadLayered for RestrictedDuplexer<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        RestrictedInput::read_with_status(self, buf)
    }

    #[inline]
    fn minimum_buffer_size(&self) -> usize {
        RestrictedInput::minimum_buffer_size(self)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> Bufferable for RestrictedDuplexer<Inner> {
    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        max(
            RestrictedInput::suggested_buffer_size(self),
            RestrictedOutput::suggested_buffer_size(self),
        )
    }

    #[inline]
    fn abandon(&mut self) {
        RestrictedInput::abandon(self);
        RestrictedOutput::abandon(self);
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> ReadStr for RestrictedDuplexer<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<usize> {
        RestrictedInput::read_str(self, buf)
    }

    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        RestrictedInput::read_exact_str(self, buf)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> ReadStrLayered for RestrictedDuplexer<Inner> {
    #[inline]
    fn read_str_with_status(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        RestrictedInput::read_str_with_status(self, buf)
    }

    #[inline]
    fn read_exact_str_using_status(&mut self, buf: &mut str) -> io::Result<Status> {
        RestrictedInput::read_exact_str_using_status(self, buf)
    }
}

impl<Inner: HalfDuplexLayered + ReadTextLayered + WriteText> ReadText for RestrictedDuplexer<Inner> {
    #[inline]
    fn read_text(&mut self, buf: &mut TextStr) -> io::Result<usize> {
        RestrictedInput::read_text(self, buf)
    }

    #[inline]
    fn read_exact_text(&mut self, buf: &mut TextStr) -> io::Result<()> {
        RestrictedInput::read_exact_text(self, buf)
    }
}

impl<Inner: HalfDuplexLayered + ReadTextLayered + WriteText> ReadRestricted for RestrictedDuplexer<Inner> {
    #[inline]
    fn read_restricted(&mut self, buf: &mut RestrictedStr) -> io::Result<usize> {
        RestrictedInput::read_restricted(self, buf)
    }

    #[inline]
    fn read_exact_restricted(&mut self, buf: &mut RestrictedStr) -> io::Result<()> {
        RestrictedInput::read_exact_restricted(self, buf)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> Read for RestrictedDuplexer<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        RestrictedInput::read(self, buf)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        default_read_to_end(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        RestrictedInput::read_to_string(self, buf)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> WriteLayered for RestrictedDuplexer<Inner> {
    #[inline]
    fn close(&mut self) -> io::Result<()> {
        RestrictedOutput::close(self)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> WriteStr for RestrictedDuplexer<Inner> {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        RestrictedOutput::write_str(self, s)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> WriteText for RestrictedDuplexer<Inner> {
    #[inline]
    fn write_text(&mut self, s: &TextStr) -> io::Result<()> {
        RestrictedOutput::write_text(self, s)
    }
}

impl<Inner: HalfDuplexLayered + ReadTextLayered + WriteText> WriteRestricted for RestrictedDuplexer<Inner> {
    #[inline]
    fn write_restricted(&mut self, s: &RestrictedStr) -> io::Result<()> {
        RestrictedOutput::write_restricted(self, s)
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> Duplex for RestrictedDuplexer<Inner> {}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr> Write for RestrictedDuplexer<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        RestrictedOutput::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        RestrictedOutput::flush(self)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_write_vectored(&self) -> bool {
        RestrictedOutput::is_write_vectored(self)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        RestrictedOutput::write_vectored(self, bufs)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [io::IoSlice<'_>]) -> io::Result<()> {
        RestrictedOutput::write_all_vectored(self, bufs)
    }
}

#[cfg(not(windows))]
impl<Inner: HalfDuplexLayered + ReadStr + WriteStr + AsRawFd> AsRawFd for RestrictedDuplexer<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: HalfDuplexLayered + ReadStr + WriteStr + AsRawHandleOrSocket> AsRawHandleOrSocket
    for RestrictedDuplexer<Inner>
{
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

// Safety: `RestrictedDuplexer` implements `OwnsRaw` if `Inner` does.
unsafe impl<Inner: OwnsRaw> OwnsRaw for RestrictedDuplexer<Inner> {}

impl<Inner: HalfDuplexLayered + ReadStr + WriteStr + fmt::Debug> fmt::Debug
    for RestrictedDuplexer<Inner>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("RestrictedDuplexer");
        b.field("inner", &self.inner);
        b.finish()
    }
}
