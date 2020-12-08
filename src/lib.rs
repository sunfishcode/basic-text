//! Streams of UTF-8, text, and restricted text.

#![deny(missing_docs)]

mod categorize;
mod rc_char_queue;
mod replace_selected;
mod text_input;
mod text_output;
mod text_reader;
mod text_reader_writer;
mod text_writer;
mod unicode;
mod utf8_input;
mod utf8_output;
mod utf8_reader;
mod utf8_reader_writer;
mod utf8_writer;

pub use text_reader::TextReader;
pub use text_reader_writer::TextReaderWriter;
pub use text_writer::TextWriter;
pub use unicode::NORMALIZATION_BUFFER_SIZE;
pub use utf8_reader::{ReadStr, Utf8Reader};
pub use utf8_reader_writer::Utf8ReaderWriter;
pub use utf8_writer::{Utf8Writer, WriteWrapper};
