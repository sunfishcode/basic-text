#[test]
fn text_string_ends_with() {
    use basic_text::text;
    let a = text!("abcdef").to_owned();
    assert!(a.ends_with("def"));
}

#[test]
fn text_string_extend() {
    use basic_text::text;
    let mut a = text!("abcdef").to_owned();
    a.extend(vec![text!("ghi"), text!("jkl"), text!("mno")].into_iter());
    assert_eq!(a, "abcdefghijklmno");
}
