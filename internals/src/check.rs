//! On output, several disallowed scalar values are rejected, to catch
//! applications attempting to use them.

use crate::replace;
use crate::unicode::{BOM, ESC, ORC};
use std::collections::VecDeque;
use thiserror::Error;

/// Test whether the given Unicode scalar value is valid in a Basic Text
/// string.
#[inline]
pub fn check_basic_text_char(c: char) -> Result<(), BasicTextError> {
    match c {
        // Newline and tab are allowed, and escape is handled specially.
        c if c.is_control() && c != '\n' && c != '\t' && c != ESC => control(c),
        '\u{149}'
        | '\u{673}'
        | '\u{f77}'
        | '\u{f79}'
        | '\u{17a3}'
        | '\u{17a4}'
        | '\u{2329}'
        | '\u{232a}'
        | '\u{2126}'
        | '\u{212a}'
        | '\u{212b}'
        | '\u{2df5}'
        | '\u{111c4}'
        | '\u{fb00}'..='\u{fb06}'
        | '\u{9e4}'
        | '\u{9e5}'
        | '\u{a64}'
        | '\u{a65}'
        | '\u{ae4}'
        | '\u{ae5}'
        | '\u{b64}'
        | '\u{b65}'
        | '\u{be4}'
        | '\u{be5}'
        | '\u{c64}'
        | '\u{c65}'
        | '\u{ce4}'
        | '\u{ce5}'
        | '\u{d64}'
        | '\u{d65}'
        | '\u{2072}'
        | '\u{2073}'
        | '\u{1d455}'
        | '\u{1d49d}'
        | '\u{1d4a0}'
        | '\u{1d4a1}'
        | '\u{1d4a3}'
        | '\u{1d4a4}'
        | '\u{1d4a7}'
        | '\u{1d4a8}'
        | '\u{1d4ad}'
        | '\u{1d4ba}'
        | '\u{1d4bc}'
        | '\u{1d4c4}'
        | '\u{1d506}'
        | '\u{1d50b}'
        | '\u{1d50c}'
        | '\u{1d515}'
        | '\u{1d51d}'
        | '\u{1d53a}'
        | '\u{1d53f}'
        | '\u{1d545}'
        | '\u{1d547}'
        | '\u{1d548}'
        | '\u{1d549}'
        | '\u{1d551}'
        | '\u{f900}'..='\u{fa0d}'
        | '\u{fa10}'
        | '\u{fa12}'
        | '\u{fa15}'..='\u{fa1e}'
        | '\u{fa20}'
        | '\u{fa22}'
        | '\u{fa25}'..='\u{fa26}'
        | '\u{fa2a}'..='\u{fa6d}'
        | '\u{fa70}'..='\u{fad9}'
        | '\u{2f800}'..='\u{2fa1d}' => replacement(c),
        '\u{e0001}' => language_tag(),
        '\u{fff9}'..='\u{fffb}' => interlinear_annotation(),
        '\u{17b4}' | '\u{17b5}' => omit(c),
        '\u{17d8}' => beyyal(),
        '\u{206a}'..='\u{206f}' => deprecated_format_character(c),
        '\u{2028}' => line_separation(),
        '\u{2029}' => para_separation(),
        '\u{202a}' | '\u{202b}' | '\u{202c}' | '\u{202d}' | '\u{202e}' | '\u{2066}'
        | '\u{2067}' | '\u{2068}' | '\u{2069}' => bidirectional_formatting_character(),
        '\u{fffe}'..='\u{ffff}'
        | '\u{1fffe}'..='\u{1ffff}'
        | '\u{2fffe}'..='\u{2ffff}'
        | '\u{3fffe}'..='\u{3ffff}'
        | '\u{4fffe}'..='\u{4ffff}'
        | '\u{5fffe}'..='\u{5ffff}'
        | '\u{6fffe}'..='\u{6ffff}'
        | '\u{7fffe}'..='\u{7ffff}'
        | '\u{8fffe}'..='\u{8ffff}'
        | '\u{9fffe}'..='\u{9ffff}'
        | '\u{afffe}'..='\u{affff}'
        | '\u{bfffe}'..='\u{bffff}'
        | '\u{cfffe}'..='\u{cffff}'
        | '\u{dfffe}'..='\u{dffff}'
        | '\u{efffe}'..='\u{effff}'
        | '\u{ffffe}'..='\u{fffff}'
        | '\u{10fffe}'..='\u{10ffff}'
        | '\u{fdd0}'..='\u{fdef}' => noncharacter(),
        ORC => orc(),
        BOM => bom(),
        _ => Ok(()),
    }
}

/// An invalid Unicode scalar value sequence.
#[derive(Error, Debug)]
pub enum BasicTextError {
    #[error("Color escape sequences are not enabled")]
    ColorEscapeSequence,
    #[error("Control code not valid in text: {0:?}")]
    ControlCode(char),
    #[error("Deprecated Format Characters are deprecated: {0:?}")]
    DeprecatedFormatChar(char),
    #[error("Escape code not valid in text")]
    Escape,
    #[error("Explicit Bidirectional Formatting Characters are unsupported")]
    BidiFormatChar,
    #[error("Interlinear Annotations depend on out-of-band information")]
    Interlinear,
    #[error("Language tagging is a deprecated mechanism")]
    LanguageTag,
    #[error("Line separation is a rich-text function")]
    LineSeparation,
    #[error("Noncharacters are intended for internal use only")]
    NonChar,
    #[error("Paragraph separation is a rich-text function")]
    ParaSeparation,
    #[error("U+FEFF is not necessary in Basic Text")]
    UnneededBOM,
    #[error("U+FFFC depends on out-of-band information")]
    OutOfBand,
    #[error("Omit {0:?}")]
    Omit(char),
    #[error("Spell beyyal with normal letters")]
    Beyyal,
    #[error("Unrecognized escape sequence")]
    UnrecognizedEscape,
    #[error("Use {yes:?} instead of {no:?}")]
    Replacement { yes: Box<[char]>, no: char },
}

#[cold]
fn control(c: char) -> Result<(), BasicTextError> {
    Err(BasicTextError::ControlCode(c))
}

#[cold]
fn replacement(c: char) -> Result<(), BasicTextError> {
    let mut queue = VecDeque::new();
    replace(c, &mut queue);
    Err(BasicTextError::Replacement {
        yes: queue.iter().copied().collect::<Vec<_>>().into_boxed_slice(),
        no: c,
    })
}

#[cold]
fn omit(c: char) -> Result<(), BasicTextError> {
    Err(BasicTextError::Omit(c))
}

#[cold]
fn beyyal() -> Result<(), BasicTextError> {
    Err(BasicTextError::Beyyal)
}

#[cold]
fn deprecated_format_character(c: char) -> Result<(), BasicTextError> {
    Err(BasicTextError::DeprecatedFormatChar(c))
}

#[cold]
fn language_tag() -> Result<(), BasicTextError> {
    Err(BasicTextError::LanguageTag)
}

#[cold]
fn line_separation() -> Result<(), BasicTextError> {
    Err(BasicTextError::LineSeparation)
}

#[cold]
fn para_separation() -> Result<(), BasicTextError> {
    Err(BasicTextError::ParaSeparation)
}

#[cold]
fn bidirectional_formatting_character() -> Result<(), BasicTextError> {
    Err(BasicTextError::BidiFormatChar)
}

#[cold]
fn noncharacter() -> Result<(), BasicTextError> {
    Err(BasicTextError::NonChar)
}

#[cold]
fn orc() -> Result<(), BasicTextError> {
    Err(BasicTextError::OutOfBand)
}

#[cold]
fn bom() -> Result<(), BasicTextError> {
    Err(BasicTextError::UnneededBOM)
}

#[cold]
fn interlinear_annotation() -> Result<(), BasicTextError> {
    Err(BasicTextError::Interlinear)
}
