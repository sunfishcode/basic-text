use basic_text::{text, BufReadText};
use std::io::BufReader;

#[test]
fn buf_read_test_basics() {
    let input = "red\norange\nyellow\ngreen\nblue\npurple\n";
    let reader = BufReader::new(input.as_bytes());
    let mut lines = reader.text_lines();
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("red").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("orange").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("yellow").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("green").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("blue").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("purple").to_owned())
    );
    assert_eq!(lines.next().map(Result::unwrap), None);
}

#[test]
fn buf_read_test_lossy() {
    let input = "\u{1d515}ed\u{200d}\n\u{200d}oran\u{1d4bc}e\nye\u{1d4a7}\u{1d4a7}ow\ngree\u{1d545}\n\u{2329}blue\u{232a}\npurple\u{2072}\n";
    let reader = BufReader::new(input.as_bytes());
    let mut lines = reader.text_lines_lossy();
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("�ed\u{200d}\u{34f}").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("\u{34f}\u{200d}oran�e").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("ye��ow").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("gree�").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("�blue�").to_owned())
    );
    assert_eq!(
        lines.next().map(Result::unwrap),
        Some(text!("purple�").to_owned())
    );
    assert_eq!(lines.next().map(Result::unwrap), None);
}
