use crate::utf8_output::Utf8Output;
use io_ext::{Bufferable, WriteExt};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, RawFd};
use std::{
    fmt::{self, Arguments},
    io::{self, Write},
    str,
};
#[cfg(feature = "terminal-support")]
use terminal_support::{Terminal, TerminalColorSupport, WriteTerminal};
#[cfg(windows)]
use unsafe_io::{AsRawHandleOrSocket, RawHandleOrSocket};

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

/// A `WriteExt` implementation which translates into an output `WriteExt`
/// producing a valid UTF-8 sequence from an arbitrary byte sequence from an
/// arbitrary byte sequence. Attempts to write invalid encodings are reported
/// as errors.
///
/// `write` is not guaranteed to perform a single operation, because short
/// writes could produce invalid UTF-8, so `write` will retry as needed.
pub struct Utf8Writer<Inner> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    pub(crate) output: Utf8Output,
}

impl<Inner: WriteExt> Utf8Writer<Inner> {
    /// Construct a new instance of `Utf8Writer` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            output: Utf8Output::new(),
        }
    }
}

#[cfg(feature = "terminal-support")]
impl<Inner: WriteExt + WriteTerminal> Terminal for Utf8Writer<Inner> {}

#[cfg(feature = "terminal-support")]
impl<Inner: WriteExt + WriteTerminal> WriteTerminal for Utf8Writer<Inner> {
    #[inline]
    fn color_support(&self) -> TerminalColorSupport {
        self.inner.color_support()
    }

    #[inline]
    fn color_preference(&self) -> bool {
        self.inner.color_preference()
    }

    #[inline]
    fn is_output_terminal(&self) -> bool {
        self.inner.is_output_terminal()
    }
}

impl<Inner: WriteExt> WriteExt for Utf8Writer<Inner> {
    #[inline]
    fn end(&mut self) -> io::Result<()> {
        Utf8Output::end(self)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        Utf8Output::write_str(self, s)
    }
}

impl<Inner: WriteExt> Bufferable for Utf8Writer<Inner> {
    #[inline]
    fn abandon(&mut self) {
        Utf8Output::abandon(self)
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        Utf8Output::suggested_buffer_size(self)
    }
}

impl<Inner: WriteExt> WriteWrapper<Inner> for Utf8Writer<Inner> {
    #[inline]
    fn close_into_inner(self) -> io::Result<Inner> {
        Utf8Output::close_into_inner(self)
    }

    #[inline]
    fn abandon_into_inner(self) -> Inner {
        Utf8Output::abandon_into_inner(self)
    }
}

impl<Inner: WriteExt> Write for Utf8Writer<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Utf8Output::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Utf8Output::flush(self)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        Utf8Output::write_fmt(self, fmt)
    }
}

#[cfg(not(windows))]
impl<Inner: WriteExt + AsRawFd> AsRawFd for Utf8Writer<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: WriteExt + AsRawHandleOrSocket> AsRawHandleOrSocket for Utf8Writer<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

impl<Inner: fmt::Debug> fmt::Debug for Utf8Writer<Inner> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("Utf8Writer");
        b.field("inner", &self.inner);
        b.finish()
    }
}
