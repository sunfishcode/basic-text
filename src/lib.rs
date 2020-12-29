//! Plain and restricted text I/O and strings

#![deny(missing_docs)]
#![cfg_attr(can_vector, feature(can_vector))]
#![cfg_attr(write_all_vectored, feature(write_all_vectored))]
#![cfg_attr(try_reserve, feature(try_reserve))]
#![cfg_attr(pattern, feature(pattern))]
#![cfg_attr(shrink_to, feature(shrink_to))]

mod categorize;
mod copy;
mod rc_char_queue;
mod read_str;
mod read_text;
mod replace_selected;
mod text_input;
mod text_interactor;
mod text_output;
mod text_reader;
mod text_string;
mod text_writer;
mod unicode;
mod utf8_input;
mod utf8_interactor;
mod utf8_output;
mod utf8_reader;
mod utf8_writer;
mod write_text;
mod write_wrapper;

pub use copy::{copy_str, copy_text};
pub use read_str::{default_read_exact_str, ReadStr};
pub use read_text::{default_read_exact_text, ReadText};
pub use text_interactor::TextInteractor;
pub use text_reader::TextReader;
pub use text_string::{TextStr, TextString};
pub use text_writer::TextWriter;
pub use unicode::NORMALIZATION_BUFFER_SIZE;
pub use utf8_interactor::Utf8Interactor;
pub use utf8_reader::Utf8Reader;
pub use utf8_writer::Utf8Writer;
pub use write_text::{default_write_text, WriteText};
pub use write_wrapper::WriteWrapper;
