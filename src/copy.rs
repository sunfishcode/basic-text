use crate::{ReadStr, ReadText, TextStr, TextString, WriteStr, WriteText};
use io_ext::Bufferable;
use std::{cmp::max, io};

/// Like `std::io::copy`, but for streams that can operate directly on strings,
/// so we can avoid re-validating them as UTF-8.
pub fn copy_str<R: ReadStr + Bufferable + ?Sized, W: WriteStr + Bufferable + ?Sized>(
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
        written += len as u64;
        if status.is_end() {
            return Ok(written);
        }
        if status.is_push() {
            writer.flush()?;
        }
    }
}

/// Like `std::io::copy`, but for streams that can operate directly on text
/// strings, so we can avoid re-validating them as text.
pub fn copy_text<R: ReadText + Bufferable + ?Sized, W: WriteText + Bufferable + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> io::Result<u64> {
    // TODO: Avoid unnecessary zero-initialization.
    let mut buf = unsafe {
        TextString::from_text_unchecked(" ".repeat(max(
            reader.suggested_buffer_size(),
            writer.suggested_buffer_size(),
        )))
    };

    let mut written = 0;
    loop {
        let (len, status) = reader.read_text(&mut buf)?;
        let s: &str = buf.as_ref(); // TODO: Avoid doing this.
        writer.write_text(unsafe { TextStr::from_text_unchecked(&s[..len]) })?; // TODO: and this
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
fn test_copy_str() {
    use crate::{write_wrapper::WriteWrapper, Utf8Reader, Utf8Writer};
    use io_ext_adapters::{ExtReader, ExtWriter};
    use std::{io::Cursor, str};

    let text = "hello world ☃";
    let mut input = Utf8Reader::new(ExtReader::new(Cursor::new(text.to_string())));
    let mut output = Utf8Writer::new(ExtWriter::new(Vec::new()));

    copy_str(&mut input, &mut output).unwrap();

    let ext = output.close_into_inner().unwrap();
    let vec = ext.abandon_into_inner().unwrap();
    assert_eq!(str::from_utf8(&vec).unwrap(), text);
}

#[test]
fn test_copy_text() {
    use crate::{TextReader, TextWriter};
    use io_ext_adapters::{ExtReader, ExtWriter};
    use std::{io::Cursor, str};

    let text = "hello world ☃\n";
    let mut input = TextReader::new(ExtReader::new(Cursor::new(text.to_string())));
    let mut output = TextWriter::new(ExtWriter::new(Vec::new()));

    copy_text(&mut input, &mut output).unwrap();

    let ext = output.close_into_inner().unwrap();
    let vec = ext.abandon_into_inner().unwrap();
    assert_eq!(str::from_utf8(&vec).unwrap(), text);
}
