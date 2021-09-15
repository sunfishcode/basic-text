use crate::text_input::TextInput;
use crate::{ReadText, ReadTextLayered, TextSubstr};
use layered_io::{default_read_to_end, Bufferable, LayeredReader, ReadLayered, Status};
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read};
use std::str;
#[cfg(feature = "terminal-io")]
use terminal_io::{ReadTerminal, Terminal};
#[cfg(windows)]
use unsafe_io::os::windows::{
    AsHandleOrSocket, AsRawHandleOrSocket, BorrowedHandleOrSocket, RawHandleOrSocket,
};
use utf8_io::{ReadStr, ReadStrLayered, Utf8Reader};
#[cfg(not(windows))]
use {
    io_lifetimes::{AsFd, BorrowedFd},
    unsafe_io::os::rsix::{AsRawFd, RawFd},
};

/// A [`Read`] implementation which translates from an input `Read`
/// implementation producing an arbitrary byte sequence into a valid Basic Text
/// stream.
///
/// # Examples
///
/// ```rust
/// use basic_text::TextReader;
///
/// let input = TextReader::new(std::io::stdin());
///
/// // read from `input`
/// ```
pub struct TextReader<Inner: ReadStrLayered> {
    /// The wrapped byte stream.
    pub(crate) inner: Inner,

    /// Text translation state.
    pub(crate) input: TextInput,
}

impl<Inner: Read> TextReader<Utf8Reader<LayeredReader<Inner>>> {
    /// Construct a new instance of `TextReader` wrapping `inner`, which can
    /// be anything that implements [`Read`].
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self::from_utf8(Utf8Reader::new(LayeredReader::new(inner)))
    }

    /// Like `new`, but replaces U+85 (NEL) with U+A instead of U+20.
    #[inline]
    pub fn with_nel_compatibility(inner: Inner) -> io::Result<Self> {
        Self::from_utf8_with_nel_compatibility(Utf8Reader::new(LayeredReader::new(inner)))
    }

    /// Like `new`, but replaces U+2028 (LS) and U+2029 (PS) with U+A instead
    /// of U+20.
    #[inline]
    pub fn with_lsps_compatibility(inner: Inner) -> io::Result<Self> {
        Self::from_utf8_with_lsps_compatibility(Utf8Reader::new(LayeredReader::new(inner)))
    }
}

impl<Inner: ReadStrLayered> TextReader<Inner> {
    /// Construct a new instance of `TextReader` wrapping `inner`, which
    /// can be anything that implements `ReadStrLayered`, such as a
    /// [`Utf8Reader`].
    #[inline]
    pub fn from_utf8(inner: Inner) -> Self {
        Self {
            inner,
            input: TextInput::new(),
        }
    }

    /// Like `from_utf8`, but replaces U+85 (NEL) with U+A instead of U+20.
    #[inline]
    pub fn from_utf8_with_nel_compatibility(inner: Inner) -> io::Result<Self> {
        let input = TextInput::with_nel_compatibility();
        Ok(Self { inner, input })
    }

    /// Like `from_utf8`, but replaces U+2028 (LS) and U+2029 (PS) with U+A
    /// instead of U+20.
    #[inline]
    pub fn from_utf8_with_lsps_compatibility(inner: Inner) -> io::Result<Self> {
        let input = TextInput::with_lsps_compatibility();
        Ok(Self { inner, input })
    }
}

#[cfg(feature = "terminal-io")]
impl<Inner: ReadStrLayered + ReadTerminal> Terminal for TextReader<Inner> {}

#[cfg(feature = "terminal-io")]
impl<Inner: ReadStrLayered + ReadTerminal> ReadTerminal for TextReader<Inner> {
    #[inline]
    fn is_line_by_line(&self) -> bool {
        self.inner.is_line_by_line()
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        self.inner.is_input_terminal()
    }
}

impl<Inner: ReadStrLayered> ReadLayered for TextReader<Inner> {
    #[inline]
    fn read_with_status(&mut self, buf: &mut [u8]) -> io::Result<(usize, Status)> {
        TextInput::read_with_status(self, buf)
    }

    #[inline]
    fn minimum_buffer_size(&self) -> usize {
        TextInput::minimum_buffer_size(self)
    }
}

impl<Inner: ReadStrLayered> Bufferable for TextReader<Inner> {
    #[inline]
    fn abandon(&mut self) {
        TextInput::abandon(self);
    }

    #[inline]
    fn suggested_buffer_size(&self) -> usize {
        TextInput::suggested_buffer_size(self)
    }
}

impl<Inner: ReadStrLayered> ReadStr for TextReader<Inner> {
    #[inline]
    fn read_str(&mut self, buf: &mut str) -> io::Result<usize> {
        TextInput::read_str(self, buf)
    }

    #[inline]
    fn read_exact_str(&mut self, buf: &mut str) -> io::Result<()> {
        TextInput::read_exact_str(self, buf)
    }
}

impl<Inner: ReadStrLayered> ReadStrLayered for TextReader<Inner> {
    #[inline]
    fn read_str_with_status(&mut self, buf: &mut str) -> io::Result<(usize, Status)> {
        TextInput::read_str_with_status(self, buf)
    }

    #[inline]
    fn read_exact_str_using_status(&mut self, buf: &mut str) -> io::Result<Status> {
        TextInput::read_exact_str_using_status(self, buf)
    }
}

impl<Inner: ReadStrLayered> ReadText for TextReader<Inner> {
    #[inline]
    fn read_text_substr(&mut self, buf: &mut TextSubstr) -> io::Result<usize> {
        TextInput::read_text_substr(self, buf)
    }

    #[inline]
    fn read_exact_text_substr(&mut self, buf: &mut TextSubstr) -> io::Result<()> {
        TextInput::read_exact_text_substr(self, buf)
    }
}

impl<Inner: ReadStrLayered> ReadTextLayered for TextReader<Inner> {
    #[inline]
    fn read_text_substr_with_status(
        &mut self,
        buf: &mut TextSubstr,
    ) -> io::Result<(usize, Status)> {
        TextInput::read_text_substr_with_status(self, buf)
    }

    #[inline]
    fn read_exact_text_substr_using_status(&mut self, buf: &mut TextSubstr) -> io::Result<Status> {
        TextInput::read_exact_text_substr_using_status(self, buf)
    }
}

impl<Inner: ReadStrLayered> Read for TextReader<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        TextInput::read(self, buf)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        default_read_to_end(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        TextInput::read_to_string(self, buf)
    }
}

#[cfg(not(windows))]
impl<Inner: ReadStrLayered + AsRawFd> AsRawFd for TextReader<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(not(windows))]
impl<Inner: ReadStrLayered + AsFd> AsFd for TextReader<Inner> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

#[cfg(windows)]
impl<Inner: ReadStrLayered + AsRawHandleOrSocket> AsRawHandleOrSocket for TextReader<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl<Inner: ReadStrLayered + AsHandleOrSocket> AsHandleOrSocket for TextReader<Inner> {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.inner.as_handle_or_socket()
    }
}

impl<Inner: ReadStrLayered + Debug> Debug for TextReader<Inner> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut b = f.debug_struct("TextReader");
        b.field("inner", &self.inner);
        b.finish()
    }
}

#[cfg(test)]
fn translate_via_reader(bytes: &[u8]) -> String {
    let mut reader = TextReader::new(bytes);
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[cfg(test)]
fn translate_via_slice_reader(bytes: &[u8]) -> String {
    let mut reader = TextReader::from_utf8(Utf8Reader::new(layered_io::SliceReader::new(bytes)));
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[cfg(test)]
fn translate_with_small_buffer(bytes: &[u8]) -> String {
    let mut reader = TextReader::new(bytes);
    let mut v = Vec::new();
    let mut buf = [0; basic_text_internals::unicode::NORMALIZATION_BUFFER_SIZE];
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
    let mut reader = TextReader::from_utf8(Utf8Reader::new(layered_io::SliceReader::new(bytes)));
    let mut v = Vec::new();
    let mut buf = [0; basic_text_internals::unicode::NORMALIZATION_BUFFER_SIZE];
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

#[test]
fn test_nl() {
    test(b"\n", "\n");
    test(b"\nhello\nworld\n", "\nhello\nworld\n");
}

#[test]
fn test_bom() {
    test("\u{feff}".as_bytes(), "");
    test(
        "\u{feff}hello\u{feff}world\u{feff}".as_bytes(),
        "hello\u{2060}world\u{2060}\n",
    );
}

#[test]
fn test_crlf() {
    test(b"\r\n", "\n");
    test(b"\r\nhello\r\nworld\r\n", "\nhello\nworld\n");
}

#[test]
fn test_cr_plain() {
    test(b"\r", "\n");
    test(b"\rhello\rworld\r", "\nhello\nworld\n");
}

#[test]
fn test_ff() {
    test(b"\x0c", " \n");
    test(b"\x0chello\x0cworld\x0c", " hello world \n");
}

#[test]
fn test_del() {
    test(b"\x7f", "\u{fffd}\n");
    test(
        b"\x7fhello\x7fworld\x7f",
        "\u{fffd}hello\u{fffd}world\u{fffd}\n",
    );
}

#[test]
fn test_non_text_c0() {
    test(
        b"\x00\x01\x02\x03\x04\x05\x06\x07",
        "\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\n",
    );
    test(b"\x08\x0b\x0e\x0f", "\u{fffd}\u{fffd}\u{fffd}\u{fffd}\n");
    test(
        b"\x10\x11\x12\x13\x14\x15\x16\x17",
        "\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\n",
    );
    test(
        b"\x18\x19\x1a\x1c\x1d\x1e\x1f",
        "\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\n",
    );
}

#[test]
fn test_c1() {
    test(
        "\u{80}\u{81}\u{82}\u{83}\u{84}\u{85}\u{86}\u{87}".as_bytes(),
        "\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd} \u{fffd}\u{fffd}\n",
    );
    test(
        "\u{88}\u{89}\u{8a}\u{8b}\u{8c}\u{8d}\u{8e}\u{8f}".as_bytes(),
        "\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\n",
    );
    test(
        "\u{90}\u{91}\u{92}\u{93}\u{94}\u{95}\u{96}\u{97}".as_bytes(),
        "\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\n",
    );
    test(
        "\u{98}\u{99}\u{9a}\u{9b}\u{9c}\u{9d}\u{9e}\u{9f}".as_bytes(),
        "\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\u{fffd}\n",
    );
}

#[test]
fn test_nfc() {
    test("\u{212b}".as_bytes(), "\u{c5}\n");
    test("\u{c5}".as_bytes(), "\u{c5}\n");
    test("\u{41}\u{30a}".as_bytes(), "\u{c5}\n");
}

#[test]
fn test_leading_nonstarters() {
    test("\u{30a}".as_bytes(), "\u{34f}\u{30a}\n");
}

#[test]
fn test_esc() {
    test(b"\x1b", "\n");
    test(b"\x1b@", "\n");
    test(b"\x1b@hello\x1b@world\x1b@", "helloworld\n");
}

#[test]
fn test_csi() {
    test(b"\x1b[", "\n");
    test(b"\x1b[@hello\x1b[@world\x1b[@", "helloworld\n");
    test(b"\x1b[+@hello\x1b[+@world\x1b[+@", "helloworld\n");
}

#[test]
fn test_osc() {
    test(b"\x1b]", "\n");
    test(b"\x1b]\x07hello\x1b]\x07world\x1b]\x07", "helloworld\n");
    test(
        b"\x1b]message\x07hello\x1b]message\x07world\x1b]message\x07",
        "helloworld\n",
    );
    test(
        b"\x1b]mes\ns\tage\x07hello\x1b]mes\ns\tage\x07world\x1b]mes\ns\tage\x07",
        "helloworld\n",
    );
}

#[test]
fn test_linux() {
    test(b"\x1b[[A", "\n");
    test(b"\x1b[[Ahello\x1b[[Aworld\x1b[[A", "helloworld\n");
}

#[test]
fn test_unassigned() {
    test("\u{1d455}".as_bytes(), "\u{fffd}\n");
    test("\u{1d455}\n".as_bytes(), "\u{fffd}\n");
}

#[test]
fn test_dddha() {
    test("\u{11099}\u{110ba}".as_bytes(), "\u{1109a}\n");
    test("\u{1109a}".as_bytes(), "\u{1109a}\n");
    test("\u{110ba}".as_bytes(), "\u{34f}\u{110ba}\n");
    test("\u{110ba}\n".as_bytes(), "\u{34f}\u{110ba}\n");
    test("\u{11099}\u{110ba}\n".as_bytes(), "\u{1109a}\n");
    test("\u{1109a}\n".as_bytes(), "\u{1109a}\n");
}

// TODO: Test Stream-Safe
// TODO: test for nonstarter after push
