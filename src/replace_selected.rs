//! On input, several disallowed scalar values are replaced, so that content
//! containing them can still be read, but applications don't have to
//! handle them.

use crate::unicode::{BOM, LS, ORC, PS, REPL, WJ};

/// An iterator over `char`s which replaces occurrences of
/// characters that have replacement sequences.
pub(crate) struct ReplaceSelected<Inner: Iterator<Item = char>> {
    inner: Inner,

    /// At this time, the longest replacement sequence is 3 USVs,
    /// so we need at most two in the buffer.
    buffer: Option<char>,
    second_buffer: Option<char>,
}

impl<Inner: Iterator<Item = char>> ReplaceSelected<Inner> {
    #[inline]
    pub(crate) fn new(inner: Inner) -> Self {
        Self {
            inner,
            buffer: None,
            second_buffer: None,
        }
    }
}

impl<Inner: Iterator<Item = char>> Iterator for ReplaceSelected<Inner> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.buffer.take() {
            return Some(c);
        }
        if let Some(c) = self.second_buffer.take() {
            return Some(c);
        }

        match self.inner.next()? {
            BOM => Some(WJ),
            '\u{149}' => {
                self.buffer = Some('\u{6e}');
                Some('\u{2bc}')
            }
            '\u{673}' => {
                self.buffer = Some('\u{65f}');
                Some('\u{627}')
            }
            '\u{f77}' => {
                self.buffer = Some('\u{f81}');
                Some('\u{fb2}')
            }
            '\u{f79}' => {
                self.buffer = Some('\u{f81}');
                Some('\u{fb3}')
            }
            '\u{17a3}' => Some('\u{17a2}'),
            '\u{17a4}' => {
                self.buffer = Some('\u{17b6}');
                Some('\u{17a2}')
            }
            // Discouraged characters
            '\u{2df5}' => {
                self.buffer = Some('\u{2dee}');
                Some('\u{2ded}')
            }
            '\u{111c4}' => {
                self.buffer = Some('\u{11180}');
                Some('\u{1118f}')
            }
            LS | PS => Some(' '),
            // Latin Ligatures
            '\u{fb00}' => {
                self.buffer = Some('f');
                Some('f')
            }
            '\u{fb01}' => {
                self.buffer = Some('i');
                Some('f')
            }
            '\u{fb02}' => {
                self.buffer = Some('l');
                Some('f')
            }
            '\u{fb03}' => {
                self.buffer = Some('f');
                self.second_buffer = Some('i');
                Some('f')
            }
            '\u{fb04}' => {
                self.buffer = Some('f');
                self.second_buffer = Some('l');
                Some('f')
            }
            '\u{fb05}' => {
                self.buffer = Some('t');
                Some('ſ')
            }
            '\u{fb06}' => {
                self.buffer = Some('t');
                Some('s')
            }
            // Interlinear Annotations
            '\u{fff9}'..='\u{fffb}' |
            // Unassigned characters with replacements.
            '\u{9e4}' | '\u{9e5}' | '\u{a64}' | '\u{a65}' |
            '\u{ae4}' | '\u{ae5}' | '\u{b64}' | '\u{b65}' |
            '\u{be4}' | '\u{be5}' | '\u{c64}' | '\u{c65}' |
            '\u{ce4}' | '\u{ce5}' | '\u{d64}' | '\u{d65}' |
            '\u{2072}' | '\u{2073}' |
            '\u{1d455}' | '\u{1d49d}' | '\u{1d4a0}' | '\u{1d4a1}' |
            '\u{1d4a3}' | '\u{1d4a4}' | '\u{1d4a7}' | '\u{1d4a8}' |
            '\u{1d4ad}' | '\u{1d4ba}' | '\u{1d4bc}' | '\u{1d4c4}' |
            '\u{1d506}' | '\u{1d50b}' | '\u{1d50c}' | '\u{1d515}' |
            '\u{1d51d}' | '\u{1d53a}' | '\u{1d53f}' | '\u{1d545}' |
            '\u{1d547}' | '\u{1d548}' | '\u{1d549}' | '\u{1d551}' |
            // Object Replacement Character
            ORC |
            // Khmer characters erroneously invented by Unicode.
            '\u{17b4}' | '\u{17b5}' | '\u{17d8}' |
            // Deprecated Format Characters
            '\u{206a}'..='\u{206f}' |
            // Tag Characters
            '\u{e0000}'..='\u{e007f}' |
            // Noncharacters
            '\u{fffe}' ..= '\u{ffff}' |
            '\u{1fffe}' ..= '\u{1ffff}' |
            '\u{2fffe}' ..= '\u{2ffff}' |
            '\u{3fffe}' ..= '\u{3ffff}' |
            '\u{4fffe}' ..= '\u{4ffff}' |
            '\u{5fffe}' ..= '\u{5ffff}' |
            '\u{6fffe}' ..= '\u{6ffff}' |
            '\u{7fffe}' ..= '\u{7ffff}' |
            '\u{8fffe}' ..= '\u{8ffff}' |
            '\u{9fffe}' ..= '\u{9ffff}' |
            '\u{afffe}' ..= '\u{affff}' |
            '\u{bfffe}' ..= '\u{bffff}' |
            '\u{cfffe}' ..= '\u{cffff}' |
            '\u{dfffe}' ..= '\u{dffff}' |
            '\u{efffe}' ..= '\u{effff}' |
            '\u{ffffe}' ..= '\u{fffff}' |
            '\u{10fffe}' ..= '\u{10ffff}' |
            '\u{fdd0}'..='\u{fdef}' => Some(REPL),

            c => Some(c),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Propagate the inner iterator's lower bound, but omit the upper bound
        // size we may use wider replacements.
        (self.inner.size_hint().0, None)
    }
}
