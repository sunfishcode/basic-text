use crate::{unicode::REPL, Utf8Interactor, Utf8Reader};
use interact_trait::InteractExt;
#[cfg(can_vector)]
use io_ext::default_is_read_vectored;
use io_ext::{
    default_read, default_read_exact, default_read_to_end, default_read_to_string,
    default_read_vectored, ReadExt, Status,
};
use std::{
    cmp::{max, min},
    io::{self, copy, repeat, Cursor, Read},
    str,
};

pub(crate) trait Utf8ReaderInternals<Inner: ReadExt>: ReadExt {
    fn impl_(&mut self) -> &mut Utf8Input;
    fn inner(&self) -> &Inner;
    fn inner_mut(&mut self) -> &mut Inner;
}

impl<Inner: ReadExt> Utf8ReaderInternals<Inner> for Utf8Reader<Inner> {
    fn impl_(&mut self) -> &mut Utf8Input {
        &mut self.input
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

impl<Inner: InteractExt> Utf8ReaderInternals<Inner> for Utf8Interactor<Inner> {
    fn impl_(&mut self) -> &mut Utf8Input {
        &mut self.input
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

pub(crate) struct Utf8Input {
    /// A queue of bytes which have not been read but which have not been
    /// translated into the output yet.
    overflow: Vec<u8>,
}

impl Utf8Input {
    /// Construct a new instance of `Utf8Input`.
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            overflow: Vec::new(),
        }
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    #[inline]
    pub(crate) fn read_str<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<(usize, Status)> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        let (size, status) = internals.read_with_status(unsafe { buf.as_bytes_mut() })?;

        debug_assert!(buf.is_char_boundary(size));

        Ok((size, status))
    }

    /// Like `read_exact` but produces the result in a `str`.
    #[inline]
    pub(crate) fn read_exact_str<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<()> {
        // Safety: This is a UTF-8 stream so we can read directly into a `str`.
        internals.read_exact(unsafe { buf.as_bytes_mut() })
    }

    pub(crate) fn read_with_status<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<(usize, Status)> {
        // To ensure we can always make progress, callers should always use a
        // buffer of at least 4 bytes.
        if buf.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "buffer for reading from Utf8Reader must be at least 4 bytes long",
            ));
        }

        let mut nread = 0;

        if !internals.impl_().overflow.is_empty() {
            nread += internals
                .impl_()
                .process_overflow(&mut buf[nread..], IncompleteHow::Include)
                .unwrap();
            if !internals.impl_().overflow.is_empty() {
                return Ok((nread, Status::active()));
            }
        }

        let (size, status) = internals.inner_mut().read_with_status(&mut buf[nread..])?;
        nread += size;

        // We may have overwritten part of a codepoint; overwrite the rest of
        // the buffer.
        // TODO: Use [`fill`] when it becomes available:
        // https://doc.rust-lang.org/std/primitive.slice.html#method.fillbbbb
        copy(
            &mut repeat(b'\0').take((buf.len() - nread) as u64),
            &mut Cursor::new(&mut buf[nread..]),
        )
        .unwrap();

        match str::from_utf8(&buf[..nread]) {
            Ok(_) => Ok((nread, status)),
            Err(error) => {
                let (valid, after_valid) = buf[..nread].split_at(error.valid_up_to());
                nread = valid.len();

                assert!(internals.impl_().overflow.is_empty());
                internals.impl_().overflow.extend_from_slice(after_valid);

                let incomplete_how = if status.is_end() {
                    IncompleteHow::Replace
                } else {
                    IncompleteHow::Exclude
                };
                nread += internals
                    .impl_()
                    .process_overflow(&mut buf[nread..], incomplete_how)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid UTF-8"))?;
                if internals.impl_().overflow.is_empty() {
                    Ok((nread, status))
                } else {
                    Ok((nread, Status::active()))
                }
            }
        }
    }

    #[inline]
    pub(crate) fn minimum_buffer_size<Inner: ReadExt>(
        internals: &impl Utf8ReaderInternals<Inner>,
    ) -> usize {
        max(4, internals.inner().minimum_buffer_size())
    }

    /// If normal reading encounters invalid bytes, the data is copied into
    /// `internals.impl_().overflow` as it may need to expand to make room for
    /// the U+FFFD's, and we may need to hold on to some of it until the next
    /// `read` call.
    ///
    /// TODO: This code could be significantly optimized.
    #[cold]
    fn process_overflow(&mut self, buf: &mut [u8], incomplete_how: IncompleteHow) -> Option<usize> {
        let mut nread = 0;

        loop {
            let num = min(buf[nread..].len(), self.overflow.len());
            match str::from_utf8(&self.overflow[..num]) {
                Ok(_) => {
                    buf[nread..nread + num].copy_from_slice(&self.overflow[..num]);
                    self.overflow.copy_within(num.., 0);
                    self.overflow.resize(self.overflow.len() - num, 0);
                    nread += num;
                }
                Err(error) => {
                    let (valid, after_valid) = self.overflow[..num].split_at(error.valid_up_to());
                    let valid_len = valid.len();
                    let after_valid_len = after_valid.len();
                    buf[nread..nread + valid_len].copy_from_slice(valid);
                    self.overflow.copy_within(valid_len.., 0);
                    self.overflow.resize(self.overflow.len() - valid_len, 0);
                    nread += valid_len;

                    if let Some(invalid_sequence_length) = error.error_len() {
                        if REPL.len_utf8() <= buf[nread..].len() {
                            nread += REPL.encode_utf8(&mut buf[nread..]).len();
                            self.overflow.copy_within(invalid_sequence_length.., 0);
                            self.overflow
                                .resize(self.overflow.len() - invalid_sequence_length, 0);
                            continue;
                        }
                    } else {
                        match incomplete_how {
                            IncompleteHow::Replace => {
                                if REPL.len_utf8() <= buf[nread..].len() {
                                    nread += REPL.encode_utf8(&mut buf[nread..]).len();
                                    self.overflow.clear();
                                } else if self.overflow.is_empty() {
                                    return None;
                                }
                            }
                            IncompleteHow::Include if after_valid_len == self.overflow.len() => {
                                if !buf[nread..].is_empty() {
                                    let num = min(buf[nread..].len(), after_valid_len);
                                    buf[nread..nread + num].copy_from_slice(&self.overflow[..num]);
                                    nread += num;
                                    self.overflow.copy_within(num.., 0);
                                    self.overflow.resize(self.overflow.len() - num, 0);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            break;
        }

        Some(nread)
    }

    #[inline]
    pub(crate) fn abandon<Inner: ReadExt>(internals: &mut impl Utf8ReaderInternals<Inner>) {
        internals.impl_().overflow.clear();
        internals.inner_mut().abandon()
    }

    #[inline]
    pub(crate) fn suggested_buffer_size<Inner: ReadExt>(
        internals: &impl Utf8ReaderInternals<Inner>,
    ) -> usize {
        max(
            Self::minimum_buffer_size(internals),
            internals.inner().suggested_buffer_size(),
        )
    }

    #[inline]
    pub(crate) fn read<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        default_read(internals, buf)
    }

    #[inline]
    pub(crate) fn read_vectored<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        bufs: &mut [io::IoSliceMut<'_>],
    ) -> io::Result<usize> {
        default_read_vectored(internals, bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    pub(crate) fn is_read_vectored<Inner: ReadExt>(
        internals: &impl Utf8ReaderInternals<Inner>,
    ) -> bool {
        default_is_read_vectored(internals)
    }

    #[inline]
    pub(crate) fn read_to_end<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut Vec<u8>,
    ) -> io::Result<usize> {
        default_read_to_end(internals, buf)
    }

    #[inline]
    pub(crate) fn read_to_string<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut String,
    ) -> io::Result<usize> {
        default_read_to_string(internals, buf)
    }

    #[inline]
    pub(crate) fn read_exact<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut [u8],
    ) -> io::Result<()> {
        default_read_exact(internals, buf)
    }
}

/// What to do when there is an incomplete UTF-8 sequence at the end of
/// the overflow buffer.
enum IncompleteHow {
    /// Include the incomplete sequence in the output.
    Include,
    /// Leave the incomplete sequence in the overflow buffer.
    Exclude,
    /// Replace the incomplete sequence with U+FFFD.
    Replace,
}
