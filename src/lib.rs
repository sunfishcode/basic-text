//! Basic Text strings and I/O streams

#![deny(missing_docs)]
#![cfg_attr(can_vector, feature(can_vector))]
#![cfg_attr(write_all_vectored, feature(write_all_vectored))]
#![cfg_attr(try_reserve, feature(try_reserve))]
#![cfg_attr(pattern, feature(pattern))]
#![cfg_attr(shrink_to, feature(shrink_to))]

mod categorize;
mod copy;
mod read_text;
mod replace_selected;
mod text_duplexer;
mod text_input;
mod text_output;
mod text_reader;
mod text_string;
mod text_writer;
mod unicode;
mod write_text;

pub use copy::{copy_text, copy_text_using_status};
pub use read_text::{default_read_exact_text, ReadText, ReadTextLayered};
pub use text_duplexer::TextDuplexer;
pub use text_reader::TextReader;
pub use text_string::{TextStr, TextString};
pub use text_writer::TextWriter;
pub use unicode::NORMALIZATION_BUFFER_SIZE;
pub use write_text::{default_write_text, WriteText};
