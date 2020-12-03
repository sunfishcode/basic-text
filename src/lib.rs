//! Streams of bytes, UTF-8, and plain text.

#![deny(missing_docs)]

mod no_forbidden_characters;
mod rc_char_queue;
mod text_reader;
mod text_reader_impl;
mod text_reader_writer;
mod text_writer;
mod text_writer_impl;
mod unicode;
mod utf8_reader;
mod utf8_reader_impl;
mod utf8_reader_writer;
mod utf8_writer;
mod utf8_writer_impl;

pub use text_reader::TextReader;
pub use text_reader_writer::TextReaderWriter;
pub use text_writer::TextWriter;
pub use unicode::NORMALIZATION_BUFFER_SIZE;
pub use utf8_reader::{ReadStr, Utf8Reader};
pub use utf8_reader_writer::Utf8ReaderWriter;
pub use utf8_writer::{Utf8Writer, WriteWrapper};
