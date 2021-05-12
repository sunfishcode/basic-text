mod disallowed_scalar_values;

use basic_text::TextReader;
use disallowed_scalar_values::DISALLOWED_SCALAR_VALUES;
use std::io::Read;

fn to_text(input: &str) -> String {
    let mut reader = TextReader::new(input.as_bytes());
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn test_text_input_start() {
    // If the stream starts with U+FEFF (BOM), it is removed.
    assert_eq!(to_text("\u{feff}"), "");
    assert_eq!(to_text("\u{feff}\n"), "\n");
    assert_eq!(to_text("\u{feff}hello"), "hello\n");
}

#[test]
fn test_text_input_crlf() {
    // Replace U+000D U+000A with U+000A (newline).
    assert_eq!(to_text("\r\n"), "\n");
    assert_eq!(to_text("hello\r\n"), "hello\n");
    assert_eq!(to_text("hello\r\nworld"), "hello\nworld\n");
}

#[test]
fn test_text_input_cr() {
    // Replace U+000D (CR) not followed by U+000A with U+000A (newline).
    assert_eq!(to_text("\r"), "\n");
    assert_eq!(to_text("\rhello\n"), "\nhello\n");
    assert_eq!(to_text("\rhello\r"), "\nhello\n");
    assert_eq!(to_text("hello\rworld"), "hello\nworld\n");
    assert_eq!(to_text("\n\r"), "\n\n");
}

#[test]
fn test_text_input_disallowed_scalars() {
    // *Disallowed scalar values* with U+FFFD (REPLACEMENT CHARACTER)
    for c in &DISALLOWED_SCALAR_VALUES {
        assert_eq!(
            to_text(c.encode_utf8(&mut [0_u8; 4])),
            "\u{fffd}\n",
            "disallowed scalar value {:?} was not replaced",
            c
        );
    }
}

#[test]
fn test_text_input_pre_nfc() {
    // Replace U+2329 with U+FFFD (before NFC).
    assert_eq!(to_text("\u{2329}"), "\u{fffd}\n");

    // Replace U+232a with U+FFFD (before NFC).
    assert_eq!(to_text("\u{232a}"), "\u{fffd}\n");
}

#[test]
fn test_text_input_nfc() {
    // These happen as part of NFC.
    assert_eq!(to_text("\u{2126}"), "\u{3a9}\n");
    assert_eq!(to_text("\u{212a}"), "\u{4b}\n");
    assert_eq!(to_text("\u{212b}"), "\u{c5}\n");

    // TODO: Test that all the following are done:
    // - Convert all CJK Compatibility Ideograph scalar values that have
    //   corresponding [Standardized Variations] into their corresponding
    //   standardized variation sequences.
    // - Apply the [Stream-Safe Text Process (UAX15-D4)].
    // - Apply `toNFC` according to the [Normalization Process for Stabilized Strings].
    //
    // TODO: Test that cjk_compat_variants is done before NFC
    // TODO: Test that stream-safe is done before NFC
}

#[test]
fn test_text_input_replacements() {
    // Replace U+FEFF (BOM) with U+2060 (WJ).
    assert_eq!(to_text("hello\u{feff}world"), "hello\u{2060}world\n");
    assert_eq!(to_text("hello\u{feff}"), "hello\u{2060}\n");
    assert_eq!(to_text("hello\u{feff}\n"), "hello\u{2060}\n");

    // Replace U+0007 (BEL) with U+FFFD (REPLACEMENT CHARACTER).
    assert_eq!(to_text("\u{7}"), "\u{fffd}\n");
    assert_eq!(to_text("\u{7}\n"), "\u{fffd}\n");
    assert_eq!(to_text("hello\u{7}world"), "hello\u{fffd}world\n");

    // Replace U+000C (FF) with U+0020 (SP).
    assert_eq!(to_text("\u{c}"), " \n");
    assert_eq!(to_text("\u{c}\n"), " \n");
    assert_eq!(to_text("\n\u{c}\n"), "\n \n");
    assert_eq!(to_text("hello\u{c}world"), "hello world\n");

    // Replace U+0085 (NEL) with U+0020 (SP).
    assert_eq!(to_text("\u{85}"), " \n");
    assert_eq!(to_text("\u{85}\n"), " \n");
    assert_eq!(to_text("\n\u{85}\n"), "\n \n");
    assert_eq!(to_text("hello\u{85}world"), "hello world\n");

    // Replace U+0149 with U+02BC U+006E.
    assert_eq!(to_text("\u{149}"), "\u{2bc}\u{6e}\n");

    // Replace U+0673 with U+0627 U+065F.
    assert_eq!(to_text("\u{673}"), "\u{627}\u{65f}\n");

    // Replace U+0F77 with U+0FB2 U+0F81. Prefix with "A" since U+F77 is Extend
    // and would otherwise be replaced by U+FFFD.
    assert_eq!(to_text("A\u{f77}"), "A\u{fb2}\u{f71}\u{f80}\n");

    // Replace U+0F79 with U+0FB3 U+0F81. Prefix with "A" since U+F79 is Extend
    // and would otherwise be replaced by U+FFFD.
    assert_eq!(to_text("A\u{f79}"), "A\u{fb3}\u{f71}\u{f80}\n");

    // Replace U+17A3 with U+17A2.
    assert_eq!(to_text("\u{17a3}"), "\u{17a2}\n");

    // Replace U+17A4 with U+17A2 U+17B6.
    assert_eq!(to_text("\u{17a4}"), "\u{17a2}\u{17b6}\n");

    // Replace U+2028 and U+2029 with U+20.
    assert_eq!(to_text("\u{2028}"), " \n");
    assert_eq!(to_text("\u{2029}"), " \n");
}

#[test]
fn test_text_input_escapes() {
    // Replace U+001B (ESC) as part of an *escape sequence* with nothing.
    assert_eq!(to_text("\u{1b}["), "\n");
    assert_eq!(to_text("\u{1b}[A"), "\n");
    assert_eq!(to_text("\u{1b}[AB"), "B\n");
    assert_eq!(to_text("\u{1b}[+"), "\n");
    assert_eq!(to_text("\u{1b}[+A"), "\n");
    assert_eq!(to_text("\u{1b}[+AB"), "B\n");
    assert_eq!(to_text("\u{1b}[++"), "\n");
    assert_eq!(to_text("\u{1b}[++A"), "\n");
    assert_eq!(to_text("\u{1b}[++AB"), "B\n");
    assert_eq!(to_text("\u{1b}[\u{18}A"), "A\n");
    assert_eq!(to_text("\u{1b}[\u{1b}AB"), "B\n");
    assert_eq!(to_text("\u{1b}]\u{7}"), "\n");
    assert_eq!(to_text("\u{1b}]\u{18}"), "\n");
    assert_eq!(to_text("\u{1b}]A\u{7}"), "\n");
    assert_eq!(to_text("\u{1b}]A\u{18}"), "\n");
    assert_eq!(to_text("\u{1b}]A\u{1b}[BC"), "C\n");
    assert_eq!(to_text("\u{1b}]A\u{1b}]CD\u{7}E"), "E\n");
    assert_eq!(to_text("\u{1b}]A\n\tB၌\u{7}"), "\n");
    assert_eq!(to_text("\u{1b}]A\n\tB၌\u{18}"), "\n");
    assert_eq!(to_text("\u{1b}]\u{18}"), "\n");
    assert_eq!(to_text("\u{1b}]A\u{18}"), "\n");
    assert_eq!(to_text("\u{1b}]A\n\tB၌\u{18}"), "\n");
    assert_eq!(to_text("\u{1b}A"), "\n");
    assert_eq!(to_text("\u{1b}\u{18}"), "\n");
    assert_eq!(to_text("\u{1b}\u{1b}A"), "\n");
    assert_eq!(to_text("\u{1b}A\n"), "\n");
    assert_eq!(to_text("\u{1b}\t"), "\u{fffd}\t\n");
    assert_eq!(to_text("\u{1b}\n"), "\u{fffd}\n");
    assert_eq!(to_text("\u{1b}[["), "\n");
    assert_eq!(to_text("\u{1b}[[A"), "\n");
    assert_eq!(to_text("\u{1b}[[\0"), "\n");
    assert_eq!(to_text("\u{1b}[[\u{7f}"), "\n");
    assert_eq!(to_text("\u{1b}[[\n"), "\n");
    assert_eq!(to_text("\u{1b}[[A\n"), "\n");
    assert_eq!(to_text("\u{1b}[[\u{18}"), "\n");
    assert_eq!(to_text("\u{1b}[[\u{7}"), "\n");
    assert_eq!(to_text("\u{1b}[[\u{1b}A"), "A\n");

    // Replace U+001B (ESC) otherwise with U+FFFD (REPLACEMENT CHARACTER).
    assert_eq!(to_text("\u{1b}"), "\u{fffd}\n");
    assert_eq!(to_text("\u{1b}\n"), "\u{fffd}\n");
}

#[test]
fn test_text_input_end() {
    // At the end of the stream, if any scalar values were transmitted and the
    // last scalar value is not U+000A, after replacements, a U+000A is
    // appended.
    assert_eq!(to_text(""), "");
    assert_eq!(to_text("\n"), "\n");
    assert_eq!(to_text("hello"), "hello\n");
    assert_eq!(to_text("hello\nworld"), "hello\nworld\n");
    assert_eq!(to_text("hello\nworld\n"), "hello\nworld\n");
    assert_eq!(to_text("hello\r\nworld"), "hello\nworld\n");
    assert_eq!(to_text("hello\r\nworld\r\n"), "hello\nworld\n");
}
