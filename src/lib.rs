//! Basic Text strings and I/O streams
//!
//! This crate provides several utilities for working with [Basic Text].
//!
//!  - [`TextString`] and [`TextStr`] are similar to the standard library's [`String`]
//!    and [`str`], but use the Basic Text string format, along with a
//!    [`text!("...")` macro] for Basic Text string literals.
//!
//!  - [`TextReader`] and [`TextWriter`] are input and output streams which use
//!    the Basic Text stream format. On input, content is converted in a way
//!    which is lossy with respect to the original bytestream. Output uses the
//!    "strict" conversion method, in which invalid content is diagnosed with
//!    errors.
//!
//!  - [`BufReadText`], an extension trait that adds [`text_lines`] and
//!    [`text_lines_lossy`] to [`BufRead`] implementations for reading lines
//!    from an input stream as `BasicText` strings.
//!
//!  - [`TextDuplexer`] is a [`Duplex`] for reading and writing on an interactive
//!    stream using Basic Text.
//!
//! # Examples
//!
//! Working with `TextString` and company is overall similar to working with
//! `String` and company, but with a little more syntax in some places:
//!
//! ```rust
//! use basic_text::{text, text_substr, ReadText, TextReader, TextString, TextWriter, WriteText};
//! use std::io::{stdin, stdout, Write};
//!
//! // Wrap stdout in an output stream that ensures that the output is
//! // Basic Text.
//! let mut stream = TextWriter::new(stdout());
//!
//! // Construct Basic Text literals.
//! let greeting = text!("Hello, World!");
//!
//! // Write Basic Text directly.
//! stream.write_text(greeting).unwrap();
//!
//! // `TextString` can't be split at arbitrary boundaries, so this crate has
//! // substring types, so you can work with Basic Text content incrementally.
//! // The following code prints the "Service Dog" ZWJ Sequence "🐕‍🦺" in
//! // parts, where splitting it would not be valid in Basic Text.
//! stream
//!     .write_text_substr(text_substr!("🐕\u{200d}"))
//!     .unwrap();
//! stream.write_text_substr(text_substr!("🦺")).unwrap();
//!
//! // Regular strings with Basic Text content can be written.
//! writeln!(stream, "Valid!").unwrap();
//!
//! // But invalid content is diagnosed as an error.
//! writeln!(stream, "\u{c}Invalid!\u{7}").unwrap_err();
//!
//! // A Basic Text reader, on the other hand, always succeeds, by replacing
//! // invalid sequences with `�`s.
//! let mut s = TextString::new();
//! TextReader::new(stdin())
//!     .read_to_text_string(&mut s)
//!     .unwrap();
//! ```
//!
//! [Basic Text]: https://github.com/sunfishcode/basic-text/blob/main/docs/BasicText.md#basic-text
//! [`text!("...")` macro]: crate::text
//! [`Duplex`]: https://docs.rs/duplex/latest/duplex/trait.Duplex.html
//! [`BufRead`]: https://doc.rust-lang.org/std/io/trait.BufRead.html
//! [`text_lines`]: https://docs.rs/basic-text/latest/basic_text/trait.BufReadText.html#method.text_lines
//! [`text_lines_lossy`]: https://docs.rs/basic-text/latest/basic_text/trait.BufReadText.html#method.text_lines_lossy

#![deny(missing_docs)]
#![cfg_attr(can_vector, feature(can_vector))]
#![cfg_attr(write_all_vectored, feature(write_all_vectored))]
#![cfg_attr(try_reserve, feature(try_reserve))]
#![cfg_attr(pattern, feature(pattern))]
#![cfg_attr(shrink_to, feature(shrink_to))]
#![cfg_attr(extend_one, feature(extend_one))]

mod buf_read_text;
mod copy;
mod partial_eq;
mod read_text;
mod text_duplexer;
mod text_input;
mod text_output;
mod text_reader;
mod text_string;
mod text_substring;
mod text_writer;
mod write_text;

pub use basic_text_internals::{
    unicode::NORMALIZATION_BUFFER_SIZE, unicode_normalization::UNICODE_VERSION,
};
pub use basic_text_literals::{text, text_substr};
pub use buf_read_text::{BufReadText, TextLines, TextLinesLossy};
pub use copy::{copy_text, copy_text_using_status};
pub use read_text::{default_read_exact_text_substr, ReadText, ReadTextLayered};
pub use text_duplexer::TextDuplexer;
pub use text_reader::TextReader;
pub use text_string::{default_read_to_text_string, FromTextError, TextError, TextStr, TextString};
pub use text_substring::{TextSubstr, TextSubstring};
pub use text_writer::TextWriter;
pub use write_text::{default_write_text_substr, WriteText};
