use crate::{TextReader, TextWriter};
use basic_text_internals::{
    is_basic_text,
    unicode::{BOM, WJ},
};
use layered_io::{Bufferable, LayeredReader, LayeredWriter};
#[cfg(try_reserve)]
use std::collections::TryReserveError;
#[cfg(pattern)]
use std::str::{
    pattern::{Pattern, ReverseSearcher},
    MatchIndices, Matches, RMatchIndices, RMatches,
};
use std::{
    borrow::{Borrow, BorrowMut, Cow},
    cmp::Ordering,
    error::Error,
    ffi::OsStr,
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
    io::{self, Read, Write},
    net::{SocketAddr, ToSocketAddrs},
    ops::{Add, AddAssign, Deref, DerefMut},
    path::Path,
    str::{self, Bytes, CharIndices, Chars, EncodeUtf16, FromStr, Lines, Utf8Error},
    string::FromUtf8Error,
    vec,
};
use utf8_io::{Utf8Reader, Utf8Writer, WriteStr};

/// A Basic Text encoded, growable string.
///
/// This is an owning string similar to [`String`], but ensures the contents
/// are Basic Text rather than just UTF-8. It's accompanied by a borrowing
/// [`TextStr`], which plays an analogous role [`str`].
#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct TextString(String);

/// Text slices.
///
/// `TextStr` is to `TextString` as [`str`] is to `String`. It is usually used
/// for borrowing, in the form of `&TextStr`.
///
/// # Examples
///
/// Text literals are text slices:
///
/// ```rust
/// use basic_text::{text, TextStr};
///
/// let hello = text!("Hello, world!");
///
/// // with an explicit type annotation
/// let hello: &'static TextStr = text!("Hello, world!");
/// ```
///
/// They are 'static because they’re stored directly in the final binary, and so
/// will be valid for the 'static duration.
#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct TextStr(str);

/// `TextError` is to `TextString` as `Utf8Error` is to `String`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TextError {
    valid_up_to: usize,
}

/// `FromTextError` is to `TextString` as `FromUtf8Error` is to `String`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromTextError {
    bytes: Vec<u8>,
    error: TextError,
}

impl TextString {
    /// Creates a new empty `TextString`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Creates a new empty `TextString` with a particular capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    /// Converts a vector of bytes to a `TextString`.
    #[inline]
    pub fn from_text_vec(vec: Vec<u8>) -> Result<Self, FromTextError> {
        Self::from_text(String::from_utf8(vec)?)
    }

    /// Converts a `String` to a `TextString`.
    #[inline]
    pub fn from_text(s: String) -> Result<Self, FromTextError> {
        let bytes: Vec<u8> = Vec::new();
        let mut writer = TextWriter::new(Utf8Writer::new(LayeredWriter::new(bytes)));

        match writer.write_str(&s).and_then(|()| writer.flush()) {
            Ok(()) => (),
            Err(_err) => {
                writer.abandon();
                let valid_up_to = compute_valid_up_to(&s);
                return Err(FromTextError {
                    bytes: s.into_bytes(),
                    error: TextError { valid_up_to },
                });
            }
        }

        Ok(unsafe {
            Self::from_text_vec_unchecked(
                writer
                    .abandon_into_inner()
                    .abandon_into_inner()
                    .abandon_into_inner()
                    .unwrap(),
            )
        })
    }

    /// Converts a slice of bytes to Basic Text, including invalid characters.
    #[inline]
    #[must_use]
    pub fn from_text_bytes_lossy(v: &[u8]) -> Cow<TextStr> {
        // TODO: optimize away the temporary String here
        Cow::Owned(Self::from_text_lossy(&String::from_utf8_lossy(v)).into_owned())
    }

    /// Converts a string to Basic Text, including invalid characters.
    #[inline]
    #[must_use]
    pub fn from_text_lossy(mut v: &str) -> Cow<TextStr> {
        // TODO: If `v` is already valid, fast-path to `Cow::Borrowed(v)`.
        // TODO: Also, this currently redoes UTF-8 validation for `v`.
        let mut reader = TextReader::new(Utf8Reader::new(LayeredReader::new(v.as_bytes())));
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        // `TextReader` strips leading BOMs, but we don't want that behavior
        // here. Translate a BOM to a WJ here.
        if let Some(suffix) = v.strip_prefix(BOM) {
            text.insert(0, WJ);
            v = suffix;
        }

        // `TextReader` ensures the stream ends in a newline, but we don't
        // want that behavior here. Strip a trailing newline if needed.
        if !v.is_empty() && !v.ends_with(|c| matches!(c, '\n' | '\r')) {
            let c = text.pop();
            assert_eq!(c.unwrap(), '\n');
        }

        Cow::Owned(unsafe { Self::from_text_unchecked(text) })
    }

    // TODO: from_raw_parts, from_utf16*

    /// Converts a vector of bytes to a `TextString` without checking that the
    /// string contains valid Basic Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextStr`s
    /// are valid Basic Text.
    #[inline]
    #[must_use]
    pub unsafe fn from_text_vec_unchecked(vec: Vec<u8>) -> Self {
        Self::from_text_unchecked(String::from_utf8_unchecked(vec))
    }

    /// Converts a `String` to a `TextString` without checking that the string
    /// contains valid Basic Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextStr`s
    /// are valid Basic Text.
    #[inline]
    #[must_use]
    pub const unsafe fn from_text_unchecked(s: String) -> Self {
        Self(s)
    }

    /// Converts a `TextString` into a `String`.
    #[inline]
    #[must_use]
    pub fn into_utf8(self) -> String {
        self.0
    }

    /// Converts a String into a byte vector.
    #[inline]
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    /// Extracts a UTF-8 string slice containing the entire `TextString`.
    #[inline]
    #[must_use]
    pub fn as_utf8(&self) -> &str {
        &self.0
    }

    /// Extracts a Basic Text string slice containing the entire `TextString`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &TextStr {
        self
    }

    /// Converts a `TextString` into a mutable UTF-8 string slice.
    #[inline]
    #[must_use]
    pub fn as_mut_utf8(&mut self) -> &mut str {
        &mut self.0
    }

    /// Converts a `TextString` into a mutable Basic Text string slice.
    #[inline]
    #[must_use]
    pub fn as_mut_str(&mut self) -> &mut TextStr {
        self
    }

    /// Appends a given string slice onto the end of this `TextString`.
    ///
    /// But wait, NFC isn't closed under concatenation! This is true, but
    /// Basic Text has additional restrictions, including that strings start
    /// with non-combining codepoints, so it is closed under concatenation.
    #[inline]
    pub fn push_str(&mut self, s: &TextStr) {
        self.0.push_str(&s.0);
    }

    /// Returns this `TextString`'s capacity, in bytes.
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Ensures that this `TextString`'s capacity is at least `additional`
    /// bytes larger than its length.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Ensures that this `TextString`'s capacity is `additional` bytes larger
    /// than its length.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    /// Tries to reserve capacity for at least `additional` more elements to
    /// be inserted in the given `TextString`.
    #[cfg(try_reserve)]
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// Tries to reserves the minimum capacity for exactly `additional` more
    /// elements to be inserted in the given `TextString`.
    #[cfg(try_reserve)]
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    /// Shrinks the capacity of this `TextString` to match its length.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Shrinks the capacity of this `String` with a lower bound.
    #[cfg(shrink_to)]
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity);
    }

    // TODO: push? But think about how to maintain NFC and other guarantees

    /// Returns a byte slice of this `TextString`'s contents.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    // TODO: truncate, pop, remove, retain, insert? ditto
    // TODO: insert_str? We could do CGJ's where needed there?

    /// Returns a mutable reference to the contents of this `TextString`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, it may
    /// cause memory unsafety issues with future users of the String, as the
    /// rest of this crate assumes that `TextString`s are valid Basic Text.
    #[inline]
    pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        self.0.as_mut_vec()
    }

    /// Returns the length of this `TextString`, in bytes, not `char`s or
    /// graphemes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if this `TextString` has a length of zero, and `false`
    /// otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // TODO: split_off?

    /// Truncates this `String`, removing all contents.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    // TODO: drain, replace_range?

    /// Converts this `TextString` into a `Box<str>`.
    #[inline]
    pub fn into_boxed_utf8(self) -> Box<str> {
        self.0.into_boxed_str()
    }

    /// Converts this `TextString` into a `Box<TextStr>`.
    #[inline]
    pub fn into_boxed_str(self) -> Box<TextStr> {
        let slice = self.into_boxed_utf8();
        unsafe { TextStr::from_boxed_text_unchecked(slice) }
    }
}

#[cold]
fn compute_valid_up_to(s: &str) -> usize {
    // Binary search in `s` for the place where the error starts. We do
    // this after the fact rather than tracking the positions of everything
    // as we go, because tracking the positions through multiple iterators
    // is complex.
    let mut begin = 0;
    let mut end = s.len();
    while begin != end {
        let mut mid = begin + (end - begin) / 2;
        while !s.is_char_boundary(mid) {
            mid -= 1;
        }
        if mid == begin {
            mid = begin + (end - begin) / 2 + 1;
            while !s.is_char_boundary(mid) {
                mid += 1;
            }
            if mid == end {
                break;
            }
        }
        let substr = &s[..mid];
        if is_basic_text(substr) {
            begin = mid;
        } else {
            end = mid;
        }
    }
    begin
}

impl AsRef<[u8]> for TextString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<OsStr> for TextString {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        let s: &str = self.as_ref();
        s.as_ref()
    }
}

impl AsRef<Path> for TextString {
    #[inline]
    fn as_ref(&self) -> &Path {
        let s: &str = self.as_ref();
        s.as_ref()
    }
}

impl AsRef<str> for TextString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_utf8()
    }
}

impl AsRef<TextStr> for TextString {
    #[inline]
    fn as_ref(&self) -> &TextStr {
        self
    }
}

impl Clone for TextString {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for TextString {
    #[inline]
    fn default() -> Self {
        Self(String::default())
    }
}

impl Deref for TextString {
    type Target = TextStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { TextStr::from_text_unchecked(&*self.0) }
    }
}

impl DerefMut for TextString {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { TextStr::from_text_unchecked_mut(&mut *self.0) }
    }
}

impl Borrow<TextStr> for TextString {
    #[inline]
    fn borrow(&self) -> &TextStr {
        self
    }
}

impl BorrowMut<TextStr> for TextString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut TextStr {
        self
    }
}

impl AsMut<TextStr> for TextString {
    #[inline]
    fn as_mut(&mut self) -> &mut TextStr {
        self
    }
}

impl Add<&TextStr> for TextString {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &TextStr) -> Self::Output {
        self.push_str(other);
        self
    }
}

impl AddAssign<&TextStr> for TextString {
    #[inline]
    fn add_assign(&mut self, other: &TextStr) {
        self.push_str(other);
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

impl<'a> PartialEq<TextString> for Cow<'a, TextStr> {
    #[inline]
    fn eq(&self, other: &TextString) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<str> for TextString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<String> for TextString {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<TextString> for String {
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

// TODO: impl Extend for TextString?

impl TextStr {
    /// Converts a slice of bytes to a text string slice.
    #[inline]
    pub fn from_text_bytes(b: &[u8]) -> Result<&Self, TextError> {
        Self::from_text(str::from_utf8(b)?)
    }

    /// Converts a string slice to a text string slice.
    #[inline]
    pub fn from_text(s: &str) -> Result<&Self, TextError> {
        if !is_basic_text(s) {
            let valid_up_to = compute_valid_up_to(s);
            return Err(TextError { valid_up_to });
        }

        Ok(unsafe { Self::from_text_unchecked(s) })
    }

    /// Converts a mutable slice of bytes to a mutable text string slice.
    #[inline]
    pub fn from_text_bytes_mut(b: &mut [u8]) -> Result<&mut Self, TextError> {
        Self::from_text_mut(str::from_utf8_mut(b)?)
    }

    /// Converts a mutable string slice to a mutable text string slice.
    #[inline]
    pub fn from_text_mut(s: &mut str) -> Result<&mut Self, TextError> {
        if !is_basic_text(s) {
            let valid_up_to = compute_valid_up_to(s);
            return Err(TextError { valid_up_to });
        }

        Ok(unsafe { Self::from_text_unchecked_mut(s) })
    }

    /// Converts a slice of bytes to a text string slice without checking that
    /// the string contains valid Basic Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextStr`s
    /// are valid Basic Text.
    #[inline]
    pub unsafe fn from_text_bytes_unchecked(b: &[u8]) -> &Self {
        Self::from_text_unchecked(str::from_utf8_unchecked(b))
    }

    /// Converts a string slice to a text string slice without checking that
    /// the string contains valid Basic Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextStr`s
    /// are valid Basic Text.
    #[inline]
    pub unsafe fn from_text_unchecked(s: &str) -> &Self {
        let ptr: *const str = s;
        &*(ptr as *const Self)
    }

    /// Converts a slice of bytes to a text string slice without checking that
    /// the string contains valid Basic Text; mutable version.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextStr`s
    /// are valid Basic Text.
    #[inline]
    pub unsafe fn from_text_bytes_unchecked_mut(b: &mut [u8]) -> &mut Self {
        Self::from_text_unchecked_mut(str::from_utf8_unchecked_mut(b))
    }

    /// Converts a string slice to a text string slice without checking that
    /// the string contains valid Basic Text; mutable version.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the string
    /// passed to it is valid Basic Text. If this constraint is violated,
    /// undefined behavior results, as the rest of this crate assumes that
    /// `&TextStr`s are valid Basic Text.
    #[inline]
    pub unsafe fn from_text_unchecked_mut(s: &mut str) -> &mut Self {
        let ptr: *mut str = s;
        &mut *(ptr as *mut Self)
    }

    /// Converts a boxed slice of bytes to a boxed text string slice without
    /// checking that the string contains valid basic text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextStr`s
    /// are valid Basic Text.
    #[inline]
    pub unsafe fn from_boxed_text_bytes_unchecked(v: Box<[u8]>) -> Box<Self> {
        let ptr = Box::into_raw(v);
        Box::from_raw(ptr as *mut Self)
    }

    /// Converts a boxed string slice to a boxed text string slice without
    /// checking that the string contains valid basic text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextStr`s
    /// are valid Basic Text.
    #[inline]
    pub unsafe fn from_boxed_text_unchecked(v: Box<str>) -> Box<Self> {
        let ptr = Box::into_raw(v);
        Box::from_raw(ptr as *mut Self)
    }

    /// Returns the length of `self`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if `self` has a length of zero bytes.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // TODO: is_char_boundary?

    /// Converts a text string slice to a byte slice.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Converts a mutable text string slice to a mutable byte slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the content of the slice is valid
    /// Basic Text before the borrow ends and the underlying `TextStr` is used.
    ///
    /// Use of a `TextStr` whose contents are not valid Basic Text is undefined
    /// behavior.
    #[inline]
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.0.as_bytes_mut()
    }

    /// Converts a text string slice to a raw pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Converts a mutable text string slice to a raw pointer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    /// Extracts a UTF-8 string slice containing the entire `TextStr`.
    #[inline]
    pub fn as_utf8(&self) -> &str {
        &self.0
    }

    /// Divide one text string slice into two at an index.
    #[inline]
    pub fn split_at(&self, mid: usize) -> (&Self, &Self) {
        let (prefix, rest) = self.0.split_at(mid);
        // TODO: More efficiently check for validity at the split point.
        (
            Self::from_text(prefix).unwrap(),
            Self::from_text(rest).unwrap(),
        )
    }

    /// Divide one mutable text string slice into two at an index.
    #[inline]
    pub fn split_at_mut(&mut self, mid: usize) -> (&mut Self, &mut Self) {
        let (prefix, rest) = self.0.split_at_mut(mid);
        // TODO: More efficiently check for validity at the split point.
        (
            Self::from_text_mut(prefix).unwrap(),
            Self::from_text_mut(rest).unwrap(),
        )
    }

    // TODO: get*, slice*

    /// Returns an iterator over the `char`s of a text string slice.
    #[inline]
    pub fn chars(&self) -> Chars {
        self.0.chars()
    }

    /// Returns an iterator over the `char`s of a text string slice, and their
    /// positions.
    #[inline]
    pub fn char_indices(&self) -> CharIndices {
        self.0.char_indices()
    }

    /// An iterator over the bytes of a text string slice.
    #[inline]
    pub fn bytes(&self) -> Bytes {
        self.0.bytes()
    }

    // TODO: split*?

    /// An iterator over the lines of a text string, as text string slices.
    #[inline]
    pub fn lines(&self) -> Lines {
        self.0.lines()
    }

    /// Returns an iterator of `u16` over the string encoded as Basic Text.
    #[inline]
    pub fn encode_utf16(&self) -> EncodeUtf16<'_> {
        self.0.encode_utf16()
    }

    /// Returns `true` if the given pattern matches a sub-slice of this
    /// text string slice.
    ///
    /// Returns `false` if it does not.
    #[cfg(pattern)]
    #[inline]
    pub fn contains<'a, P>(&'a self, pat: P) -> bool
    where
        P: Pattern<'a>,
    {
        self.0.contains(pat)
    }

    /// Returns `true` if the given pattern matches a prefix of this
    /// text string slice.
    ///
    /// Returns `false` if it does not.
    #[cfg(pattern)]
    #[inline]
    pub fn starts_with<'a, P>(&'a self, pat: P) -> bool
    where
        P: Pattern<'a>,
    {
        self.0.starts_with(pat)
    }

    /// Returns `true` if the given pattern matches a suffix of this
    /// text string slice.
    ///
    /// Returns `false` if it does not.
    #[cfg(pattern)]
    #[inline]
    pub fn ends_with<'a, P>(&'a self, pat: P) -> bool
    where
        P: Pattern<'a>,
        <P as Pattern<'a>>::Searcher: ReverseSearcher<'a>,
    {
        self.0.ends_with(pat)
    }

    /// Returns the byte index of the first character of this text string slice
    /// that matches the pattern.
    ///
    /// Returns `None` if the pattern doesn't match.
    #[cfg(pattern)]
    #[inline]
    pub fn find<'a, P>(&'a self, pat: P) -> Option<usize>
    where
        P: Pattern<'a>,
    {
        self.0.find(pat)
    }

    /// Returns the byte index for the first character of the rightmost match of
    /// the pattern in this text string slice.
    ///
    /// Returns `None` if the pattern doesn't match.
    #[cfg(pattern)]
    #[inline]
    pub fn rfind<'a, P>(&'a self, pat: P) -> Option<usize>
    where
        P: Pattern<'a>,
        <P as Pattern<'a>>::Searcher: ReverseSearcher<'a>,
    {
        self.0.rfind(pat)
    }

    // TODO: *split*?

    /// An iterator over the disjoint matches of a pattern within the given
    /// text string slice.
    #[cfg(pattern)]
    #[inline]
    pub fn matches<'a, P>(&'a self, pat: P) -> Matches<'a, P>
    where
        P: Pattern<'a>,
    {
        self.0.matches(pat)
    }

    /// An iterator over the disjoint matches of a pattern within this
    /// text string slice, yielded in reverse order.
    #[cfg(pattern)]
    #[inline]
    pub fn rmatches<'a, P>(&'a self, pat: P) -> RMatches<'a, P>
    where
        P: Pattern<'a>,
        <P as Pattern<'a>>::Searcher: ReverseSearcher<'a>,
    {
        self.0.rmatches(pat)
    }

    /// An iterator over the disjoint matches of a pattern within this
    /// text string slice as well as the index that the match starts at.
    #[cfg(pattern)]
    #[inline]
    pub fn match_indices<'a, P>(&'a self, pat: P) -> MatchIndices<'a, P>
    where
        P: Pattern<'a>,
    {
        self.0.match_indices(pat)
    }

    /// An iterator over the disjoint matches of a pattern within `self`,
    /// yielded in reverse order along with the index of the match.
    #[cfg(pattern)]
    #[inline]
    pub fn rmatch_indices<'a, P>(&'a self, pat: P) -> RMatchIndices<'a, P>
    where
        P: Pattern<'a>,
        <P as Pattern<'a>>::Searcher: ReverseSearcher<'a>,
    {
        self.0.rmatch_indices(pat)
    }

    /// Returns a text string slice with leading and trailing whitespace removed.
    #[inline]
    pub fn trim(&self) -> &Self {
        unsafe { Self::from_text_unchecked(self.0.trim()) }
    }

    /// Returns a text string slice with leading whitespace removed.
    #[inline]
    pub fn trim_start(&self) -> &Self {
        unsafe { Self::from_text_unchecked(self.0.trim_start()) }
    }

    /// Returns a text string slice with trailing whitespace removed.
    #[inline]
    pub fn trim_end(&self) -> &Self {
        unsafe { Self::from_text_unchecked(self.0.trim_end()) }
    }

    // TODO: trim_matches, trim_start_matches, strip_prefix, strip_suffix, trim_end_matches?

    /// Parses this text string slice into another type.
    #[inline]
    pub fn parse<F>(&self) -> Result<F, <F as FromStr>::Err>
    where
        F: FromStr,
    {
        // TODO: Is there a way we could use `TryFrom<&TextStr>` to allow
        // parsers to work from a `TextStr` instead of just a `str`?
        self.0.parse()
    }

    /// Checks if all characters in this text string are within the ASCII
    /// range.
    #[inline]
    pub fn is_ascii(&self) -> bool {
        self.0.is_ascii()
    }

    /// Checks that two text strings are an ASCII case-insensitive match.
    #[inline]
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }

    /// Converts a `Box<TextStr`> into a `Box<[u8]>` without copying or
    /// allocating.
    #[inline]
    pub fn into_boxed_bytes(self: Box<Self>) -> Box<[u8]> {
        self.into_boxed_str().into_boxed_bytes()
    }

    /// Converts a `Box<TextStr>` into a `Box<str>` without copying or
    /// allocating.
    #[inline]
    pub fn into_boxed_str(self: Box<Self>) -> Box<str> {
        self.into()
    }

    /// Converts a `Box<TextStr>` into a `String` without copying or allocating.
    #[inline]
    pub fn into_utf8(self: Box<Self>) -> String {
        let slice = Box::<[u8]>::from(self);
        unsafe { String::from_utf8_unchecked(slice.into_vec()) }
    }

    /// Converts a `Box<TextStr>` into a `TextString` without copying or
    /// allocating.
    #[inline]
    pub fn into_string(self: Box<Self>) -> TextString {
        unsafe { TextString::from_text_unchecked(Self::into_utf8(self)) }
    }

    // TODO: make_ascii_uppercase, make_ascii_lowercase, escape_debug,
    // escape_default, escape_unicode, replace*, to_lowercase, to_uppercase,
    // into_string, repeat, to_ascii_uppercase, to_ascii_lowercase; determine
    // whether these can be done without breaking NFC.
}

impl AsRef<[u8]> for TextStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<OsStr> for TextStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for TextStr {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<str> for TextStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<Self> for TextStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for &TextStr {
    #[inline]
    fn default() -> Self {
        unsafe { TextStr::from_text_unchecked("") }
    }
}

impl Default for &mut TextStr {
    #[inline]
    fn default() -> Self {
        unsafe { TextStr::from_text_bytes_unchecked_mut(&mut []) }
    }
}

impl Display for TextStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

// TODO: impl Index* for TextStr?

impl Ord for TextStr {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
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

impl<'a> PartialEq<str> for TextStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<String> for TextStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<TextStr> for String {
    #[inline]
    fn eq(&self, other: &TextStr) -> bool {
        self.eq(&other.0)
    }
}

// TODO: all the PartialEq impls for TextStr

impl PartialOrd<Self> for TextStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// TODO: Pattern for TextStr

impl ToOwned for TextStr {
    type Owned = TextString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        TextString(self.0.to_owned())
    }
}

impl ToSocketAddrs for TextStr {
    type Iter = vec::IntoIter<SocketAddr>;

    #[inline]
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        self.0.to_socket_addrs()
    }
}

impl TextError {
    // Returns the index in the given string up to which valid Basic Text was
    // verified.
    pub fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }
}

impl From<Utf8Error> for TextError {
    fn from(err: Utf8Error) -> Self {
        Self {
            valid_up_to: err.valid_up_to(),
        }
    }
}

impl Display for TextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "TODO: Display for TextError: {:?}", self)
    }
}

impl Error for TextError {}

impl FromTextError {
    /// Returns a slice of `u8`s bytes that were attempted to convert to a
    /// `TextString`.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the bytes that were attempted to convert to a `TextString`.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Fetch a `TextError` to get more details about the conversion failure.
    #[inline]
    pub fn text_error(&self) -> TextError {
        self.error
    }
}

impl From<FromUtf8Error> for FromTextError {
    #[inline]
    fn from(err: FromUtf8Error) -> Self {
        let error = err.utf8_error().into();
        let bytes = err.into_bytes();
        Self { bytes, error }
    }
}

impl Display for FromTextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "TODO: Display for FromTextError: {:?}", self)
    }
}

impl Error for FromTextError {}

impl From<Box<TextStr>> for Box<[u8]> {
    #[inline]
    fn from(s: Box<TextStr>) -> Self {
        let ptr = Box::into_raw(s);
        unsafe { Self::from_raw(ptr as *mut [u8]) }
    }
}

impl From<Box<TextStr>> for Box<str> {
    #[inline]
    fn from(s: Box<TextStr>) -> Self {
        let ptr = Box::into_raw(s);
        unsafe { Self::from_raw(ptr as *mut str) }
    }
}

impl From<Box<TextStr>> for TextString {
    #[inline]
    fn from(s: Box<TextStr>) -> Self {
        s.into_string()
    }
}

impl From<&'_ Self> for TextString {
    #[inline]
    fn from(s: &Self) -> Self {
        s.clone()
    }
}

impl From<&'_ mut TextStr> for TextString {
    #[inline]
    fn from(s: &mut TextStr) -> Self {
        s.to_owned()
    }
}

impl From<&'_ TextStr> for TextString {
    #[inline]
    fn from(s: &TextStr) -> Self {
        s.to_owned()
    }
}

impl From<Cow<'_, TextStr>> for Box<TextStr> {
    #[inline]
    fn from(cow: Cow<'_, TextStr>) -> Self {
        match cow {
            Cow::Borrowed(s) => Self::from(s),
            Cow::Owned(s) => Self::from(s),
        }
    }
}

impl From<TextString> for Box<TextStr> {
    #[inline]
    fn from(s: TextString) -> Self {
        s.into_boxed_str()
    }
}

impl Clone for Box<TextStr> {
    #[inline]
    fn clone(&self) -> Self {
        let buf: Box<[u8]> = self.as_bytes().into();
        unsafe { TextStr::from_boxed_text_bytes_unchecked(buf) }
    }
}

impl Default for Box<TextStr> {
    #[inline]
    fn default() -> Self {
        unsafe { TextStr::from_boxed_text_bytes_unchecked(Box::default()) }
    }
}

impl From<&TextStr> for Box<TextStr> {
    #[inline]
    fn from(s: &TextStr) -> Self {
        unsafe { TextStr::from_boxed_text_bytes_unchecked(Box::from(s.as_bytes())) }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for &'a TextStr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let size = u.arbitrary_len::<u8>()?;
        match TextStr::from_text_bytes(&u.peek_bytes(size).unwrap()) {
            Ok(s) => {
                u.bytes(size).unwrap();
                Ok(s)
            }
            Err(e) => {
                let i = e.valid_up_to();
                let valid = u.bytes(i).unwrap();
                let s = unsafe {
                    debug_assert!(TextStr::from_text_bytes(valid).is_ok());
                    TextStr::from_text_bytes_unchecked(valid)
                };
                Ok(s)
            }
        }
    }

    fn arbitrary_take_rest(u: arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let bytes = u.take_rest();
        TextStr::from_text_bytes(bytes)
            .map_err(|_| arbitrary::Error::IncorrectFormat)
            .map(Into::into)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::and(<usize as arbitrary::Arbitrary>::size_hint(depth), (0, None))
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for TextString {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        <&TextStr as arbitrary::Arbitrary>::arbitrary(u).map(Into::into)
    }

    fn arbitrary_take_rest(u: arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        <&TextStr as arbitrary::Arbitrary>::arbitrary_take_rest(u).map(Into::into)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <&TextStr as arbitrary::Arbitrary>::size_hint(depth)
    }
}

#[test]
fn normalize_string() {
    let ring = "\u{030a}";
    let unnormal = "A\u{30a}";
    let unnormal_nl = "A\u{30a}\n";
    let composed = TextStr::from_text("\u{c5}").unwrap();
    let composed_nl = TextStr::from_text("\u{c5}\n").unwrap();

    assert_eq!(TextStr::from_text(unnormal).unwrap_err().valid_up_to(), 1);
    assert_eq!(TextStr::from_text(ring).unwrap_err().valid_up_to(), 0);
    assert_eq!(composed, &TextString::from_text_lossy(unnormal));
    assert_eq!(composed_nl, &TextString::from_text_lossy(unnormal_nl));
}

#[test]
fn validate_string() {
    assert!(TextStr::from_text_bytes(b"").is_ok());
    assert_eq!(
        TextStr::from_text_bytes(b"\xff").unwrap_err().valid_up_to(),
        0
    );
}

#[test]
fn split_escape() {
    //assert_eq!(TextStr::from_text_bytes(b"\x1b[!p").unwrap_err().valid_up_to(), 0);
    assert_eq!(
        TextStr::from_text_bytes(b"\x1b[p")
            .unwrap_err()
            .valid_up_to(),
        0
    );
    assert_eq!(
        TextStr::from_text_bytes(b"\x1b[!")
            .unwrap_err()
            .valid_up_to(),
        0
    );
    assert_eq!(
        TextStr::from_text_bytes(b"\x1b[")
            .unwrap_err()
            .valid_up_to(),
        0
    );
    assert_eq!(
        TextStr::from_text_bytes(b"\x1b").unwrap_err().valid_up_to(),
        0
    );
}
