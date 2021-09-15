//! On output, several disallowed scalar values are rejected, to catch
//! applications attempting to use them.

use crate::unicode::SUB;
use crate::{check_basic_text_char, BasicTextError};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Categorize<Iter: Iterator<Item = char>> {
    iter: Iter,

    // Because we wrap this iterator in the NFC etc. iterator chain, it has
    // to yield `char`s, and can't directly return errors. We indicate errors
    // by returning the special `SUB` value, which we intercept on the other
    // side to report the error stored in this error field.
    error: Rc<RefCell<Option<BasicTextError>>>,
}

impl<Iter: Iterator<Item = char>> Categorize<Iter> {
    #[inline]
    pub fn new(iter: Iter, error: Rc<RefCell<Option<BasicTextError>>>) -> Self {
        Self { iter, error }
    }

    #[cold]
    fn record_error(&mut self, error: BasicTextError) -> char {
        *self.error.borrow_mut() = Some(error);
        SUB
    }
}

impl<Iter: Iterator<Item = char>> Iterator for Categorize<Iter> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.iter.next()?;
        Some(match check_basic_text_char(c) {
            Ok(()) => c,
            Err(e) => self.record_error(e),
        })
    }
}
