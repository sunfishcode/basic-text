use basic_text::{text, TextStr, TextString};
use std::{fs::File, io::Read};

#[test]
fn basic_text_example() {
    let mut s = String::new();
    File::open("samples/basic-text-example.txt")
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    assert_eq!(TextStr::from_text(&s).unwrap(), s);
}

#[test]
fn basic_text_invalid() {
    let mut invalid = Vec::new();
    File::open("samples/basic-text-invalid.txt")
        .unwrap()
        .read_to_end(&mut invalid)
        .unwrap();
    let mut converted = String::new();
    File::open("samples/basic-text-invalid-converted.txt")
        .unwrap()
        .read_to_string(&mut converted)
        .unwrap();

    if *TextString::from_text_bytes_lossy(&invalid) != converted {
        for (a, b) in (*TextString::from_text_bytes_lossy(&invalid))
            .chars()
            .zip(converted.chars())
        {
            if a == b {
                dbg!(a);
            } else {
                eprintln!("DIFFER: {:?} vs {:?}", a, b);
            }
        }
    }

    // Manually append a U+A, since string conversion doesn't do stream conversion.
    let mut check = (*TextString::from_text_bytes_lossy(&invalid)).to_owned();
    check.push_text(text!("\n"));

    assert_eq!(check, converted);
}
