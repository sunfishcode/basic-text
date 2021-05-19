//! This file contains various utilities which are sensitive to the Unicode
//! version. Is is currently up to date with Unicode 13.0.0 and
//! Unicode Text Segmentation revision 37.
//!
//! Ideally the major predicates in this file should be auto-generated from
//! the Unicode data files rather than manually maintained.

use crate::{
    categorize::Categorize,
    unicode::{is_normalization_form_starter, ESC, SUB, ZWJ},
};
use std::{cell::RefCell, rc::Rc};
use unicode_normalization::{is_nfc_stream_safe, is_nfc_stream_safe_quick, IsNormalized};

/// Test whether `c` is a valid start value for a string in Basic Text.
#[inline]
pub fn is_basic_text_start(c: char) -> bool {
    is_starter(c) &&
    // https://unicode.org/reports/tr29/tr29-37.html#Grapheme_Cluster_Break_Property_Values
    // ZWJ
    c != ZWJ &&
    // Extend
    !is_grapheme_extend_not_cgj(c) &&
    !is_emoji_modifier(c) &&
    // SpacingMark (plus some Extend, which is redundant here)
    !is_grapheme_cluster_break_spacing_mark_plus(c)
}

/// Test whether `c` is a valid end value for a string in Basic Text.
#[inline]
pub fn is_basic_text_end(c: char) -> bool {
    // https://unicode.org/reports/tr29/tr29-37.html#Grapheme_Cluster_Break_Property_Values
    // ZWJ
    c != ZWJ &&
    // Prepend
    !indic_syllabic_category_consonant_preceding_repha(c) &&
    !indic_syllabic_category_consonant_prefixed(c) &&
    !prepended_concatenation_mark(c)
}

/// Test whether `s` is a valid string in Basic Text.
#[inline]
pub fn is_basic_text(s: &str) -> bool {
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

    is_basic_text_substr(s)
}

/// Test whether `s` is a valid string in Basic Text.
#[inline]
pub fn is_basic_text_substr(s: &str) -> bool {
    !Categorize::new(s.chars(), Rc::new(RefCell::new(None))).any(|c| matches!(c, SUB | ESC))
        && is_nfc_stream_safe(s)
}

/// Test whether `s` is a valid string in Basic Text quickly, in a way that
/// may return `None` if it can't be determined quickly.
#[inline]
pub fn is_basic_text_substr_quick(s: &str) -> Option<bool> {
    if !Categorize::new(s.chars(), Rc::new(RefCell::new(None))).any(|c| matches!(c, SUB | ESC)) {
        return Some(false);
    }

    match is_nfc_stream_safe_quick(s.chars()) {
        IsNormalized::Yes => Some(true),
        IsNormalized::No => Some(false),
        IsNormalized::Maybe => None,
    }
}

#[inline]
fn is_starter(c: char) -> bool {
    // All ASCII values are starters and many of them are common, so
    // add a fast-path optimization for this case.
    c.is_ascii() || is_normalization_form_starter(c)
}

/// `Grapheme_Extend = Yes`, except CGJ
const fn is_grapheme_extend_not_cgj(c: char) -> bool {
    // Unicode 13.0.0, DerivedCoreProperties.txt
    matches!(
        c,
        '\u{300}'..='\u{34e}' // exclude U+34F (CGJ)
        | '\u{350}'..='\u{36f}'
        | '\u{483}'..='\u{487}'
        | '\u{488}'..='\u{489}'
        | '\u{591}'..='\u{5bd}'
        | '\u{5bf}'
        | '\u{5c1}'..='\u{5c2}'
        | '\u{5c4}'..='\u{5c5}'
        | '\u{5c7}'
        | '\u{610}'..='\u{61a}'
        | '\u{64b}'..='\u{65f}'
        | '\u{670}'
        | '\u{6d6}'..='\u{6dc}'
        | '\u{6df}'..='\u{6e4}'
        | '\u{6e7}'..='\u{6e8}'
        | '\u{6ea}'..='\u{6ed}'
        | '\u{711}'
        | '\u{730}'..='\u{74a}'
        | '\u{7a6}'..='\u{7b0}'
        | '\u{7eb}'..='\u{7f3}'
        | '\u{7fd}'
        | '\u{816}'..='\u{819}'
        | '\u{81b}'..='\u{823}'
        | '\u{825}'..='\u{827}'
        | '\u{829}'..='\u{82d}'
        | '\u{859}'..='\u{85b}'
        | '\u{8d3}'..='\u{8e1}'
        | '\u{8e3}'..='\u{902}'
        | '\u{93a}'
        | '\u{93c}'
        | '\u{941}'..='\u{948}'
        | '\u{94d}'
        | '\u{951}'..='\u{957}'
        | '\u{962}'..='\u{963}'
        | '\u{981}'
        | '\u{9bc}'
        | '\u{9be}'
        | '\u{9c1}'..='\u{9c4}'
        | '\u{9cd}'
        | '\u{9d7}'
        | '\u{9e2}'..='\u{9e3}'
        | '\u{9fe}'
        | '\u{a01}'..='\u{a02}'
        | '\u{a3c}'
        | '\u{a41}'..='\u{a42}'
        | '\u{a47}'..='\u{a48}'
        | '\u{a4b}'..='\u{a4d}'
        | '\u{a51}'
        | '\u{a70}'..='\u{a71}'
        | '\u{a75}'
        | '\u{a81}'..='\u{a82}'
        | '\u{abc}'
        | '\u{ac1}'..='\u{ac5}'
        | '\u{ac7}'..='\u{ac8}'
        | '\u{acd}'
        | '\u{ae2}'..='\u{ae3}'
        | '\u{afa}'..='\u{aff}'
        | '\u{b01}'
        | '\u{b3c}'
        | '\u{b3e}'
        | '\u{b3f}'
        | '\u{b41}'..='\u{b44}'
        | '\u{b4d}'
        | '\u{b55}'..='\u{b56}'
        | '\u{b57}'
        | '\u{b62}'..='\u{b63}'
        | '\u{b82}'
        | '\u{bbe}'
        | '\u{bc0}'
        | '\u{bcd}'
        | '\u{bd7}'
        | '\u{c00}'
        | '\u{c04}'
        | '\u{c3e}'..='\u{c40}'
        | '\u{c46}'..='\u{c48}'
        | '\u{c4a}'..='\u{c4d}'
        | '\u{c55}'..='\u{c56}'
        | '\u{c62}'..='\u{c63}'
        | '\u{c81}'
        | '\u{cbc}'
        | '\u{cbf}'
        | '\u{cc2}'
        | '\u{cc6}'
        | '\u{ccc}'..='\u{ccd}'
        | '\u{cd5}'..='\u{cd6}'
        | '\u{ce2}'..='\u{ce3}'
        | '\u{d00}'..='\u{d01}'
        | '\u{d3b}'..='\u{d3c}'
        | '\u{d3e}'
        | '\u{d41}'..='\u{d44}'
        | '\u{d4d}'
        | '\u{d57}'
        | '\u{d62}'..='\u{d63}'
        | '\u{d81}'
        | '\u{dca}'
        | '\u{dcf}'
        | '\u{dd2}'..='\u{dd4}'
        | '\u{dd6}'
        | '\u{ddf}'
        | '\u{e31}'
        | '\u{e34}'..='\u{e3a}'
        | '\u{e47}'..='\u{e4e}'
        | '\u{eb1}'
        | '\u{eb4}'..='\u{ebc}'
        | '\u{ec8}'..='\u{ecd}'
        | '\u{f18}'..='\u{f19}'
        | '\u{f35}'
        | '\u{f37}'
        | '\u{f39}'
        | '\u{f71}'..='\u{f7e}'
        | '\u{f80}'..='\u{f84}'
        | '\u{f86}'..='\u{f87}'
        | '\u{f8d}'..='\u{f97}'
        | '\u{f99}'..='\u{fbc}'
        | '\u{fc6}'
        | '\u{102d}'..='\u{1030}'
        | '\u{1032}'..='\u{1037}'
        | '\u{1039}'..='\u{103a}'
        | '\u{103d}'..='\u{103e}'
        | '\u{1058}'..='\u{1059}'
        | '\u{105e}'..='\u{1060}'
        | '\u{1071}'..='\u{1074}'
        | '\u{1082}'
        | '\u{1085}'..='\u{1086}'
        | '\u{108d}'
        | '\u{109d}'
        | '\u{135d}'..='\u{135f}'
        | '\u{1712}'..='\u{1714}'
        | '\u{1732}'..='\u{1734}'
        | '\u{1752}'..='\u{1753}'
        | '\u{1772}'..='\u{1773}'
        | '\u{17b4}'..='\u{17b5}'
        | '\u{17b7}'..='\u{17bd}'
        | '\u{17c6}'
        | '\u{17c9}'..='\u{17d3}'
        | '\u{17dd}'
        | '\u{180b}'..='\u{180d}'
        | '\u{1885}'..='\u{1886}'
        | '\u{18a9}'
        | '\u{1920}'..='\u{1922}'
        | '\u{1927}'..='\u{1928}'
        | '\u{1932}'
        | '\u{1939}'..='\u{193b}'
        | '\u{1a17}'..='\u{1a18}'
        | '\u{1a1b}'
        | '\u{1a56}'
        | '\u{1a58}'..='\u{1a5e}'
        | '\u{1a60}'
        | '\u{1a62}'
        | '\u{1a65}'..='\u{1a6c}'
        | '\u{1a73}'..='\u{1a7c}'
        | '\u{1a7f}'
        | '\u{1ab0}'..='\u{1abd}'
        | '\u{1abe}'
        | '\u{1abf}'..='\u{1ac0}'
        | '\u{1b00}'..='\u{1b03}'
        | '\u{1b34}'
        | '\u{1b35}'
        | '\u{1b36}'..='\u{1b3a}'
        | '\u{1b3c}'
        | '\u{1b42}'
        | '\u{1b6b}'..='\u{1b73}'
        | '\u{1b80}'..='\u{1b81}'
        | '\u{1ba2}'..='\u{1ba5}'
        | '\u{1ba8}'..='\u{1ba9}'
        | '\u{1bab}'..='\u{1bad}'
        | '\u{1be6}'
        | '\u{1be8}'..='\u{1be9}'
        | '\u{1bed}'
        | '\u{1bef}'..='\u{1bf1}'
        | '\u{1c2c}'..='\u{1c33}'
        | '\u{1c36}'..='\u{1c37}'
        | '\u{1cd0}'..='\u{1cd2}'
        | '\u{1cd4}'..='\u{1ce0}'
        | '\u{1ce2}'..='\u{1ce8}'
        | '\u{1ced}'
        | '\u{1cf4}'
        | '\u{1cf8}'..='\u{1cf9}'
        | '\u{1dc0}'..='\u{1df9}'
        | '\u{1dfb}'..='\u{1dff}'
        | '\u{200c}'
        | '\u{20d0}'..='\u{20dc}'
        | '\u{20dd}'..='\u{20e0}'
        | '\u{20e1}'
        | '\u{20e2}'..='\u{20e4}'
        | '\u{20e5}'..='\u{20f0}'
        | '\u{2cef}'..='\u{2cf1}'
        | '\u{2d7f}'
        | '\u{2de0}'..='\u{2dff}'
        | '\u{302a}'..='\u{302d}'
        | '\u{302e}'..='\u{302f}'
        | '\u{3099}'..='\u{309a}'
        | '\u{a66f}'
        | '\u{a670}'..='\u{a672}'
        | '\u{a674}'..='\u{a67d}'
        | '\u{a69e}'..='\u{a69f}'
        | '\u{a6f0}'..='\u{a6f1}'
        | '\u{a802}'
        | '\u{a806}'
        | '\u{a80b}'
        | '\u{a825}'..='\u{a826}'
        | '\u{a82c}'
        | '\u{a8c4}'..='\u{a8c5}'
        | '\u{a8e0}'..='\u{a8f1}'
        | '\u{a8ff}'
        | '\u{a926}'..='\u{a92d}'
        | '\u{a947}'..='\u{a951}'
        | '\u{a980}'..='\u{a982}'
        | '\u{a9b3}'
        | '\u{a9b6}'..='\u{a9b9}'
        | '\u{a9bc}'..='\u{a9bd}'
        | '\u{a9e5}'
        | '\u{aa29}'..='\u{aa2e}'
        | '\u{aa31}'..='\u{aa32}'
        | '\u{aa35}'..='\u{aa36}'
        | '\u{aa43}'
        | '\u{aa4c}'
        | '\u{aa7c}'
        | '\u{aab0}'
        | '\u{aab2}'..='\u{aab4}'
        | '\u{aab7}'..='\u{aab8}'
        | '\u{aabe}'..='\u{aabf}'
        | '\u{aac1}'
        | '\u{aaec}'..='\u{aaed}'
        | '\u{aaf6}'
        | '\u{abe5}'
        | '\u{abe8}'
        | '\u{abed}'
        | '\u{fb1e}'
        | '\u{fe00}'..='\u{fe0f}'
        | '\u{fe20}'..='\u{fe2f}'
        | '\u{ff9e}'..='\u{ff9f}'
        | '\u{101fd}'
        | '\u{102e0}'
        | '\u{10376}'..='\u{1037a}'
        | '\u{10a01}'..='\u{10a03}'
        | '\u{10a05}'..='\u{10a06}'
        | '\u{10a0c}'..='\u{10a0f}'
        | '\u{10a38}'..='\u{10a3a}'
        | '\u{10a3f}'
        | '\u{10ae5}'..='\u{10ae6}'
        | '\u{10d24}'..='\u{10d27}'
        | '\u{10eab}'..='\u{10eac}'
        | '\u{10f46}'..='\u{10f50}'
        | '\u{11001}'
        | '\u{11038}'..='\u{11046}'
        | '\u{1107f}'..='\u{11081}'
        | '\u{110b3}'..='\u{110b6}'
        | '\u{110b9}'..='\u{110ba}'
        | '\u{11100}'..='\u{11102}'
        | '\u{11127}'..='\u{1112b}'
        | '\u{1112d}'..='\u{11134}'
        | '\u{11173}'
        | '\u{11180}'..='\u{11181}'
        | '\u{111b6}'..='\u{111be}'
        | '\u{111c9}'..='\u{111cc}'
        | '\u{111cf}'
        | '\u{1122f}'..='\u{11231}'
        | '\u{11234}'
        | '\u{11236}'..='\u{11237}'
        | '\u{1123e}'
        | '\u{112df}'
        | '\u{112e3}'..='\u{112ea}'
        | '\u{11300}'..='\u{11301}'
        | '\u{1133b}'..='\u{1133c}'
        | '\u{1133e}'
        | '\u{11340}'
        | '\u{11357}'
        | '\u{11366}'..='\u{1136c}'
        | '\u{11370}'..='\u{11374}'
        | '\u{11438}'..='\u{1143f}'
        | '\u{11442}'..='\u{11444}'
        | '\u{11446}'
        | '\u{1145e}'
        | '\u{114b0}'
        | '\u{114b3}'..='\u{114b8}'
        | '\u{114ba}'
        | '\u{114bd}'
        | '\u{114bf}'..='\u{114c0}'
        | '\u{114c2}'..='\u{114c3}'
        | '\u{115af}'
        | '\u{115b2}'..='\u{115b5}'
        | '\u{115bc}'..='\u{115bd}'
        | '\u{115bf}'..='\u{115c0}'
        | '\u{115dc}'..='\u{115dd}'
        | '\u{11633}'..='\u{1163a}'
        | '\u{1163d}'
        | '\u{1163f}'..='\u{11640}'
        | '\u{116ab}'
        | '\u{116ad}'
        | '\u{116b0}'..='\u{116b5}'
        | '\u{116b7}'
        | '\u{1171d}'..='\u{1171f}'
        | '\u{11722}'..='\u{11725}'
        | '\u{11727}'..='\u{1172b}'
        | '\u{1182f}'..='\u{11837}'
        | '\u{11839}'..='\u{1183a}'
        | '\u{11930}'
        | '\u{1193b}'..='\u{1193c}'
        | '\u{1193e}'
        | '\u{11943}'
        | '\u{119d4}'..='\u{119d7}'
        | '\u{119da}'..='\u{119db}'
        | '\u{119e0}'
        | '\u{11a01}'..='\u{11a0a}'
        | '\u{11a33}'..='\u{11a38}'
        | '\u{11a3b}'..='\u{11a3e}'
        | '\u{11a47}'
        | '\u{11a51}'..='\u{11a56}'
        | '\u{11a59}'..='\u{11a5b}'
        | '\u{11a8a}'..='\u{11a96}'
        | '\u{11a98}'..='\u{11a99}'
        | '\u{11c30}'..='\u{11c36}'
        | '\u{11c38}'..='\u{11c3d}'
        | '\u{11c3f}'
        | '\u{11c92}'..='\u{11ca7}'
        | '\u{11caa}'..='\u{11cb0}'
        | '\u{11cb2}'..='\u{11cb3}'
        | '\u{11cb5}'..='\u{11cb6}'
        | '\u{11d31}'..='\u{11d36}'
        | '\u{11d3a}'
        | '\u{11d3c}'..='\u{11d3d}'
        | '\u{11d3f}'..='\u{11d45}'
        | '\u{11d47}'
        | '\u{11d90}'..='\u{11d91}'
        | '\u{11d95}'
        | '\u{11d97}'
        | '\u{11ef3}'..='\u{11ef4}'
        | '\u{16af0}'..='\u{16af4}'
        | '\u{16b30}'..='\u{16b36}'
        | '\u{16f4f}'
        | '\u{16f8f}'..='\u{16f92}'
        | '\u{16fe4}'
        | '\u{1bc9d}'..='\u{1bc9e}'
        | '\u{1d165}'
        | '\u{1d167}'..='\u{1d169}'
        | '\u{1d16e}'..='\u{1d172}'
        | '\u{1d17b}'..='\u{1d182}'
        | '\u{1d185}'..='\u{1d18b}'
        | '\u{1d1aa}'..='\u{1d1ad}'
        | '\u{1d242}'..='\u{1d244}'
        | '\u{1da00}'..='\u{1da36}'
        | '\u{1da3b}'..='\u{1da6c}'
        | '\u{1da75}'
        | '\u{1da84}'
        | '\u{1da9b}'..='\u{1da9f}'
        | '\u{1daa1}'..='\u{1daaf}'
        | '\u{1e000}'..='\u{1e006}'
        | '\u{1e008}'..='\u{1e018}'
        | '\u{1e01b}'..='\u{1e021}'
        | '\u{1e023}'..='\u{1e024}'
        | '\u{1e026}'..='\u{1e02a}'
        | '\u{1e130}'..='\u{1e136}'
        | '\u{1e2ec}'..='\u{1e2ef}'
        | '\u{1e8d0}'..='\u{1e8d6}'
        | '\u{1e944}'..='\u{1e94a}'
        | '\u{e0020}'..='\u{e007f}'
        | '\u{e0100}'..='\u{e01ef}',
    )
}

/// `Emoji_Modifier = Yes`
const fn is_emoji_modifier(c: char) -> bool {
    // Unicode 13.0.0, emoji/emoji-data.txt
    matches!(c, '\u{1f3fb}'..='\u{1f3ff}')
}

/// `Grapheme_Cluster_Break = SpacingMark`, ignoring the
/// `Grapheme_Cluster_Break ≠ Extend` rule, because it's redundant here.
const fn is_grapheme_cluster_break_spacing_mark_plus(c: char) -> bool {
    c == '\u{e33}'
        || c == '\u{eb3}'
        || (is_general_category_spacing_mark(c)
            && !matches!(
                    c,
                    '\u{102b}'
                    | '\u{102c}'
                    | '\u{1038}'
                    | '\u{1062}'..='\u{1064}'
                    | '\u{1067}'..='\u{106d}'
                    | '\u{1083}'
                    | '\u{1087}'..='\u{108c}'
                    | '\u{108f}'
                    | '\u{109a}'..='\u{109c}'
                    | '\u{1a61}'
                    | '\u{1a63}'
                    | '\u{1a64}'
                    | '\u{aa7b}'
                    | '\u{aa7d}'
                    | '\u{11720}'
                    | '\u{11721}',
            ))
}

/// `General_Category = Spacing_Mark`
const fn is_general_category_spacing_mark(c: char) -> bool {
    // Unicode 13.0.0, DerivedGeneralCategory.txt
    matches!(
        c,
        '\u{903}'
        | '\u{93b}'
        | '\u{93e}'..='\u{940}'
        | '\u{949}'..='\u{94c}'
        | '\u{94e}'..='\u{94f}'
        | '\u{982}'..='\u{983}'
        | '\u{9be}'..='\u{9c0}'
        | '\u{9c7}'..='\u{9c8}'
        | '\u{9cb}'..='\u{9cc}'
        | '\u{9d7}'
        | '\u{a03}'
        | '\u{a3e}'..='\u{a40}'
        | '\u{a83}'
        | '\u{abe}'..='\u{ac0}'
        | '\u{ac9}'
        | '\u{acb}'..='\u{acc}'
        | '\u{b02}'..='\u{b03}'
        | '\u{b3e}'
        | '\u{b40}'
        | '\u{b47}'..='\u{b48}'
        | '\u{b4b}'..='\u{b4c}'
        | '\u{b57}'
        | '\u{bbe}'..='\u{bbf}'
        | '\u{bc1}'..='\u{bc2}'
        | '\u{bc6}'..='\u{bc8}'
        | '\u{bca}'..='\u{bcc}'
        | '\u{bd7}'
        | '\u{c01}'..='\u{c03}'
        | '\u{c41}'..='\u{c44}'
        | '\u{c82}'..='\u{c83}'
        | '\u{cbe}'
        | '\u{cc0}'..='\u{cc4}'
        | '\u{cc7}'..='\u{cc8}'
        | '\u{cca}'..='\u{ccb}'
        | '\u{cd5}'..='\u{cd6}'
        | '\u{d02}'..='\u{d03}'
        | '\u{d3e}'..='\u{d40}'
        | '\u{d46}'..='\u{d48}'
        | '\u{d4a}'..='\u{d4c}'
        | '\u{d57}'
        | '\u{d82}'..='\u{d83}'
        | '\u{dcf}'..='\u{dd1}'
        | '\u{dd8}'..='\u{ddf}'
        | '\u{df2}'..='\u{df3}'
        | '\u{f3e}'..='\u{f3f}'
        | '\u{f7f}'
        | '\u{102b}'..='\u{102c}'
        | '\u{1031}'
        | '\u{1038}'
        | '\u{103b}'..='\u{103c}'
        | '\u{1056}'..='\u{1057}'
        | '\u{1062}'..='\u{1064}'
        | '\u{1067}'..='\u{106d}'
        | '\u{1083}'..='\u{1084}'
        | '\u{1087}'..='\u{108c}'
        | '\u{108f}'
        | '\u{109a}'..='\u{109c}'
        | '\u{17b6}'
        | '\u{17be}'..='\u{17c5}'
        | '\u{17c7}'..='\u{17c8}'
        | '\u{1923}'..='\u{1926}'
        | '\u{1929}'..='\u{192b}'
        | '\u{1930}'..='\u{1931}'
        | '\u{1933}'..='\u{1938}'
        | '\u{1a19}'..='\u{1a1a}'
        | '\u{1a55}'
        | '\u{1a57}'
        | '\u{1a61}'
        | '\u{1a63}'..='\u{1a64}'
        | '\u{1a6d}'..='\u{1a72}'
        | '\u{1b04}'
        | '\u{1b35}'
        | '\u{1b3b}'
        | '\u{1b3d}'..='\u{1b41}'
        | '\u{1b43}'..='\u{1b44}'
        | '\u{1b82}'
        | '\u{1ba1}'
        | '\u{1ba6}'..='\u{1ba7}'
        | '\u{1baa}'
        | '\u{1be7}'
        | '\u{1bea}'..='\u{1bec}'
        | '\u{1bee}'
        | '\u{1bf2}'..='\u{1bf3}'
        | '\u{1c24}'..='\u{1c2b}'
        | '\u{1c34}'..='\u{1c35}'
        | '\u{1ce1}'
        | '\u{1cf7}'
        | '\u{302e}'..='\u{302f}'
        | '\u{a823}'..='\u{a824}'
        | '\u{a827}'
        | '\u{a880}'..='\u{a881}'
        | '\u{a8b4}'..='\u{a8c3}'
        | '\u{a952}'..='\u{a953}'
        | '\u{a983}'
        | '\u{a9b4}'..='\u{a9b5}'
        | '\u{a9ba}'..='\u{a9bb}'
        | '\u{a9be}'..='\u{a9c0}'
        | '\u{aa2f}'..='\u{aa30}'
        | '\u{aa33}'..='\u{aa34}'
        | '\u{aa4d}'
        | '\u{aa7b}'
        | '\u{aa7d}'
        | '\u{aaeb}'
        | '\u{aaee}'..='\u{aaef}'
        | '\u{aaf5}'
        | '\u{abe3}'..='\u{abe4}'
        | '\u{abe6}'..='\u{abe7}'
        | '\u{abe9}'..='\u{abea}'
        | '\u{abec}'
        | '\u{11000}'
        | '\u{11002}'
        | '\u{11082}'
        | '\u{110b0}'..='\u{110b2}'
        | '\u{110b7}'..='\u{110b8}'
        | '\u{1112c}'
        | '\u{11145}'..='\u{11146}'
        | '\u{11182}'
        | '\u{111b3}'..='\u{111b5}'
        | '\u{111bf}'..='\u{111c0}'
        | '\u{111ce}'
        | '\u{1122c}'..='\u{1122e}'
        | '\u{11232}'..='\u{11233}'
        | '\u{11235}'
        | '\u{112e0}'..='\u{112e2}'
        | '\u{11302}'..='\u{11303}'
        | '\u{1133e}'..='\u{1133f}'
        | '\u{11341}'..='\u{11344}'
        | '\u{11347}'..='\u{11348}'
        | '\u{1134b}'..='\u{1134d}'
        | '\u{11357}'
        | '\u{11362}'..='\u{11363}'
        | '\u{11435}'..='\u{11437}'
        | '\u{11440}'..='\u{11441}'
        | '\u{11445}'
        | '\u{114b0}'..='\u{114b2}'
        | '\u{114b9}'
        | '\u{114bb}'..='\u{114be}'
        | '\u{114c1}'
        | '\u{115af}'..='\u{115b1}'
        | '\u{115b8}'..='\u{115bb}'
        | '\u{115be}'
        | '\u{11630}'..='\u{11632}'
        | '\u{1163b}'..='\u{1163c}'
        | '\u{1163e}'
        | '\u{116ac}'
        | '\u{116ae}'..='\u{116af}'
        | '\u{116b6}'
        | '\u{11720}'..='\u{11721}'
        | '\u{11726}'
        | '\u{1182c}'..='\u{1182e}'
        | '\u{11838}'
        | '\u{11930}'..='\u{11935}'
        | '\u{11937}'..='\u{11938}'
        | '\u{1193d}'
        | '\u{11940}'
        | '\u{11942}'
        | '\u{119d1}'..='\u{119d3}'
        | '\u{119dc}'..='\u{119df}'
        | '\u{119e4}'
        | '\u{11a39}'
        | '\u{11a57}'..='\u{11a58}'
        | '\u{11a97}'
        | '\u{11c2f}'
        | '\u{11c3e}'
        | '\u{11ca9}'
        | '\u{11cb1}'
        | '\u{11cb4}'
        | '\u{11d8a}'..='\u{11d8e}'
        | '\u{11d93}'..='\u{11d94}'
        | '\u{11d96}'
        | '\u{11ef5}'..='\u{11ef6}'
        | '\u{16f51}'..='\u{16f87}'
        | '\u{16ff0}'..='\u{16ff1}'
        | '\u{1d165}'..='\u{1d166}'
        | '\u{1d16d}'..='\u{1d172}',
    )
}

/// `Indic_Syllabic_Category = Consonant_Preceding_Repha`
const fn indic_syllabic_category_consonant_preceding_repha(c: char) -> bool {
    // Unicode 13.0.0, IndicSyllabicCategory.txt
    matches!(c, '\u{d4e}' | '\u{11941}' | '\u{11d46}')
}

/// `Indic_Syllabic_Category = Consonant_Prefixed`
const fn indic_syllabic_category_consonant_prefixed(c: char) -> bool {
    // Unicode 13.0.0, IndicSyllabicCategory.txt
    matches!(
        c,
        '\u{111c2}'..='\u{111c3}' | '\u{1193f}' | '\u{11a3a}' | '\u{11a84}'..='\u{11a89}',
    )
}

/// `Prepended_Concatenation_Mark = Yes`
const fn prepended_concatenation_mark(c: char) -> bool {
    // Unicode 13.0.0, PropList.txt
    matches!(
        c,
        '\u{600}'..='\u{605}' | '\u{6dd}' | '\u{70f}' | '\u{8e2}' | '\u{110bd}' | '\u{110cd}',
    )
}
