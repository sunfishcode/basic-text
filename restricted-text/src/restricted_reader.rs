use crate::{restricted_input::RestrictedInput, ReadRestricted, ReadRestrictedLayered, RestrictedStr};
use layered_io::{default_read_to_end, Bufferable, ReadLayered, Status};
#[cfg(not(windows))]
use unsafe_io::os::posish::{AsRawFd, RawFd};
use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Read},
    str,
};
#[cfg(feature = "terminal-io")]
use terminal_io::{ReadTerminal, Terminal};
#[cfg(windows)]
use unsafe_io::os::windows::{AsRawHandleOrSocket, RawHandleOrSocket};
#[cfg(test)]
use utf8_io::Utf8Reader;
use utf8_io::{ReadStr, ReadStrLayered};
#[cfg(test)]
use basic_text::{TextReader};
use basic_text::{TextStr, ReadTextLayered, ReadText};
use unsafe_io::OwnsRaw;

/// A [`Read`] implementation which translates from an input `Read`
/// implementation producing an arbitrary byte sequence into a valid Restricted Text
/// stream.
pub struct RestrictedReader<Inner: ReadStrLayered> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// Text translation state.
    pub(crate) input: RestrictedInput,
}

impl<Inner: ReadStrLayered> RestrictedReader<Inner> {
    /// Construct a new instance of `RestrictedReader` wrapping `inner`.
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            input: RestrictedInput::new(),
        }
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: ReadStrLayered + ReadTerminal> Terminal for RestrictedReader<Inner> {}

#[cfg(feature = "terminal-io")]
impl<Inner: ReadStrLayered + ReadTerminal> ReadTerminal for RestrictedReader<Inner> {
    #[inline]
    fn is_line_by_line(&self) -> bool {
        self.inner.is_line_by_line()
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        self.inner.is_input_terminal()
    }
}

impl<Inner: ReadStrLayered> ReadLayered for RestrictedReader<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        RestrictedInput::read_with_status(self, buf)
    }

    #[inline]
    fn minimum_buffer_size(&self) -> usize {
        RestrictedInput::minimum_buffer_size(self)
    }
}

impl<Inner: ReadStrLayered> Bufferable for RestrictedReader<Inner> {
    #[inline]
    fn abandon(&mut self) {
        RestrictedInput::abandon(self)
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        RestrictedInput::suggested_buffer_size(self)
    }
}

impl<Inner: ReadStrLayered> ReadStr for RestrictedReader<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<usize> {
        RestrictedInput::read_str(self, buf)
    }

    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        RestrictedInput::read_exact_str(self, buf)
    }
}

impl<Inner: ReadStrLayered> ReadStrLayered for RestrictedReader<Inner> {
    #[inline]
    fn read_str_with_status(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        RestrictedInput::read_str_with_status(self, buf)
    }

    #[inline]
    fn read_exact_str_using_status(&mut self, buf: &mut str) -> io::Result<Status> {
        RestrictedInput::read_exact_str_using_status(self, buf)
    }
}

impl<Inner: ReadTextLayered> ReadText for RestrictedReader<Inner> {
    #[inline]
    fn read_text(&mut self, buf: &mut TextStr) -> io::Result<usize> {
        RestrictedInput::read_text(self, buf)
    }

    #[inline]
    fn read_exact_text(&mut self, buf: &mut TextStr) -> io::Result<()> {
        RestrictedInput::read_exact_text(self, buf)
    }
}

impl<Inner: ReadTextLayered> ReadRestricted for RestrictedReader<Inner> {
    #[inline]
    fn read_restricted(&mut self, buf: &mut RestrictedStr) -> io::Result<usize> {
        RestrictedInput::read_restricted(self, buf)
    }

    #[inline]
    fn read_exact_restricted(&mut self, buf: &mut RestrictedStr) -> io::Result<()> {
        RestrictedInput::read_exact_restricted(self, buf)
    }
}

impl<Inner: ReadStrLayered> ReadTextLayered for RestrictedReader<Inner> {
    #[inline]
    fn read_text_with_status(&mut self, buf: &mut TextStr) -> io::Result<(usize, Status)> {
        RestrictedInput::read_text_with_status(self, buf)
    }

    #[inline]
    fn read_exact_text_using_status(&mut self, buf: &mut TextStr) -> io::Result<Status> {
        RestrictedInput::read_exact_text_using_status(self, buf)
    }
}

impl<Inner: ReadRestrictedLayered> ReadRestrictedLayered for RestrictedReader<Inner> {
    #[inline]
    fn read_restricted_with_status(&mut self, buf: &mut RestrictedStr) -> io::Result<(usize, Status)> {
        RestrictedInput::read_restricted_with_status(self, buf)
    }

    #[inline]
    fn read_exact_restricted_using_status(&mut self, buf: &mut RestrictedStr) -> io::Result<Status> {
        RestrictedInput::read_exact_restricted_using_status(self, buf)
    }
}

impl<Inner: ReadStrLayered> Read for RestrictedReader<Inner> {
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

#[cfg(not(windows))]
impl<Inner: ReadStrLayered + AsRawFd> AsRawFd for RestrictedReader<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: ReadStrLayered + AsRawHandleOrSocket> AsRawHandleOrSocket for RestrictedReader<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

// Safety: `RestrictedReader` implements `OwnsRaw` if `Inner` does.
unsafe impl<Inner: ReadStrLayered + OwnsRaw> OwnsRaw for RestrictedReader<Inner> {}

impl<Inner: ReadStrLayered + Debug> Debug for RestrictedReader<Inner> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("RestrictedReader");
        b.field("inner", &self.inner);
        b.finish()
    }
}

#[cfg(test)]
fn translate_via_reader(bytes: &[u8]) -> String {
    let mut reader = RestrictedReader::new(TextReader::new(bytes));
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[cfg(test)]
fn translate_via_slice_reader(bytes: &[u8]) -> String {
    let mut reader = RestrictedReader::new(TextReader::from_utf8(Utf8Reader::new(layered_io::SliceReader::new(bytes))));
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[cfg(test)]
fn translate_with_small_buffer(bytes: &[u8]) -> String {
    let mut reader = RestrictedReader::new(TextReader::new(bytes));
    let mut v = Vec::new();
    let mut buf = [0; basic_text::NORMALIZATION_BUFFER_SIZE];
    loop {
        let size = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(size) => size,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => 0,
            Err(err) => Err(err).unwrap(),
        };
        v.extend_from_slice(&buf[..size]);
    }
    String::from_utf8(v).unwrap()
}

#[cfg(test)]
fn translate_with_small_buffer_layered(bytes: &[u8]) -> String {
    let mut reader = RestrictedReader::new(TextReader::from_utf8(Utf8Reader::new(layered_io::SliceReader::new(bytes))));
    let mut v = Vec::new();
    let mut buf = [0; basic_text::NORMALIZATION_BUFFER_SIZE];
    loop {
        let (size, status) = reader.read_with_status(&mut buf).unwrap();
        v.extend_from_slice(&buf[..size]);
        if status.is_end() {
            break;
        }
    }
    String::from_utf8(v).unwrap()
}

#[cfg(test)]
fn test(bytes: &[u8], s: &str) {
    assert_eq!(translate_via_reader(bytes), s);
    assert_eq!(translate_via_slice_reader(bytes), s);
    assert_eq!(translate_with_small_buffer(bytes), s);
    assert_eq!(translate_with_small_buffer_layered(bytes), s);
}

#[test]
fn test_empty_string() {
    test(b"", "");
}
