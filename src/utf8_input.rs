use crate::{unicode::REPL, Utf8Reader, Utf8ReaderWriter};
use io_ext::{
    default_read, default_read_exact, default_read_to_end, default_read_to_string,
    default_read_vectored, ReadExt, ReadWriteExt, Status,
};
use std::{cmp::min, io, str};

pub(crate) trait Utf8ReaderInternals<Inner: ReadExt>: ReadExt {
    fn impl_(&mut self) -> &mut Utf8Input;
    fn inner(&mut self) -> &mut Inner;
}

impl<Inner: ReadExt> Utf8ReaderInternals<Inner> for Utf8Reader<Inner> {
    fn impl_(&mut self) -> &mut Utf8Input {
        &mut self.impl_
    }

    fn inner(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

impl<Inner: ReadWriteExt> Utf8ReaderInternals<Inner> for Utf8ReaderWriter<Inner> {
    fn impl_(&mut self) -> &mut Utf8Input {
        &mut self.input
    }

    fn inner(&mut self) -> &mut Inner {
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
    pub(crate) fn new() -> Self {
        Self {
            overflow: Vec::new(),
        }
    }

    /// Like `read_with_status` but produces the result in a `str`. Be sure to
    /// check the `size` field of the return value to see how many bytes were
    /// written.
    pub(crate) fn read_str<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<(usize, Status)> {
        let size_and_status = unsafe { internals.read_with_status(buf.as_bytes_mut()) }?;

        debug_assert!(buf.is_char_boundary(size_and_status.0));

        Ok(size_and_status)
    }

    /// Like `read_exact` but produces the result in a `str`.
    pub(crate) fn read_exact_str<Inner: ReadExt>(
        internals: &mut impl Utf8ReaderInternals<Inner>,
        buf: &mut str,
    ) -> io::Result<()> {
        unsafe { internals.read_exact(buf.as_bytes_mut()) }
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

        let (size, status) = internals.inner().read_with_status(&mut buf[nread..])?;
        nread += size;

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

    #[cfg(feature = "nightly")]
    #[inline]
    pub(crate) fn is_read_vectored<Inner: ReadExt>(
        &internals: &Utf8ReaderInternals<Inner>,
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
