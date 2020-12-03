#![no_main]

#[macro_use]
extern crate libfuzzer_sys;

use plain_text::{Read, StdReader, TextReader};

fuzz_target!(|bytes: &[u8]| {
    let input = bytes;
    let mut reader = TextReader::new(StdReader::new(input));

    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();

    // No BOMs.
    assert!(!s.chars().any(|c| c == '\u{feff}'));

    // Trailing newline.
    assert!(s.is_empty() || s.ends_with('\n'));

    // No control codes other than '\n' and '\t'.
    assert!(!s.chars().any(|c| c.is_control() && c != '\n' && c != '\t'));

    // Stream-Safe NFC.
    assert!(unicode_normalization::is_nfc_stream_safe(&s));

    // Don't start with non-starter.
    if let Some(first) = s.chars().next() {
        assert!(unicode_normalization::char::canonical_combining_class(first) == 0);
    }
});
