//! TODO: This uses `ucd` which is unmaintained and based on Unicode 9.0.

use crate::unicode::is_normalization_form_starter;
use ucd::{Codepoint, GraphemeClusterBreak};

pub(crate) fn is_start_ok(c: char) -> bool {
    is_normalization_form_starter(c)
        && !matches!(
            c.grapheme_cluster_break(),
            GraphemeClusterBreak::ZWJ
                | GraphemeClusterBreak::SpacingMark
                | GraphemeClusterBreak::Extend
        )
}

pub(crate) fn is_end_ok(c: char) -> bool {
    !matches!(
        c.grapheme_cluster_break(),
        GraphemeClusterBreak::ZWJ | GraphemeClusterBreak::Prepend
    )
}
