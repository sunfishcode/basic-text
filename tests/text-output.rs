mod disallowed_scalar_values;

use disallowed_scalar_values::DISALLOWED_SCALAR_VALUES;
use io_ext_adapters::ExtWriter;
use std::io::{self, Write};
use text_streams::TextWriter;

fn to_text(input: &str) -> io::Result<String> {
    let mut writer = TextWriter::new(ExtWriter::new(Vec::<u8>::new()));
    writer.write_all(input.as_bytes())?;
    let inner = writer.close_into_inner()?;
    Ok(String::from_utf8(inner.get_ref().to_vec()).unwrap())
}

fn to_text_with_bom_compatibility(input: &str) -> io::Result<String> {
    let mut writer = TextWriter::with_bom_compatibility(ExtWriter::new(Vec::<u8>::new())).unwrap();
    writer.write_all(input.as_bytes())?;
    let inner = writer.close_into_inner()?;
    Ok(String::from_utf8(inner.get_ref().to_vec()).unwrap())
}

fn to_text_with_crlf_compatibility(input: &str) -> io::Result<String> {
    let mut writer = TextWriter::with_crlf_compatibility(ExtWriter::new(Vec::<u8>::new()));
    writer.write_all(input.as_bytes())?;
    let inner = writer.close_into_inner()?;
    Ok(String::from_utf8(inner.get_ref().to_vec()).unwrap())
}

#[test]
fn test_text_output_nfc() {
    // TODO: Test that all the following are done:
    // - Convert all CJK Compatibility Ideograph scalar values that have corresponding
    //   [Standardized Variations] into their corresponding standardized variation
    //   sequences.
    // - Apply the [Stream-Safe Text Process (UAX15-D4)].
    // - Apply `toNFC` according to the [Normalization Process for Stabilized Strings].
    //
    // TODO: Test that svar is done before NFC
    // TODO: Test that stream-safe is done before NFC
}

#[test]
fn test_bom_compatibility() {
    // As an option (BOM compatibility), off by default, prepend U+FEFF to the stream.
    assert_eq!(to_text_with_bom_compatibility("").unwrap(), "\u{feff}");
    assert_eq!(to_text_with_bom_compatibility("\n").unwrap(), "\u{feff}\n");
    assert_eq!(
        to_text_with_bom_compatibility("hello\n").unwrap(),
        "\u{feff}hello\n"
    );
}

#[test]
fn test_crlf_compatibility() {
    // As an option (CRLF compatibility), off by default, replace "\n" with "\r\n".
    assert_eq!(to_text_with_crlf_compatibility("\n").unwrap(), "\r\n");
    assert_eq!(to_text_with_crlf_compatibility("\n\n").unwrap(), "\r\n\r\n");
    assert_eq!(
        to_text_with_crlf_compatibility("\r\n").unwrap_err().kind(),
        io::ErrorKind::Other
    );
    assert_eq!(
        to_text_with_crlf_compatibility("hello\n").unwrap(),
        "hello\r\n"
    );
    assert_eq!(
        to_text_with_crlf_compatibility("hello\nworld\n").unwrap(),
        "hello\r\nworld\r\n"
    );
}

#[test]
fn test_text_output_rules() {
    // Fail at *disallowed scalar values*.
    for c in &DISALLOWED_SCALAR_VALUES {
        assert_eq!(
            to_text(&format!("{}\n", c)).unwrap_err().kind(),
            io::ErrorKind::Other,
            "disallowed scalar value {:?} was not rejected",
            c,
        );
    }

    // Fail at U+FEFF (not at the beginning of the stream).
    assert_eq!(
        to_text("hello\u{feff}\n").unwrap_err().kind(),
        io::ErrorKind::Other
    );

    // Fail at specific scalar values.
    for c in &[
        '\u{7}', '\u{c}', '\u{1b}', '\u{feff}', '\u{149}', '\u{0673}', '\u{0F77}', '\u{0F79}',
        '\u{17a3}', '\u{17a4}', '\u{2329}', '\u{232a}', '\u{2126}', '\u{212a}', '\u{212b}',
    ] {
        assert_eq!(
            to_text(&format!("{}\n", c)).unwrap_err().kind(),
            io::ErrorKind::Other,
            "specific scalar value {:?} was not rejected",
            c,
        );
    }

    // At the end of the stream, if any scalar values were transmitted and the
    // last scalar value is not U+000A, fail.
    assert_eq!(to_text("").unwrap(), "");
    assert_eq!(to_text("\n").unwrap(), "\n");
    assert_eq!(to_text("hello\n").unwrap(), "hello\n");
    assert_eq!(to_text("hello\nworld\n").unwrap(), "hello\nworld\n");

    assert_eq!(to_text("hello").unwrap_err().kind(), io::ErrorKind::Other);
    assert_eq!(
        to_text("hello\nworld").unwrap_err().kind(),
        io::ErrorKind::Other
    );

    assert_eq!(to_text("\r\n").unwrap_err().kind(), io::ErrorKind::Other);
    assert_eq!(
        to_text("hello\r\n").unwrap_err().kind(),
        io::ErrorKind::Other
    );
    assert_eq!(
        to_text("hello\r\nworld").unwrap_err().kind(),
        io::ErrorKind::Other
    );
    assert_eq!(
        to_text("hello\r\nworld\r\n").unwrap_err().kind(),
        io::ErrorKind::Other
    );
}
