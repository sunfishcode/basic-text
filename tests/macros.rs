use basic_text::{text, text_substr};
use std::io::{sink, Write};

#[test]
fn text_macro() {
    let mut s = sink();
    writeln!(s, "{}", text!("hello world")).unwrap();
}

#[test]
fn text_substr_macro() {
    let mut s = sink();
    writeln!(s, "{}", text_substr!("hello world")).unwrap();
    writeln!(s, "{}", text_substr!("\u{200d}hello world\u{200d}")).unwrap();
}
