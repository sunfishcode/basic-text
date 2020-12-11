//! Defines `RcCharQueue` and `RcCharQueueIter`.

use crate::unicode::CGJ;
use std::{cell::RefCell, collections::vec_deque::VecDeque, rc::Rc};

/// A queue of `char`s held by an `Rc<RefCell<...>>` so that we can insert
/// bytes into the queue while holding an iterator to it.
pub(crate) struct RcCharQueue {
    // Contains a tuple of:
    //  - The queue.
    //  - The number of newline or CGJ scalar values in the queue. These scalar
    //    values reset the NFC algorithm, so if we know there are some present
    //    in the queue, we can let the NFC algorithm run without fear of it
    //    hitting the end of the queue.
    queue: Rc<RefCell<VecDeque<char>>>,
    has_reset: bool,
}

impl RcCharQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
            has_reset: false,
        }
    }

    pub(crate) fn push(&mut self, c: char) {
        self.has_reset = matches!(c, '\n' | CGJ);
        self.queue.borrow_mut().push_back(c)
    }

    pub(crate) fn len(&self) -> usize {
        self.queue.borrow().len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    pub(crate) fn has_reset(&self) -> bool {
        self.has_reset
    }

    pub(crate) fn iter(&self) -> RcCharQueueIter {
        RcCharQueueIter::new(Rc::clone(&self.queue))
    }
}

/// An iterator over the chars in an `RcCharQueue`.
pub(crate) struct RcCharQueueIter {
    queue: Rc<RefCell<VecDeque<char>>>,
}

impl RcCharQueueIter {
    pub(crate) fn new(queue: Rc<RefCell<VecDeque<char>>>) -> Self {
        Self { queue }
    }
}

impl Iterator for RcCharQueueIter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.borrow_mut().pop_front()
    }
}
