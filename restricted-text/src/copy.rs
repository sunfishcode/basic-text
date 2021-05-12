use crate::{ReadRestricted, ReadRestrictedLayered, RestrictedStr, RestrictedString, WriteRestricted};
use layered_io::Bufferable;
use std::{cmp::max, io};
use basic_text::{TextStr, TextString};

/// Like `std::io::copy`, but for streams that can operate directly on
/// restricted strings, so we can avoid re-validating them as restricted.
pub fn copy_restricted<R: ReadRestricted + Bufferable + ?Sized, W: WriteRestricted + Bufferable + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> io::Result<u64> {
    // TODO: Avoid unnecessary zero-initialization.
    let mut buf = unsafe {
        RestrictedString::from_restricted_unchecked(" ".repeat(max(
            reader.suggested_buffer_size(),
            writer.suggested_buffer_size(),
        )))
    };

    let mut written = 0;
    loop {
        let len = match reader.read_restricted(&mut buf) {
            Ok(0) => break,
            Ok(nread) => nread,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        };
        // TODO: Implement `Index` for `RestrictedStr`?
        writer.write_restricted(RestrictedStr::from_restricted(&buf.as_utf8()[..len]).unwrap())?;
        written += len as u64;
    }
}

/// Like `std::io::copy`, but for streams that can operate directly on restricted
/// strings, so we can avoid re-validating them as restricted.
///
/// Also, like `copy_restricted`, but uses `read_restricted_with_status` to avoid performing
/// an extra `read` at the end.
pub fn copy_restricted_using_status<R: ReadRestrictedLayered + ?Sized, W: WriteRestricted + Bufferable + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> io::Result<u64> {
    // TODO: Avoid unnecessary zero-initialization.
    let mut buf = RestrictedString::from_restricted(" ".repeat(max(
        reader.suggested_buffer_size(),
        writer.suggested_buffer_size(),
    )))
    .unwrap();

    let mut written = 0;
    loop {
        let (len, status) = reader.read_restricted_with_status(&mut buf)?;
        // TODO: Implement `Index` for `RestrictedStr`?
        writer.write_restricted(RestrictedStr::from_restricted(&buf.as_utf8()[..len]).unwrap())?;
        written += len as u64;
        if status.is_end() {
            return Ok(written);
        }
        if status.is_push() {
            writer.flush()?;
        }
    }
}

#[test]
fn test_copy_restricted() {
    use crate::{RestrictedReader, RestrictedStr, RestrictedWriter};
    use layered_io::{LayeredReader, LayeredWriter};
    use std::{io::Cursor, str};
    use utf8_io::{Utf8Reader, Utf8Writer};
    use basic_text::{TextReader, TextWriter};

    let restricted = "hello world ☃\n";
    let mut input = RestrictedReader::new(TextReader::new(Cursor::new(
        restricted.to_string(),
    )));
    let mut output = RestrictedWriter::new(TextWriter::new(Vec::new()));

    copy_restricted(&mut input, &mut output).unwrap();

    let ext = output.close_into_inner().unwrap();
    let vec = ext.abandon_into_inner().abandon_into_inner().unwrap();
    let s = str::from_utf8(&vec).unwrap();
    assert_eq!(s, restricted);
    let t = TextStr::from_restricted(s).unwrap();
    assert_eq!(t, restricted);
}

#[test]
fn test_copy_restricted_using_status() {
    use crate::{TextReader, TextStr, TextWriter};
    use layered_io::{LayeredReader, LayeredWriter};
    use std::{io::Cursor, str};
    use utf8_io::{Utf8Reader, Utf8Writer};

    let restricted = "hello world ☃";
    let mut input = TextReader::new(Cursor::new(
        restricted.to_string(),
    ));
    let mut output = TextWriter::new(Vec::new());

    copy_restricted_using_status(&mut input, &mut output).unwrap();

    let ext = output.close_into_inner().unwrap();
    let vec = ext.abandon_into_inner().abandon_into_inner().unwrap();
    let s = str::from_utf8(&vec).unwrap();
    assert_eq!(s, &format!("{}\n", restricted));
    let t = TextStr::from_restricted(s).unwrap();
    assert_eq!(t, &format!("{}\n", restricted));
}
