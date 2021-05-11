//! TODO: This uses `ucd` which is unmaintained and based on Unicode 9.0.

use crate::{
    categorize::Categorize,
    unicode::{is_normalization_form_starter, ESC, SUB},
};
use std::{cell::RefCell, rc::Rc};
use ucd::{Codepoint, GraphemeClusterBreak};
use unicode_normalization::is_nfc_stream_safe;

/// Test whether `c` is a valid start value for a string in Basic Text.
pub(crate) fn is_basic_text_start(c: char) -> bool {
    is_normalization_form_starter(c)
        && !matches!(
            c.grapheme_cluster_break(),
            GraphemeClusterBreak::ZWJ
                | GraphemeClusterBreak::SpacingMark
                | GraphemeClusterBreak::Extend
        )
}

/// Test whether `c` is a valid end value for a string in Basic Text.
pub(crate) fn is_basic_text_end(c: char) -> bool {
    !matches!(
        c.grapheme_cluster_break(),
        GraphemeClusterBreak::ZWJ | GraphemeClusterBreak::Prepend
    )
}

/// Test whether `s` is a valid string in Basic Text.
pub(crate) fn is_basic_text(s: &str) -> bool {
    if let Some(c) = s.chars().next() {
        if !is_basic_text_start(c) {
            return false;
        }
    }
    if let Some(c) = s.chars().next_back() {
        if !is_basic_text_end(c) {
            return false;
        }
    }

    !Categorize::new(s.chars(), Rc::new(RefCell::new(None))).any(|c| matches!(c, SUB | ESC))
        && is_nfc_stream_safe(s)
}
