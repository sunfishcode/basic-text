use crate::{BasicTextError, Categorize, IsolateUnassigned};
use std::cell::RefCell;
use std::rc::Rc;

/// An extension crate providing iterator transforms useful before
/// normalization.
pub trait PreNormalization<I: Iterator<Item = char>> {
    /// Return an iterator which detects some sequences which are not valid in
    /// Basic Text.
    fn categorize(self, error: Rc<RefCell<Option<BasicTextError>>>) -> Categorize<I>;

    /// Return an iterator that inserts CGJs around unassigned scalar values to
    /// protect them from future normalization.
    fn isolate_unassigned(self) -> IsolateUnassigned<I>;
}

impl<I: Iterator<Item = char>> PreNormalization<I> for I {
    #[inline]
    fn categorize(self, error: Rc<RefCell<Option<BasicTextError>>>) -> Categorize<I> {
        Categorize::new(self, error)
    }

    #[inline]
    fn isolate_unassigned(self) -> IsolateUnassigned<I> {
        IsolateUnassigned::new(self)
    }
}
