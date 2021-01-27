use crate::{utf8_input::Utf8Input, utf8_output::Utf8Output, ReadStr, WriteStr};
use interact_trait::{Interact, InteractExt};
use io_ext::{Bufferable, ReadExt, Status, WriteExt};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, RawFd};
use std::{
    cmp::max,
    fmt,
    io::{self, Read, Write},
    str,
};
#[cfg(feature = "terminal-support")]
use terminal_support::{
    InteractTerminal, ReadTerminal, Terminal, TerminalColorSupport, WriteTerminal,
};
#[cfg(windows)]
use unsafe_io::{AsRawHandleOrSocket, RawHandleOrSocket};

/// The combination of `Utf8Reader` and `Utf8Writer`.
pub struct Utf8Interactor<Inner> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    pub(crate) input: Utf8Input,
    pub(crate) output: Utf8Output,
}

impl<Inner> Utf8Interactor<Inner> {
    /// Construct a new instance of `Utf8Interactor` wrapping `inner`.
    #[inline]
    pub const fn new(inner: Inner) -> Self {
        Self {
            inner,
            input: Utf8Input::new(),
            output: Utf8Output::new(),
        }
    }
}

impl<Inner: InteractExt> Utf8Interactor<Inner> {
    /// Flush and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn close_into_inner(self) -> io::Result<Inner> {
        Utf8Output::close_into_inner(self)
    }

    /// Discard and close the underlying stream and return the underlying
    /// stream object.
    #[inline]
    pub fn abandon_into_inner(self) -> Inner {
        Utf8Output::abandon_into_inner(self)
    }
}

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> Terminal for Utf8Interactor<Inner> {}

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> ReadTerminal for Utf8Interactor<Inner> {
    #[inline]
    fn is_line_by_line(&self) -> bool {
        self.inner.is_line_by_line()
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        self.inner.is_input_terminal()
    }
}

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> WriteTerminal for Utf8Interactor<Inner> {
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

#[cfg(feature = "terminal-support")]
impl<Inner: InteractExt + InteractTerminal> InteractTerminal for Utf8Interactor<Inner> {}

impl<Inner: ReadStr + InteractExt> ReadStr for Utf8Interactor<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        Utf8Input::read_str(self, buf)
    }

    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        Utf8Input::read_exact_str(self, buf)
    }
}

impl<Inner: InteractExt> ReadExt for Utf8Interactor<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        Utf8Input::read_with_status(self, buf)
    }

    #[inline]
    fn minimum_buffer_size(&self) -> usize {
        Utf8Input::minimum_buffer_size(self)
    }
}

impl<Inner: InteractExt> Bufferable for Utf8Interactor<Inner> {
    #[inline]
    fn abandon(&mut self) {
        Utf8Input::abandon(self);
        Utf8Output::abandon(self);
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        max(
            Utf8Input::suggested_buffer_size(self),
            Utf8Output::suggested_buffer_size(self),
        )
    }
}

impl<Inner: InteractExt> Read for Utf8Interactor<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Utf8Input::read(self, buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        Utf8Input::read_vectored(self, bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_read_vectored(&self) -> bool {
        Utf8Input::is_read_vectored(self)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        Utf8Input::read_to_end(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        Utf8Input::read_to_string(self, buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        Utf8Input::read_exact(self, buf)
    }
}

impl<Inner: InteractExt> WriteExt for Utf8Interactor<Inner> {
    #[inline]
    fn close(&mut self) -> io::Result<()> {
        Utf8Output::close(self)
    }
}

impl<Inner: InteractExt> WriteStr for Utf8Interactor<Inner> {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        Utf8Output::write_str(self, s)
    }
}

impl<Inner: InteractExt> Interact for Utf8Interactor<Inner> {}

impl<Inner: InteractExt> Write for Utf8Interactor<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Utf8Output::write(self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Utf8Output::flush(self)
    }
}

#[cfg(not(windows))]
impl<Inner: InteractExt + AsRawFd> AsRawFd for Utf8Interactor<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: InteractExt + AsRawHandleOrSocket> AsRawHandleOrSocket for Utf8Interactor<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

impl<Inner: fmt::Debug> fmt::Debug for Utf8Interactor<Inner> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("Utf8Interactor");
        b.field("inner", &self.inner);
        b.finish()
    }
}
