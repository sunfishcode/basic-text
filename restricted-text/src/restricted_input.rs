//! Input for `RestrictedReader` and the reader half of `RestrictedDuplexer`.

use crate::{
    RestrictedDuplexer, RestrictedReader, RestrictedStr,
};
use layered_io::{default_read, HalfDuplexLayered, Status, WriteLayered};
use std::{
    cmp::max,
    collections::{vec_deque, VecDeque},
    io::{self, copy, repeat, Cursor, Read},
    mem::take, str,
};
use unicode_normalization::{Recompositions, UnicodeNormalization};
use utf8_io::{ReadStrLayered, WriteStr};
use basic_text::NORMALIZATION_BUFFER_SIZE;
use basic_text::TextStr;

pub(crate) trait RestrictedReaderInternals<Inner: ReadStrLayered>: ReadStrLayered {
    fn impl_(&mut self) -> &mut RestrictedInput;
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
    fn into_inner(self) -> Inner;
}

impl<Inner: ReadStrLayered> RestrictedReaderInternals<Inner> for RestrictedReader<Inner> {
    fn impl_(&mut self) -> &mut RestrictedInput {
        &mut self.input
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }
}

impl<Inner: HalfDuplexLayered + ReadStrLayered + WriteStr + WriteLayered> RestrictedReaderInternals<Inner>
    for RestrictedDuplexer<Inner>
{
    fn impl_(&mut self) -> &mut RestrictedInput {
        &mut self.input
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Inner {
        self.inner
    }
}

pub(crate) struct RestrictedInput {
    /// Temporary storage for reading scalar values from the underlying stream.
    raw_string: String,

    /// A queue of scalar values which have been translated but not written to
    /// the output yet.
    /// TODO: This is awkward; what we really want here is a streaming stream-safe
    /// and NFKC translator.
    queue: VecDeque<char>,

    /// An iterator over the chars in `self.queue`.
    queue_iter: Recompositions<vec_deque::IntoIter<char>>,

    /// When we can't fit all the data from an underlying read in our buffer,
    /// we buffer it up. Remember the status value so we can replay that too.
    pending_status: Status,
}

impl RestrictedInput {
    /// Construct a new instance of `RestrictedInput`.
    #[inline]
    pub(crate) fn new() -> Self {
        let queue = VecDeque::new();
        Self {
            raw_string: String::new(),
            queue,
            queue_iter: VecDeque::<char>::new().into_iter().nfkc(),
        }
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[inline]
    pub(crate) fn read_str<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<usize> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_exact` but produces the result in a `str`.
    #[inline]
    pub(crate) fn read_exact_str<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<()> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read_exact(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[inline]
    pub(crate) fn read_str_with_status<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<(usize, Status)> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        let (size, status) = internals.read_with_status(unsafe { buf.as_bytes_mut() })?;

        debug_assert!(buf.is_char_boundary(size));

        Ok((size, status))
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[inline]
    pub(crate) fn read_exact_str_using_status<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<Status> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read_exact_using_status(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_with_status` but produces the result in a `TextStr`. Be sure
    /// to check the `size` field of the return value to see how many bytes
    /// were written.
    #[inline]
    pub(crate) fn read_text<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut TextStr,
    ) -> io::Result<usize> {
        // Safety: This is a UTF-8 stream so we can read directly into a
        // `TextStr`.
        internals.read(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_exact` but produces the result in a `TextStr`.
    #[inline]
    pub(crate) fn read_exact_text<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut TextStr,
    ) -> io::Result<()> {
        // Safety: This is a Text stream so we can read directly into a
        // `TextStr`.
        internals.read_exact(unsafe { buf.as_bytes_mut() })
    }

    /// Like `read_with_status` but produces the result in a `TextStr`. Be sure
    /// to check the `size` field of the return value to see how many bytes
    /// were written.
    #[inline]
    pub(crate) fn read_text_with_status<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut TextStr,
    ) -> io::Result<(usize, Status)> {
        // Safety: This is a text stream so we can read directly into a `TextStr`.
        let (size, status) = internals.read_with_status(unsafe { buf.as_bytes_mut() })?;

        // TODO
        //debug_assert!(buf.is_char_boundary(size));

        Ok((size, status))
    }

    /// Like `read_with_status` but produces the result in a `TextStr`. Be sure
    /// to check the `size` field of the return value to see how many bytes
    /// were written.
    #[inline]
    pub(crate) fn read_exact_text_using_status<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut TextStr,
    ) -> io::Result<Status> {
        // Safety: This is a text stream so we can read directly into a `str`.
        internals.read_exact_using_status(unsafe { buf.as_bytes_mut() })
    }

    fn queue_next(&mut self) -> Option<char> {
        match self.queue_iter.next() {
            Some(c) => Some(c),
            None => {
                let index = self.queue.iter().position(|c| matches!(*c, '\n'))?;
                let tmp = self.queue.drain(0..=index).collect::<VecDeque<char>>();
                self.queue_iter = tmp.into_iter().nfkc();
                self.queue_iter.next()
            }
        }
    }

    fn process_raw_string(&mut self) {
        for c in self.raw_string.chars() {
            // TODO: Implement Restricted Text
            self.queue.push_back(c);
        }
    }

    pub(crate) fn read_with_status<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<(usize, Status)> {
        if buf.len() < NORMALIZATION_BUFFER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("buffer for text input is {} bytes; at least NORMALIZATION_BUFFER_SIZE bytes are required", buf.len())
            ));
        }

        let mut nread = 0;

        loop {
            match internals.impl_().queue_next() {
                Some(c) => nread += c.encode_utf8(&mut buf[nread..]).len(),
                None => break,
            }
            if nread == buf.len() {
                return Ok((nread, Status::active()));
            }
        }
        if internals.impl_().pending_status != Status::active() {
            internals.impl_().pending_status = Status::active();
            internals.impl_().expect_starter = true;

            // We may have overwritten part of a codepoint; overwrite the rest
            // of the buffer.
            // TODO: Use [`fill`] when it becomes available:
            // https://doc.rust-lang.org/std/primitive.slice.html#method.fill
            copy(
                &mut repeat(b'?').take((buf.len() - nread) as u64),
                &mut Cursor::new(&mut buf[nread..]),
            )
            .unwrap();

            return Ok((nread, internals.impl_().pending_status));
        }

        let mut raw_bytes =
            take(&mut internals.impl_().raw_string).into_bytes();
        raw_bytes.resize(4096, 0_u8);
        let (size, status) = internals.inner_mut().read_with_status(&mut raw_bytes)?;
        raw_bytes.resize(size, 0);
        // Safety: This is a UTF-8 stream so we can read into a `String`.
        internals.impl_().raw_string = unsafe { String::from_utf8_unchecked(raw_bytes) };

        internals.impl_().process_raw_string();

        let mut queue_empty = false;
        loop {
            match internals.impl_().queue_next() {
                Some(c) => nread += c.encode_utf8(&mut buf[nread..]).len(),
                None => {
                    queue_empty = true;
                    break;
                }
            }
            if nread == buf.len() {
                break;
            }
        }

        // We may have overwritten part of a codepoint; overwrite the rest
        // of the buffer.
        copy(
            &mut repeat(b'?').take((buf.len() - nread) as u64),
            &mut Cursor::new(&mut buf[nread..]),
        )
        .unwrap();

        Ok((
            nread,
            if queue_empty {
                if status != Status::active() {
                    internals.impl_().expect_starter = true;
                }
                status
            } else {
                internals.impl_().pending_status = status;
                Status::active()
            },
        ))
    }

    #[inline]
    pub(crate) fn minimum_buffer_size<Inner: ReadStrLayered>(
        internals: &impl RestrictedReaderInternals<Inner>,
    ) -> usize {
        max(
            NORMALIZATION_BUFFER_SIZE,
            internals.inner().minimum_buffer_size(),
        )
    }

    #[inline]
    pub(crate) fn abandon<Inner: ReadStrLayered>(internals: &mut impl RestrictedReaderInternals<Inner>) {
        assert!(internals.impl_().queue.is_empty());

        internals.inner_mut().abandon()
    }

    #[inline]
    pub(crate) fn suggested_buffer_size<Inner: ReadStrLayered>(
        internals: &impl RestrictedReaderInternals<Inner>,
    ) -> usize {
        max(
            Self::minimum_buffer_size(internals),
            internals.inner().suggested_buffer_size(),
        )
    }

    #[inline]
    pub(crate) fn read<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        default_read(internals, buf)
    }

    #[inline]
    pub(crate) fn read_to_string<Inner: ReadStrLayered>(
        internals: &mut impl RestrictedReaderInternals<Inner>,
        buf: &mut String,
    ) -> io::Result<usize> {
        // Safety: This is a UTF-8 stream so we can read into a `String`.
        internals.read_to_end(unsafe { buf.as_mut_vec() })
    }
}
