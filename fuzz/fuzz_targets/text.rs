#![no_main]

#[macro_use]
extern crate libfuzzer_sys;

use basic_text::{TextReader, TextWriter};
use layered_io::{LayeredWriter, SliceReader};
use std::{
    io::{Read, Write},
    str,
};
use unicode_normalization::UnicodeNormalization;
use utf8_io::{Utf8Reader, Utf8Writer, WriteStr};

fuzz_target!(|bytes: &[u8]| {
    let mut reader = TextReader::new(Utf8Reader::new(SliceReader::new(bytes)));
    let mut writer = TextWriter::new(Utf8Writer::new(LayeredWriter::new(Vec::<u8>::new())));

    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();

    // No BOMs.
    assert!(!s.chars().any(|c| c == '\u{feff}'));

    // No ORCs.
    assert!(!s.chars().any(|c| c == '\u{fffc}'));

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

    // Writing it back out to a text writer should preserve the bytes.
    writer.write_str(&s).unwrap();
    let inner = writer
        .close_into_inner()
        .unwrap()
        .close_into_inner()
        .unwrap();
    assert_eq!(inner.get_ref(), s.as_bytes());

    // Iff a text reader normalized something, that same thing should fail
    // when written as output.
    let mut writer = TextWriter::new(Utf8Writer::new(LayeredWriter::new(Vec::<u8>::new())));
    match str::from_utf8(bytes) {
        Ok(utf8) => {
            let result = writer.write_all(bytes).and_then(|()| {
                writer
                    .close_into_inner()?
                    .close_into_inner()?
                    .close_into_inner()
            });
            if !s.contains('\x1b')
                && utf8
                    .chars()
                    .cjk_compat_variants()
                    .stream_safe()
                    .nfc()
                    .collect::<String>()
                    == s
            {
                assert_eq!(String::from_utf8(result.unwrap()).unwrap(), s);
            } else {
                result.unwrap_err();
            }
        }
        Err(_) => {
            writer
                .write_all(bytes)
                .and_then(|()| writer.close_into_inner().map(|_| ()))
                .unwrap_err();
        }
    }
});
