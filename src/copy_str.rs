use crate::ReadStr;
use io_ext::{Bufferable, WriteExt};
use std::{cmp::max, io};

/// Like `std::io::copy`, but for streams that can operate directly on strings,
/// so we can avoid re-validating them as UTF-8.
pub fn copy_str<R: ReadStr + Bufferable + ?Sized, W: WriteExt + Bufferable + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> io::Result<u64> {
    // TODO: Avoid unnecessary zero-initialization.
    let mut buf = "\0".repeat(max(
        reader.suggested_buffer_size(),
        writer.suggested_buffer_size(),
    ));

    let mut written = 0;
    loop {
        let (len, status) = reader.read_str(&mut buf)?;
        writer.write_str(&buf[..len])?;
        writer.flush_with_status(status)?;
        written += len as u64;
        if status.is_end() {
            return Ok(written);
        }
    }
}
