#![no_main]

#[macro_use]
extern crate libfuzzer_sys;

use io_ext::ReadExt;
use io_ext_adapters::ExtReader;
use std::{io::Read, str};
use textual::Utf8Reader;

fuzz_target!(|bytes: &[u8]| {
    // Reading from a `Utf8Reader` should produce the same output as `String::from_utf8_lossy`.
    let lossy = String::from_utf8_lossy(bytes).to_string();
    let input = bytes;
    let mut reader = Utf8Reader::new(ExtReader::new(input));
    let mut buf = [0; 4];
    let mut b4 = Vec::new();
    let r4 = loop {
        match reader.read_with_status(&mut buf) {
            Ok((size, status)) => {
                b4.extend_from_slice(&buf[..size]);
                if status.is_end() {
                    break Ok(());
                }
            }
            Err(e) => break Err(e),
        }
    };
    let b4s = str::from_utf8(&b4).unwrap().to_string();
    match &r4 {
        Ok(()) => assert_eq!(lossy, b4s),
        Err(_) => assert_eq!(lossy, b4s + "\u{fffd}"),
    }

    // Reading with 8-byte buffers should produce the same results as reading with 4-byte buffers.
    let input = bytes;
    let mut reader = Utf8Reader::new(ExtReader::new(input));
    let mut buf = [0; 8];
    let mut b8 = Vec::new();
    let r8 = loop {
        match reader.read_with_status(&mut buf) {
            Ok((size, status)) => {
                b8.extend_from_slice(&buf[..size]);
                if status.is_end() {
                    break Ok(());
                }
            }
            Err(e) => break Err(e),
        }
    };
    assert_eq!(r4.is_ok(), r8.is_ok());
    if r8.is_ok() {
        assert_eq!(b4, b8);
    }
});
