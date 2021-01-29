//! Basic Text strings and I/O streams

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
mod text_duplexer;
mod text_output;
mod text_reader;
mod text_string;
mod text_writer;
mod unicode;
mod write_str;
mod write_text;

pub use copy::{copy_str, copy_text};
pub use read_str::{default_read_exact_str, ReadStr};
pub use read_text::{default_read_exact_text, ReadText};
pub use text_duplexer::TextDuplexer;
pub use text_reader::TextReader;
pub use text_string::{TextStr, TextString};
pub use text_writer::TextWriter;
pub use unicode::NORMALIZATION_BUFFER_SIZE;
pub use write_str::{default_write_fmt, default_write_str, WriteStr};
pub use write_text::{default_write_text, WriteText};
