//! On output, several disallowed scalar values are rejected, to catch
//! applications attempting to use them.

use crate::{
    replace,
    unicode::{BOM, ESC, ORC},
};
use std::collections::VecDeque;
use thiserror::Error;

/// Test whether the given Unicode scalar value is valid in a Basic Text string.
#[inline]
pub fn check_basic_text_char(c: char) -> Result<(), BasicTextError> {
    match c {
        // Newline and tab are allowed, and escape is handled specially.
        c if c.is_control() && c != '\n' && c != '\t' && c != ESC => control(c),
        c @ '\u{149}'
        | c @ '\u{673}'
        | c @ '\u{f77}'
        | c @ '\u{f79}'
        | c @ '\u{17a3}'
        | c @ '\u{17a4}'
        | c @ '\u{2329}'
        | c @ '\u{232a}'
        | c @ '\u{2126}'
        | c @ '\u{212a}'
        | c @ '\u{212b}'
        | c @ '\u{2df5}'
        | c @ '\u{111c4}'
        | c @ '\u{fb00}'..='\u{fb06}'
        | c @ '\u{9e4}'
        | c @ '\u{9e5}'
        | c @ '\u{a64}'
        | c @ '\u{a65}'
        | c @ '\u{ae4}'
        | c @ '\u{ae5}'
        | c @ '\u{b64}'
        | c @ '\u{b65}'
        | c @ '\u{be4}'
        | c @ '\u{be5}'
        | c @ '\u{c64}'
        | c @ '\u{c65}'
        | c @ '\u{ce4}'
        | c @ '\u{ce5}'
        | c @ '\u{d64}'
        | c @ '\u{d65}'
        | c @ '\u{2072}'
        | c @ '\u{2073}'
        | c @ '\u{1d455}'
        | c @ '\u{1d49d}'
        | c @ '\u{1d4a0}'
        | c @ '\u{1d4a1}'
        | c @ '\u{1d4a3}'
        | c @ '\u{1d4a4}'
        | c @ '\u{1d4a7}'
        | c @ '\u{1d4a8}'
        | c @ '\u{1d4ad}'
        | c @ '\u{1d4ba}'
        | c @ '\u{1d4bc}'
        | c @ '\u{1d4c4}'
        | c @ '\u{1d506}'
        | c @ '\u{1d50b}'
        | c @ '\u{1d50c}'
        | c @ '\u{1d515}'
        | c @ '\u{1d51d}'
        | c @ '\u{1d53a}'
        | c @ '\u{1d53f}'
        | c @ '\u{1d545}'
        | c @ '\u{1d547}'
        | c @ '\u{1d548}'
        | c @ '\u{1d549}'
        | c @ '\u{1d551}' => replacement(c),
        '\u{e0001}' => language_tag(),
        '\u{fff9}'..='\u{fffb}' => interlinear_annotation(),
        c @ '\u{17b4}' | c @ '\u{17b5}' | c @ '\u{17d8}' => discouraged(c),
        c @ '\u{206a}'..='\u{206f}' => deprecated_format_character(c),
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
    #[error("Unicode discourages use of {0:?}")]
    Discouraged(char),
    #[error("Unrecognized escape sequence")]
    UnrecognizedEscape,
    #[error("Use Standardized Variants instead of CJK Compatibility Ideographs")]
    CJKCompat,
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
fn discouraged(c: char) -> Result<(), BasicTextError> {
    Err(BasicTextError::Discouraged(c))
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
