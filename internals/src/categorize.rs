//! On output, several disallowed scalar values are rejected, to catch
//! applications attempting to use them.

use crate::unicode::{BOM, ESC, ORC, SUB};
use std::{cell::RefCell, io, rc::Rc};

pub struct Categorize<Iter: Iterator<Item = char>> {
    iter: Iter,

    // Because we wrap this iterator in the NFC etc. iterator chain, it has
    // to yield `char`s, and can't directly return errors. We indicate errors
    // by returning the special `SUB` value, which we intercept on the other
    // side to report the error stored in this error field.
    error: Rc<RefCell<Option<io::Error>>>,
}

impl<Iter: Iterator<Item = char>> Categorize<Iter> {
    #[inline]
    pub fn new(iter: Iter, error: Rc<RefCell<Option<io::Error>>>) -> Self {
        Self { iter, error }
    }

    fn record_error(&mut self, error: io::Error) -> char {
        *self.error.borrow_mut() = Some(error);
        SUB
    }
}

impl<Iter: Iterator<Item = char>> Iterator for Categorize<Iter> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.iter.next()? {
            // Newline and tab are allowed, and escape is handled specially.
            c if c.is_control() && c != '\n' && c != '\t' && c != ESC => self.control(c),
            c @ '\u{149}'
            | c @ '\u{673}'
            | c @ '\u{f77}'
            | c @ '\u{f79}'
            | c @ '\u{17a3}'
            | c @ '\u{17a4}'
            | c @ '\u{2329}'
            | c @ '\u{232a}'
            | c @ '\u{e0001}' => self.deprecated(c),
            c @ '\u{2126}' | c @ '\u{212a}' | c @ '\u{212b}' => self.obsolete_compatibility(c),
            // Interlinear Annotations
            '\u{fff9}'..='\u{fffb}' => self.interlinear_annotation(),
            // Khmer characters erroneously invented by Unicode.
            c @ '\u{17b4}' | c @ '\u{17b5}' | c @ '\u{17d8}' => self.erroneous_khmer_character(c),
            // Deprecated Format Characters
            c @ '\u{206a}'..='\u{206f}' => self.deprecated_format_character(c),
            // Tag Characters
            c @ '\u{e0000}'..='\u{e007f}' => self.tag_character(c),
            // Discouraged Characters
            c @ '\u{2df5}' | c @ '\u{111c4}' => self.discouraged_character(c),
            // Latin Ligatures
            c @ '\u{fb00}' ..= '\u{fb06}' => self.latin_ligature(c),
            // Markup Characters
            c @ '\u{2028}' | c @ '\u{2029}' => self.markup_character(c),
            // Bidirectional Formatting Characters
            c @ '\u{202a}' |
            c @ '\u{202b}' |
            c @ '\u{202c}' |
            c @ '\u{202d}' |
            c @ '\u{202e}' |
            c @ '\u{2066}' |
            c @ '\u{2067}' |
            c @ '\u{2068}' |
            c @ '\u{2069}' => self.bidirectional_formatting_character(c),
            // Noncharacters
            c @ '\u{fffe}'..='\u{ffff}'
            | c @ '\u{1fffe}'..='\u{1ffff}'
            | c @ '\u{2fffe}'..='\u{2ffff}'
            | c @ '\u{3fffe}'..='\u{3ffff}'
            | c @ '\u{4fffe}'..='\u{4ffff}'
            | c @ '\u{5fffe}'..='\u{5ffff}'
            | c @ '\u{6fffe}'..='\u{6ffff}'
            | c @ '\u{7fffe}'..='\u{7ffff}'
            | c @ '\u{8fffe}'..='\u{8ffff}'
            | c @ '\u{9fffe}'..='\u{9ffff}'
            | c @ '\u{afffe}'..='\u{affff}'
            | c @ '\u{bfffe}'..='\u{bffff}'
            | c @ '\u{cfffe}'..='\u{cffff}'
            | c @ '\u{dfffe}'..='\u{dffff}'
            | c @ '\u{efffe}'..='\u{effff}'
            | c @ '\u{ffffe}'..='\u{fffff}'
            | c @ '\u{10fffe}'..='\u{10ffff}'
            | c @ '\u{fdd0}'..='\u{fdef}' => self.noncharacter(c),
            // Unassigned characters with replacements.
            c @ '\u{9e4}' | c @ '\u{9e5}' | c @ '\u{a64}' | c @ '\u{a65}'
            | c @ '\u{ae4}' | c @ '\u{ae5}' | c @ '\u{b64}' | c @ '\u{b65}'
            | c @ '\u{be4}' | c @ '\u{be5}' | c @ '\u{c64}' | c @ '\u{c65}'
            | c @ '\u{ce4}' | c @ '\u{ce5}' | c @ '\u{d64}' | c @ '\u{d65}'
            // Unassigned characters with replacements.
            | c @ '\u{2072}' | c @ '\u{2073}'
            // Unassigned alphanumeric mathematical symbols.
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
            | c @ '\u{1d551}' => self.unassigned_with_replacement(c),
            ORC => self.orc(),
            BOM => self.bom(),
            c => c,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Propagate the inner interator's lower bound.
        self.iter.size_hint()
    }
}

impl<Iter: Iterator<Item = char>> Categorize<Iter> {
    #[cold]
    fn control(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!("Control code written to text output stream: {:?}", c),
        ))
    }

    #[cold]
    fn deprecated(&mut self, c: char) -> char {
        let replacement = match c {
            '\u{149}' => Some("\u{2bc}\u{6e}"),
            '\u{673}' => Some("\u{627}\u{65f}"),
            '\u{f77}' => Some("\u{fb2}\u{f81}"),
            '\u{f79}' => Some("\u{fb3}\u{f81}"),
            '\u{17a3}' => Some("\u{17a2}"),
            '\u{17a4}' => Some("\u{17a2}\u{17b6}"),
            _ => None,
        };
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            match replacement {
                Some(replacement) => format!(
                    "Deprecated character written to text output stream: {:?}; use {:?} instead",
                    c, replacement
                ),
                None => format!(
                    "Deprecated character written to text output stream: {:?}",
                    c
                ),
            },
        ))
    }

    #[cold]
    fn obsolete_compatibility(&mut self, c: char) -> char {
        let replacement = match c {
            '\u{2126}' => "\u{3a9}",
            '\u{212a}' => "\u{4b}",
            '\u{212b}' => "\u{c5}",
            _ => panic!(),
        };
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Obsolete compatibility written to text output stream: {:?}; use {:?} instead",
                c, replacement
            ),
        ))
    }

    #[cold]
    fn erroneous_khmer_character(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Erroneous Khmer character written to text output stream: {:?}",
                c
            ),
        ))
    }

    #[cold]
    fn deprecated_format_character(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Deprecated format character written to text output stream: {:?}",
                c
            ),
        ))
    }

    #[cold]
    fn tag_character(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!("Tag character written to text output stream: {:?}", c),
        ))
    }

    #[cold]
    fn markup_character(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!("Markup character written to text output stream: {:?}", c),
        ))
    }

    #[cold]
    fn bidirectional_formatting_character(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Bidirectional formatting character written to text output stream: {:?}",
                c
            ),
        ))
    }

    #[cold]
    fn discouraged_character(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Discouraged character written to text output stream: {:?}",
                c
            ),
        ))
    }

    #[cold]
    fn latin_ligature(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!("Latin ligature written to text output stream: {:?}", c),
        ))
    }

    #[cold]
    fn noncharacter(&mut self, c: char) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!("Noncharacter written to text output stream: {:?}", c),
        ))
    }

    #[cold]
    fn unassigned_with_replacement(&mut self, c: char) -> char {
        let replacement = match c {
            '\u{9e4}' => "\u{964}",
            '\u{9e5}' => "\u{965}",
            '\u{a64}' => "\u{964}",
            '\u{a65}' => "\u{965}",
            '\u{ae4}' => "\u{964}",
            '\u{ae5}' => "\u{965}",
            '\u{b64}' => "\u{964}",
            '\u{b65}' => "\u{965}",
            '\u{be4}' => "\u{964}",
            '\u{be5}' => "\u{965}",
            '\u{c64}' => "\u{964}",
            '\u{c65}' => "\u{965}",
            '\u{ce4}' => "\u{964}",
            '\u{ce5}' => "\u{965}",
            '\u{d64}' => "\u{964}",
            '\u{d65}' => "\u{965}",
            '\u{2072}' => "\u{b2}",
            '\u{2073}' => "\u{b3}",
            '\u{1d455}' => "\u{210e}",
            '\u{1d49d}' => "\u{212c}",
            '\u{1d4a0}' => "\u{2130}",
            '\u{1d4a1}' => "\u{2131}",
            '\u{1d4a3}' => "\u{210b}",
            '\u{1d4a4}' => "\u{2110}",
            '\u{1d4a7}' => "\u{2112}",
            '\u{1d4a8}' => "\u{2133}",
            '\u{1d4ad}' => "\u{211b}",
            '\u{1d4ba}' => "\u{212f}",
            '\u{1d4bc}' => "\u{210a}",
            '\u{1d4c4}' => "\u{2134}",
            '\u{1d506}' => "\u{212d}",
            '\u{1d50b}' => "\u{210c}",
            '\u{1d50c}' => "\u{2111}",
            '\u{1d515}' => "\u{211c}",
            '\u{1d51d}' => "\u{2128}",
            '\u{1d53a}' => "\u{2102}",
            '\u{1d53f}' => "\u{210d}",
            '\u{1d545}' => "\u{2115}",
            '\u{1d547}' => "\u{2119}",
            '\u{1d548}' => "\u{211a}",
            '\u{1d549}' => "\u{211d}",
            '\u{1d551}' => "\u{2124}",
            _ => panic!("ff [{:?}", c),
        };
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Unassigned character written to text output stream: {:?}; use {:?} instead",
                c, replacement
            ),
        ))
    }

    #[cold]
    fn orc(&mut self) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            "Object replacement character written to text output stream",
        ))
    }

    #[cold]
    fn bom(&mut self) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            "Byte-order mark written to text output stream",
        ))
    }

    #[cold]
    fn interlinear_annotation(&mut self) -> char {
        self.record_error(io::Error::new(
            io::ErrorKind::Other,
            "Interlinear annotation written to text output stream",
        ))
    }
}
