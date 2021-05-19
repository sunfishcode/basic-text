use crate::TextString;
use basic_text_internals::{is_basic_text, is_basic_text_end, unicode::CGJ};
use std::{
    borrow::Cow,
    io::{self, BufRead},
};

/// An extension trait for `BufRead` which adds functions for reading
/// lines as `TextString`s.
pub trait BufReadText: BufRead {
    /// Read all bytes until a newline (the `0xA` byte) is reached, and append
    /// them to the provided buffer, similar to [`BufRead::read_line`], but
    /// require the input to contain valid Basic Text, and require each line
    /// to be a valid Basic Text string.
    ///
    /// Basic Text streams always end with a newline, so the returned string
    /// will always have a trailing newline.
    ///
    /// This function is blocking and should be used carefully: it is possible
    /// for an attacker to continuously send bytes without ever sending a
    /// newline or ending the stream.
    fn read_text_line(&mut self, buf: &mut TextString) -> io::Result<usize> {
        let len = self.read_line(&mut buf.0)?;

        if !is_basic_text(&buf.0) {
            buf.0.clear();
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stream did not contain valid Basic Text",
            ));
        }

        Ok(len)
    }

    /// Read all bytes until a newline (the `0xA` byte) is reached, and append
    /// them to the provided buffer, similar to [`BufRead::read_line`],
    /// converting the input to Basic Text using lossy conversions if needed.
    ///
    /// Basic Text streams always end with a newline, so the returned string
    /// will always have a trailing newline.
    ///
    /// This function is blocking and should be used carefully: it is possible
    /// for an attacker to continuously send bytes without ever sending a
    /// newline or ending the stream.
    fn read_text_line_lossy(&mut self, buf: &mut TextString) -> io::Result<usize> {
        let len = self.read_line(&mut buf.0)?;

        if let Cow::Owned(text) = TextString::from_text_lossy(&buf.0) {
            buf.0 = text.0;
        }

        Ok(len)
    }

    /// Returns an iterator over the lines of this reader, similar to
    /// [`BufRead::lines`], but returning `TextString`s.
    fn text_lines(self) -> TextLines<Self>
    where
        Self: Sized,
    {
        TextLines { buf: self }
    }

    /// Returns an iterator over the lines of this reader, similar to
    /// [`BufRead::lines`], but returning `TextString`s, converting the
    /// input to Basic Text using lossy conversions if needed.
    fn text_lines_lossy(self) -> TextLinesLossy<Self>
    where
        Self: Sized,
    {
        TextLinesLossy { buf: self }
    }
}

/// An iterator over the lines of an instance of `BufReadText`.
///
/// This struct is generally created by calling [`text_lines`] on a
/// `BufReadText`. Please see the documentation of [`text_lines`] for more
/// details.
///
/// [`text_lines`]: BufReadText::text_lines
#[derive(Debug)]
pub struct TextLines<B> {
    buf: B,
}

impl<B: BufReadText> Iterator for TextLines<B> {
    type Item = io::Result<TextString>;

    fn next(&mut self) -> Option<io::Result<TextString>> {
        let mut buf = TextString::new();
        match self.buf.read_text_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                debug_assert!(buf.0.ends_with('\n'));
                buf.0.pop();
                debug_assert!(!buf.0.ends_with('\r'));

                if let Some(c) = buf.0.chars().next_back() {
                    if !is_basic_text_end(c) {
                        return Some(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "stream did not contain valid Basic Text lines",
                        )));
                    }
                }

                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// An iterator over the lines of an instance of `BufReadText`.
///
/// This struct is generally created by calling [`text_lines_lossy`] on a
/// `BufReadText`. Please see the documentation of [`text_lines_lossy`] for
/// more details.
///
/// [`text_lines_lossy`]: BufReadText::text_lines_lossy
#[derive(Debug)]
pub struct TextLinesLossy<B> {
    buf: B,
}

impl<B: BufReadText> Iterator for TextLinesLossy<B> {
    type Item = io::Result<TextString>;

    fn next(&mut self) -> Option<io::Result<TextString>> {
        let mut buf = TextString::new();
        match self.buf.read_text_line_lossy(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                debug_assert!(buf.0.ends_with('\n'));
                buf.0.pop();
                debug_assert!(!buf.0.ends_with('\r'));

                // We just popped the newline, so make sure we're not exposing
                // an invalid end.
                if let Some(c) = buf.0.chars().next_back() {
                    if !is_basic_text_end(c) {
                        buf.0.push(CGJ);
                    }
                }

                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

// Implement `BufReadText` for all `BufRead` implementations.
impl<T: BufRead> BufReadText for T {}
