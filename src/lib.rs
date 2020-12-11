//! Streams of UTF-8, text, and restricted text.

#![deny(missing_docs)]
#![cfg_attr(can_vector, feature(can_vector))]
#![cfg_attr(write_all_vectored, feature(write_all_vectored))]

mod categorize;
mod copy_str;
mod rc_char_queue;
mod read_str;
mod replace_selected;
mod text_input;
mod text_interactor;
mod text_output;
mod text_reader;
mod text_writer;
mod unicode;
mod utf8_input;
mod utf8_interactor;
mod utf8_output;
mod utf8_reader;
mod utf8_writer;

pub use copy_str::copy_str;
pub use read_str::{default_read_exact_str, ReadStr};
pub use text_interactor::TextInteractor;
pub use text_reader::TextReader;
pub use text_writer::TextWriter;
pub use unicode::NORMALIZATION_BUFFER_SIZE;
pub use utf8_interactor::Utf8Interactor;
pub use utf8_reader::Utf8Reader;
pub use utf8_writer::{Utf8Writer, WriteWrapper};
