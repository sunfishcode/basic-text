//! On output, several disallowed scalar values are rejected, to catch
//! applications attempting to use them.

use crate::unicode::{BOM, ESC, ORC, SUB};
use std::{cell::RefCell, io, rc::Rc};

pub(crate) struct Categorize<Iter: Iterator<Item = char>> {
    iter: Iter,

    // Because we wrap this iterator in the NFC etc. iterator chain, it has
    // to yield `char`s, and can't directly return errors. We indicate errors
    // by returning the special `SUB` value, which we intercept on the other
    // side to report the error stored in this error field.
    error: Rc<RefCell<Option<io::Error>>>,
}

impl<Iter: Iterator<Item = char>> Categorize<Iter> {
    #[inline]
    pub(crate) fn new(iter: Iter, error: Rc<RefCell<Option<io::Error>>>) -> Self {
        Self { iter, error }
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
            // Unassigned alphanumeric mathematical symbols.
            c @ '\u{1d455}'
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
            | c @ '\u{1d551}' => self.unassigned_math(c),
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
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            format!("Control code written to text output stream: {:?}", c),
        ));
        SUB
    }

    #[cold]
    fn deprecated(&mut self, c: char) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Deprecated character written to text output stream: {:?}",
                c
            ),
        ));
        SUB
    }

    #[cold]
    fn obsolete_compatibility(&mut self, c: char) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Obsolete compatibility written to text output stream: {:?}",
                c
            ),
        ));
        SUB
    }

    #[cold]
    fn erroneous_khmer_character(&mut self, c: char) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Erroneous Khmer character written to text output stream: {:?}",
                c
            ),
        ));
        SUB
    }

    #[cold]
    fn deprecated_format_character(&mut self, c: char) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Deprecated format character written to text output stream: {:?}",
                c
            ),
        ));
        SUB
    }

    #[cold]
    fn tag_character(&mut self, c: char) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            format!("Tag character written to text output stream: {:?}", c),
        ));
        SUB
    }

    #[cold]
    fn noncharacter(&mut self, c: char) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            format!("Noncharacter written to text output stream: {:?}", c),
        ));
        SUB
    }

    #[cold]
    fn unassigned_math(&mut self, c: char) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Unassigned mathematical alphanumeric symbol written to text output stream: {:?}",
                c
            ),
        ));
        SUB
    }

    #[cold]
    fn orc(&mut self) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            "Object replacement character written to text output stream",
        ));
        SUB
    }

    #[cold]
    fn bom(&mut self) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            "Byte-order mark written to text output stream",
        ));
        SUB
    }

    #[cold]
    fn interlinear_annotation(&mut self) -> char {
        *self.error.borrow_mut() = Some(io::Error::new(
            io::ErrorKind::Other,
            "Interlinear annotation written to text output stream",
        ));
        SUB
    }
}
