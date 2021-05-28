pub mod unicode;

mod categorize;
mod check;
mod isolate_unassigned;
mod replace;
mod text_utils;

// Re-export `unicode_normalization` so that our users can use the same version
// we're using.
pub use unicode_normalization;

pub use categorize::Categorize;
pub use check::{check_basic_text_char, BasicTextError};
pub use isolate_unassigned::IsolateUnassigned;
pub use replace::replace;
pub use text_utils::{
    is_basic_text, is_basic_text_end, is_basic_text_start, is_basic_text_substr,
    is_basic_text_substr_quick,
};
