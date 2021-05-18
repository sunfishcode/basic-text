pub mod unicode;

mod categorize;
mod check;
mod replace;
mod text_utils;

pub use categorize::Categorize;
pub use check::{check_basic_text_char, BasicTextError};
pub use replace::replace;
pub use text_utils::{
    is_basic_text, is_basic_text_end, is_basic_text_start, is_basic_text_substr,
    is_basic_text_substr_quick,
};
