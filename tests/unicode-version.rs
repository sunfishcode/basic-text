#[test]
fn unicode_version() {
    // Check that our version of Unicode matches `unicode_normalization`'s.
    assert_eq!(
        basic_text::UNICODE_VERSION,
        (15, 1, 0),
        "Code referencing Unicode 15.1.0 needs to be updated."
    );
}
