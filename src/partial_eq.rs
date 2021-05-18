//! `PartialEq` implementations for `TextString`, `TextStr`, `TextSubstring`,
//! and `TextSubstr`.

use crate::{TextStr, TextString, TextSubstr, TextSubstring};
use std::{borrow::Cow, str};

impl PartialEq<TextStr> for str {
    #[inline]
    fn eq(&self, other: &TextStr) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<&'a TextStr> for String {
    #[inline]
    fn eq(&self, other: &&'a TextStr) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<&'a str> for TextString {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<&'a TextStr> for TextString {
    #[inline]
    fn eq(&self, other: &&'a TextStr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, TextStr>> for TextString {
    #[inline]
    fn eq(&self, other: &Cow<'a, TextStr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, str>> for TextString {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl<'a, 'b> PartialEq<Cow<'a, str>> for &'b TextStr {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<TextString> for Cow<'a, TextStr> {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<TextString> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<TextStr> for Cow<'a, TextStr> {
    #[inline]
    fn eq(&self, other: &TextStr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<&'b TextStr> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &&TextStr) -> bool {
        self.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<&'b TextStr> for Cow<'a, TextStr> {
    #[inline]
    fn eq(&self, other: &&TextStr) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<str> for TextString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<String> for TextString {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<TextStr> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &TextStr) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<TextString> for String {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<TextString> for &'a str {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<TextString> for &'a TextStr {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<String> for &'a TextStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.eq(&other)
    }
}

impl PartialEq<TextString> for str {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<TextStr> for TextString {
    #[inline]
    fn eq(&self, other: &TextStr) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<TextString> for TextStr {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, Self>> for TextStr {
    #[inline]
    fn eq(&self, other: &Cow<'a, Self>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<Cow<'a, TextStr>> for &'b TextStr {
    #[inline]
    fn eq(&self, other: &Cow<'a, TextStr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, str>> for TextStr {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<str> for TextStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<String> for TextStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<TextStr> for String {
    #[inline]
    fn eq(&self, other: &TextStr) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<TextSubstr> for str {
    #[inline]
    fn eq(&self, other: &TextSubstr) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<&'a TextSubstr> for String {
    #[inline]
    fn eq(&self, other: &&'a TextSubstr) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<&'a str> for TextSubstring {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<&'a TextSubstr> for TextSubstring {
    #[inline]
    fn eq(&self, other: &&'a TextSubstr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, TextSubstr>> for TextSubstring {
    #[inline]
    fn eq(&self, other: &Cow<'a, TextSubstr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, str>> for TextSubstring {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl<'a, 'b> PartialEq<Cow<'a, str>> for &'b TextSubstr {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<TextSubstring> for Cow<'a, TextSubstr> {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<TextSubstring> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<TextSubstr> for Cow<'a, TextSubstr> {
    #[inline]
    fn eq(&self, other: &TextSubstr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<&'b TextSubstr> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &&TextSubstr) -> bool {
        self.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<&'b TextSubstr> for Cow<'a, TextSubstr> {
    #[inline]
    fn eq(&self, other: &&TextSubstr) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<str> for TextSubstring {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<String> for TextSubstring {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<TextSubstr> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &TextSubstr) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<TextSubstring> for String {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<TextSubstring> for &'a str {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<TextSubstring> for &'a TextSubstr {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<String> for &'a TextSubstr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.eq(&other)
    }
}

impl PartialEq<TextSubstring> for str {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<TextSubstr> for TextSubstring {
    #[inline]
    fn eq(&self, other: &TextSubstr) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<TextSubstring> for TextSubstr {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, Self>> for TextSubstr {
    #[inline]
    fn eq(&self, other: &Cow<'a, Self>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<Cow<'a, TextSubstr>> for &'b TextSubstr {
    #[inline]
    fn eq(&self, other: &Cow<'a, TextSubstr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, str>> for TextSubstr {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<str> for TextSubstr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<String> for TextSubstr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<TextSubstr> for String {
    #[inline]
    fn eq(&self, other: &TextSubstr) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<&'a TextStr> for TextSubstring {
    #[inline]
    fn eq(&self, other: &&'a TextStr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<&'a TextSubstr> for TextString {
    #[inline]
    fn eq(&self, other: &&'a TextSubstr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, TextStr>> for TextSubstring {
    #[inline]
    fn eq(&self, other: &Cow<'a, TextStr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, TextSubstr>> for TextString {
    #[inline]
    fn eq(&self, other: &Cow<'a, TextSubstr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<TextString> for Cow<'a, TextSubstr> {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<TextSubstring> for Cow<'a, TextStr> {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<TextStr> for Cow<'a, TextSubstr> {
    #[inline]
    fn eq(&self, other: &TextStr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<TextSubstr> for Cow<'a, TextStr> {
    #[inline]
    fn eq(&self, other: &TextSubstr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<&'b TextStr> for Cow<'a, TextSubstr> {
    #[inline]
    fn eq(&self, other: &&TextStr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<&'b TextSubstr> for Cow<'a, TextStr> {
    #[inline]
    fn eq(&self, other: &&TextSubstr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<TextString> for &'a TextSubstr {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<TextSubstring> for &'a TextStr {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<TextStr> for TextSubstring {
    #[inline]
    fn eq(&self, other: &TextStr) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<TextSubstr> for TextString {
    #[inline]
    fn eq(&self, other: &TextSubstr) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<TextString> for TextSubstr {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<TextSubstring> for TextStr {
    #[inline]
    fn eq(&self, other: &TextSubstring) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<Cow<'a, TextStr>> for &'b TextSubstr {
    #[inline]
    fn eq(&self, other: &Cow<'a, TextStr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<Cow<'a, TextSubstr>> for &'b TextStr {
    #[inline]
    fn eq(&self, other: &Cow<'a, TextSubstr>) -> bool {
        self.0.eq(&other.0)
    }
}
