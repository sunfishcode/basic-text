use crate::{text_utils::is_private_use_area, unicode::CGJ};
use std::iter::Fuse;
use unicode_normalization::char::is_public_assigned;

/// An iterator which inserts CGJ around unassigned scalar values.
pub struct IsolateUnassigned<I: Iterator<Item = char>> {
    iter: Fuse<I>,
    need_cgj: bool,
    have_cgj: bool,
    deferred: Option<char>,
}

impl<I: Iterator<Item = char>> IsolateUnassigned<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter: Iterator::fuse(iter),
            need_cgj: false,
            have_cgj: false,
            deferred: None,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for IsolateUnassigned<I> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        if let Some(deferred) = self.deferred.take() {
            return Some(deferred);
        }

        let c = self.iter.next();

        match c {
            Some(c) => {
                let assigned = is_public_assigned(c) || is_private_use_area(c);
                let have_cgj = c == CGJ;
                Some(
                    if (self.need_cgj && !have_cgj) || (!assigned && !self.have_cgj) {
                        self.need_cgj = !assigned;
                        self.have_cgj = false;
                        self.deferred = Some(c);
                        CGJ
                    } else {
                        self.need_cgj = false;
                        self.have_cgj = have_cgj;
                        c
                    },
                )
            }
            None => {
                if self.need_cgj {
                    self.need_cgj = false;
                    Some(CGJ)
                } else {
                    None
                }
            }
        }
    }
}
