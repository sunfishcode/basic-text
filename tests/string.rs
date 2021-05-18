#[test]
fn text_string_ends_with() {
    use basic_text::text;
    let a = text!("abcdef").to_owned();
    assert!(a.ends_with("def"));
}
