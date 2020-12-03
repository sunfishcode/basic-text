//! Defines `RcCharQueue` and `RcCharQueueIter`.

use std::{cell::RefCell, collections::vec_deque::VecDeque, rc::Rc};

/// A queue of `char`s held by an `Rc<RefCell<...>>` so that we can insert
/// bytes into the queue while holding an iterator to it.
pub(crate) struct RcCharQueue {
    queue: Rc<RefCell<VecDeque<char>>>,
}

impl RcCharQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub(crate) fn push(&mut self, c: char) {
        self.queue.borrow_mut().push_back(c)
    }

    pub(crate) fn len(&self) -> usize {
        self.queue.borrow().len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
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
