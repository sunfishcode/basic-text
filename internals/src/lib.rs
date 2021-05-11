pub mod unicode;

mod categorize;
mod replace_selected;
mod text_utils;

pub use categorize::Categorize;
pub use replace_selected::ReplaceSelected;
pub use text_utils::{is_basic_text, is_basic_text_end, is_basic_text_start};
