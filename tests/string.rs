use basic_text::{text, TextString};

#[test]
fn text_string_ends_with() {
    let a = text!("abcdef").to_owned();
    assert!(a.ends_with("def"));
}

#[test]
fn text_string_extend() {
    let mut a = text!("abcdef").to_owned();
    a.extend(vec![text!("ghi"), text!("jkl"), text!("mno")].into_iter());
    assert_eq!(a, "abcdefghijklmno");
}

#[test]
fn text_string_escape_default() {
    assert_eq!(
        text!("abc\tdef\n").escape_default().collect::<String>(),
        "abc\\tdef\\n".to_owned()
    );
}

#[test]
fn text_string_concat() {
    let hello = TextString::from_text_lossy("hello").into_owned();
    let world = TextString::from_text_lossy(" world");
    let hello_world = hello + &world;
    assert_eq!(&hello_world, text!("hello world"));
}

#[test]
fn text_string_concat_no_compose() {
    let hello = TextString::from_text_lossy("hello").into_owned();
    let world = TextString::from_text_lossy("\u{308}world");
    let hello_world = hello + &world;
    assert_eq!(&hello_world, text!("hello\u{34f}\u{308}world"));
}

#[test]
fn text_string_concat_lossy() {
    let hello = TextString::from_text_lossy("hello\u{110bd}").into_owned();
    let world = TextString::from_text_lossy("\u{308}world");
    let hello_world = hello + &world;
    assert_eq!(&hello_world, text!("hello\u{110bd}\u{34f}\u{308}world"));
}
