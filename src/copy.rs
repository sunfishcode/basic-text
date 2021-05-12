use crate::{ReadText, ReadTextLayered, WriteText};
use layered_io::Bufferable;
use std::{cmp::max, io};

/// Like `std::io::copy`, but for streams that can operate directly on text
/// strings, so we can avoid re-validating them as text.
pub fn copy_text<R: ReadText + Bufferable + ?Sized, W: WriteText + Bufferable + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> io::Result<u64> {
    // TODO: Avoid unnecessary zero-initialization.
    let mut buf = " ".repeat(max(
        reader.suggested_buffer_size(),
        writer.suggested_buffer_size(),
    ));

    let mut written = 0;
    loop {
        let len = match reader.read_text(&mut buf) {
            Ok(0) => break,
            Ok(nread) => nread,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        };
        // Use `write_str` here instead of `write_text` because we may split
        // strings at non-starters.
        writer.write_str(&buf[..len])?;
        written += len as u64;
    }
    Ok(written)
}

/// Like `std::io::copy`, but for streams that can operate directly on text
/// strings, so we can avoid re-validating them as text.
///
/// Also, like `copy_text`, but uses `read_text_with_status` to avoid performing
/// an extra `read` at the end.
pub fn copy_text_using_status<R: ReadTextLayered + ?Sized, W: WriteText + Bufferable + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> io::Result<u64> {
    // TODO: Avoid unnecessary zero-initialization.
    let mut buf = " ".repeat(max(
        reader.suggested_buffer_size(),
        writer.suggested_buffer_size(),
    ));

    let mut written = 0;
    loop {
        let (len, status) = reader.read_text_with_status(&mut buf)?;
        // Use `write_str` here instead of `write_text` because we may split
        // strings at non-starters.
        writer.write_str(&buf[..len])?;
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
fn test_copy_text() {
    use crate::{TextReader, TextStr, TextWriter};
    use std::{io::Cursor, str};

    let text = "hello world ☃\n";
    let mut input = TextReader::new(Cursor::new(text.to_owned()));
    let mut output = TextWriter::new(Vec::new());

    copy_text(&mut input, &mut output).unwrap();

    let ext = output.close_into_inner().unwrap();
    let vec = ext.abandon_into_inner().abandon_into_inner().unwrap();
    let s = str::from_utf8(&vec).unwrap();
    assert_eq!(s, text);
    let t = TextStr::from_text(s).unwrap();
    assert_eq!(t, text);
}

#[test]
fn test_copy_text_using_status() {
    use crate::{TextReader, TextStr, TextWriter};
    use std::{io::Cursor, str};

    let text = "hello world ☃";
    let mut input = TextReader::new(Cursor::new(text.to_owned()));
    let mut output = TextWriter::new(Vec::new());

    copy_text_using_status(&mut input, &mut output).unwrap();

    let ext = output.close_into_inner().unwrap();
    let vec = ext.abandon_into_inner().abandon_into_inner().unwrap();
    let s = str::from_utf8(&vec).unwrap();
    assert_eq!(s, &format!("{}\n", text));
    let t = TextStr::from_text(s).unwrap();
    assert_eq!(t, &format!("{}\n", text));
}
