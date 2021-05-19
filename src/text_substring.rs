//! The `TextSubstring` and `TextSubstr` types.

use crate::{FromTextError, TextError, TextReader, TextWriter};
use basic_text_internals::{
    is_basic_text_substr,
    unicode::{BOM, WJ},
};
use layered_io::Bufferable;
#[cfg(try_reserve)]
use std::collections::TryReserveError;
#[cfg(pattern)]
use std::str::pattern::{Pattern, ReverseSearcher};
use std::{
    borrow::{Borrow, BorrowMut, Cow},
    cmp::Ordering,
    ffi::OsStr,
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
    io::{self, Read, Write},
    net::{SocketAddr, ToSocketAddrs},
    ops::{Deref, DerefMut, Index, Range, RangeFrom, RangeTo},
    path::Path,
    str::{
        self, Bytes, CharIndices, Chars, EncodeUtf16, EscapeDebug, EscapeDefault, EscapeUnicode,
        FromStr, Lines, MatchIndices, Matches, RMatchIndices, RMatches,
    },
    vec,
};
use utf8_io::WriteStr;

/// A substring of a Basic Text string or stream.
///
/// This does not enforce the Basic Text requirements on the beginning or end
/// of a stream, so it can represent substrings of Basic Text.
///
/// This is an owning string similar to [`TextString`], but doesn't enforce the
/// starting and ending requirements, so it can represent substrings. It's
/// accompanied by a borrowing [`TextSubstr`], which plays an analogous role to
/// [`TextStr`].
///
/// [`TextString`]: https://docs.rs/basic-text/latest/basic_text/struct.TextString.html
/// [`TextStr`]: https://docs.rs/basic-text/latest/basic_text/struct.TextStr.html
///
/// # Examples
///
/// You can create a `TextSubstring` from a literal text string with `TextSubstring::from`:
///
/// ```rust
/// use basic_text::{text_substr, TextSubstring};
///
/// let hello = TextSubstring::from(text_substr!("Hello, world!"));
/// ```
///
/// If you have a `String` containing a Basic Text string, you can create a
/// `TextSubstring` from it with the `from_text` method:
///
/// ```rust
/// use basic_text::{text_substr, TextSubstring};
///
/// // a `String`
/// let sparkle_heart = "💖".to_owned();
///
/// // We know this string is valid, so we'll use `unwrap()`.
/// let sparkle_heart = TextSubstring::from_text(sparkle_heart).unwrap();
///
/// assert_eq!(text_substr!("💖"), &sparkle_heart);
/// ```
///
/// If you have a vector of Basic Text bytes, you can create a `String` from it
/// with the `from_text_vec` method:
///
/// ```rust
/// use basic_text::{text_substr, TextSubstring};
///
/// // some bytes, in a vector
/// let sparkle_heart = vec![240, 159, 146, 150];
///
/// // We know these bytes are valid, so we'll use `unwrap()`.
/// let sparkle_heart = TextSubstring::from_text_vec(sparkle_heart).unwrap();
///
/// assert_eq!(text_substr!("💖"), &sparkle_heart);
/// ```
#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct TextSubstring(pub(crate) String);

/// Text substring slices.
///
/// `TextSubstr` is to `TextSubstring` as [`TextStr`] is to `TextString`. It is
/// usually used for borrowing, in the form of `&TextSubstr`.
///
/// [`TextString`]: https://docs.rs/basic-text/latest/basic_text/struct.TextString.html
/// [`TextStr`]: https://docs.rs/basic-text/latest/basic_text/struct.TextStr.html
#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct TextSubstr(pub(crate) str);

impl TextSubstring {
    /// Creates a new empty `TextSubstring`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Creates a new empty `TextSubstring` with a particular capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    /// Converts a vector of bytes to a `TextSubstring`.
    #[inline]
    pub fn from_text_vec(vec: Vec<u8>) -> Result<Self, FromTextError> {
        Self::from_text(String::from_utf8(vec)?)
    }

    /// Converts a `String` to a `TextSubstring`.
    #[inline]
    pub fn from_text(s: String) -> Result<Self, FromTextError> {
        let bytes: Vec<u8> = Vec::new();
        let mut writer = TextWriter::new(bytes);

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
    pub fn from_text_bytes_lossy(v: &[u8]) -> Cow<TextSubstr> {
        // TODO: optimize away the temporary String here
        Cow::Owned(Self::from_text_lossy(&String::from_utf8_lossy(v)).into_owned())
    }

    /// Converts a string to Basic Text, including invalid characters.
    #[inline]
    #[must_use]
    pub fn from_text_lossy(mut v: &str) -> Cow<TextSubstr> {
        // TODO: If `v` is already valid, fast-path to `Cow::Borrowed(v)`.
        // TODO: Also, this currently redoes UTF-8 validation for `v`.
        let mut reader = TextReader::new(v.as_bytes());
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

    /// Converts a vector of bytes to a `TextSubstring` without checking that the
    /// string contains valid Basic Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextSubstr`s
    /// are valid Basic Text.
    #[inline]
    #[must_use]
    pub unsafe fn from_text_vec_unchecked(vec: Vec<u8>) -> Self {
        Self::from_text_unchecked(String::from_utf8_unchecked(vec))
    }

    /// Converts a `String` to a `TextSubstring` without checking that the string
    /// contains valid Basic Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&TextSubstr`s
    /// are valid Basic Text.
    #[inline]
    #[must_use]
    pub const unsafe fn from_text_unchecked(s: String) -> Self {
        Self(s)
    }

    /// Converts a `TextSubstring` into a `String`.
    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    /// Converts a String into a byte vector.
    #[inline]
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    /// Extracts a UTF-8 string slice containing the entire `TextSubstring`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extracts a Basic Text string slice containing the entire `TextSubstring`.
    #[inline]
    #[must_use]
    pub fn as_text(&self) -> &TextSubstr {
        self
    }

    /// Converts a `TextSubstring` into a mutable Basic Text substring slice.
    #[inline]
    #[must_use]
    pub fn as_mut_text(&mut self) -> &mut TextSubstr {
        self
    }

    /// Returns this `TextSubstring`'s capacity, in bytes.
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Ensures that this `TextSubstring`'s capacity is at least `additional`
    /// bytes larger than its length.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Ensures that this `TextSubstring`'s capacity is `additional` bytes larger
    /// than its length.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    /// Tries to reserve capacity for at least `additional` more elements to
    /// be inserted in the given `TextSubstring`.
    #[cfg(try_reserve)]
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// Tries to reserves the minimum capacity for exactly `additional` more
    /// elements to be inserted in the given `TextSubstring`.
    #[cfg(try_reserve)]
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    /// Shrinks the capacity of this `TextSubstring` to match its length.
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

    /// Returns a byte slice of this `TextSubstring`'s contents.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    // TODO: truncate, pop, remove, retain, insert? ditto
    // TODO: insert_str? We could do CGJ's where needed there?

    /// Returns a mutable reference to the contents of this `TextSubstring`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, it may
    /// cause memory unsafety issues with future users of the String, as the
    /// rest of this crate assumes that `TextSubstring`s are valid Basic Text.
    #[inline]
    pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        self.0.as_mut_vec()
    }

    /// Returns a mutable reference to the contents of this `TextSubstring`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Basic Text. If this constraint is violated, it may
    /// cause memory unsafety issues with future users of the String, as the
    /// rest of this crate assumes that `TextSubstring`s are valid Basic Text.
    #[inline]
    pub unsafe fn as_mut_string(&mut self) -> &mut String {
        &mut self.0
    }

    /// Returns the length of this `TextSubstring`, in bytes, not `char`s or
    /// graphemes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if this `TextSubstring` has a length of zero, and `false`
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

    /// Converts this `TextSubstring` into a `Box<str>`.
    #[inline]
    pub fn into_boxed_str(self) -> Box<str> {
        self.0.into_boxed_str()
    }

    /// Converts this `TextSubstring` into a `Box<TextSubstr>`.
    #[inline]
    pub fn into_boxed_text(self) -> Box<TextSubstr> {
        let slice = self.into_boxed_str();
        unsafe { TextSubstr::from_boxed_text_unchecked(slice) }
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
        if is_basic_text_substr(substr) {
            begin = mid;
        } else {
            end = mid;
        }
    }
    begin
}

impl AsRef<[u8]> for TextSubstring {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<OsStr> for TextSubstring {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        let s: &str = self.as_ref();
        s.as_ref()
    }
}

impl AsRef<Path> for TextSubstring {
    #[inline]
    fn as_ref(&self) -> &Path {
        let s: &str = self.as_ref();
        s.as_ref()
    }
}

impl AsRef<str> for TextSubstring {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<TextSubstr> for TextSubstring {
    #[inline]
    fn as_ref(&self) -> &TextSubstr {
        self
    }
}

impl Clone for TextSubstring {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for TextSubstring {
    #[inline]
    fn default() -> Self {
        Self(String::default())
    }
}

impl Deref for TextSubstring {
    type Target = TextSubstr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { TextSubstr::from_text_unchecked(&*self.0) }
    }
}

impl DerefMut for TextSubstring {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { TextSubstr::from_text_unchecked_mut(&mut *self.0) }
    }
}

impl Borrow<TextSubstr> for TextSubstring {
    #[inline]
    fn borrow(&self) -> &TextSubstr {
        self
    }
}

impl BorrowMut<TextSubstr> for TextSubstring {
    #[inline]
    fn borrow_mut(&mut self) -> &mut TextSubstr {
        self
    }
}

impl AsMut<TextSubstr> for TextSubstring {
    #[inline]
    fn as_mut(&mut self) -> &mut TextSubstr {
        self
    }
}

impl TextSubstr {
    /// Converts a slice of bytes to a text string slice.
    #[inline]
    pub fn from_text_bytes(b: &[u8]) -> Result<&Self, TextError> {
        Self::from_text(str::from_utf8(b)?)
    }

    /// Converts a string slice to a text string slice.
    #[inline]
    pub fn from_text(s: &str) -> Result<&Self, TextError> {
        if !is_basic_text_substr(s) {
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
        if !is_basic_text_substr(s) {
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
    /// behavior results, as the rest of this crate assumes that `&TextSubstr`s
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
    /// behavior results, as the rest of this crate assumes that `&TextSubstr`s
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
    /// behavior results, as the rest of this crate assumes that `&TextSubstr`s
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
    /// `&TextSubstr`s are valid Basic Text.
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
    /// behavior results, as the rest of this crate assumes that `&TextSubstr`s
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
    /// behavior results, as the rest of this crate assumes that `&TextSubstr`s
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
    /// Basic Text before the borrow ends and the underlying `TextSubstr` is used.
    ///
    /// Use of a `TextSubstr` whose contents are not valid Basic Text is undefined
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

    /// Extracts a UTF-8 string slice containing the entire `TextSubstr`.
    #[inline]
    pub fn as_str(&self) -> &str {
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

    // TODO: get*

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
    ///
    /// TODO: There should be a `TextLines` which yields `&TextSubstr`s.
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

    /// Returns `true` if the given pattern matches a sub-slice of this
    /// text string slice.
    ///
    /// Returns `false` if it does not.
    #[cfg(not(pattern))]
    #[inline]
    pub fn contains<'a>(&'a self, pat: &str) -> bool {
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

    /// Returns `true` if the given pattern matches a prefix of this
    /// text string slice.
    ///
    /// Returns `false` if it does not.
    #[cfg(not(pattern))]
    #[inline]
    pub fn starts_with<'a, P>(&'a self, pat: &str) -> bool {
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

    /// Returns `true` if the given pattern matches a suffix of this
    /// text string slice.
    ///
    /// Returns `false` if it does not.
    #[cfg(not(pattern))]
    #[inline]
    pub fn ends_with<'a>(&'a self, pat: &str) -> bool {
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

    /// Returns the byte index of the first character of this text string slice
    /// that matches the pattern.
    ///
    /// Returns `None` if the pattern doesn't match.
    #[cfg(not(pattern))]
    #[inline]
    pub fn find<'a>(&'a self, pat: &str) -> Option<usize> {
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

    /// Returns the byte index for the first character of the rightmost match of
    /// the pattern in this text string slice.
    ///
    /// Returns `None` if the pattern doesn't match.
    #[cfg(not(pattern))]
    #[inline]
    pub fn rfind<'a>(&'a self, pat: &str) -> Option<usize> {
        self.0.rfind(pat)
    }

    // TODO: *split*?

    /// An iterator over the disjoint matches of a pattern within the given
    /// text string slice.
    ///
    /// TODO: There should be a `TextMatches` which yields `&TextSubstr`s.
    #[cfg(pattern)]
    #[inline]
    pub fn matches<'a, P>(&'a self, pat: P) -> Matches<'a, P>
    where
        P: Pattern<'a>,
    {
        self.0.matches(pat)
    }

    /// An iterator over the disjoint matches of a pattern within the given
    /// text string slice.
    #[cfg(not(pattern))]
    #[inline]
    pub fn matches<'a>(&'a self, pat: &'a str) -> Matches<'a, &str> {
        self.0.matches(pat)
    }

    /// An iterator over the disjoint matches of a pattern within this
    /// text string slice, yielded in reverse order.
    ///
    /// TODO: There should be a `TextRMatches` which yields `&TextSubstr`s.
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
    /// text string slice, yielded in reverse order.
    #[cfg(not(pattern))]
    #[inline]
    pub fn rmatches<'a>(&'a self, pat: &'a str) -> RMatches<'a, &str> {
        self.0.rmatches(pat)
    }

    /// An iterator over the disjoint matches of a pattern within this
    /// text string slice as well as the index that the match starts at.
    ///
    /// TODO: There should be a `TextMatchIndices` which yields `&TextSubstr`s.
    #[cfg(pattern)]
    #[inline]
    pub fn match_indices<'a, P>(&'a self, pat: P) -> MatchIndices<'a, P>
    where
        P: Pattern<'a>,
    {
        self.0.match_indices(pat)
    }

    /// An iterator over the disjoint matches of a pattern within this
    /// text string slice as well as the index that the match starts at.
    #[cfg(not(pattern))]
    #[inline]
    pub fn match_indices<'a>(&'a self, pat: &'a str) -> MatchIndices<'a, &str> {
        self.0.match_indices(pat)
    }

    /// An iterator over the disjoint matches of a pattern within `self`,
    /// yielded in reverse order along with the index of the match.
    ///
    /// TODO: There should be a `TextRMatchIndices` which yields `&TextSubstr`s.
    #[cfg(pattern)]
    #[inline]
    pub fn rmatch_indices<'a, P>(&'a self, pat: P) -> RMatchIndices<'a, P>
    where
        P: Pattern<'a>,
        <P as Pattern<'a>>::Searcher: ReverseSearcher<'a>,
    {
        self.0.rmatch_indices(pat)
    }

    /// An iterator over the disjoint matches of a pattern within `self`,
    /// yielded in reverse order along with the index of the match.
    #[cfg(not(pattern))]
    #[inline]
    pub fn rmatch_indices<'a>(&'a self, pat: &'a str) -> RMatchIndices<'a, &str> {
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
        // TODO: Is there a way we could use `TryFrom<&TextSubstr>` to allow
        // parsers to work from a `TextSubstr` instead of just a `str`?
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

    /// Converts a `Box<TextSubstr`> into a `Box<[u8]>` without copying or
    /// allocating.
    #[inline]
    pub fn into_boxed_bytes(self: Box<Self>) -> Box<[u8]> {
        self.into_boxed_text().into_boxed_bytes()
    }

    /// Converts a `Box<TextSubstr>` into a `Box<str>` without copying or
    /// allocating.
    #[inline]
    pub fn into_boxed_text(self: Box<Self>) -> Box<str> {
        self.into()
    }

    /// Converts a `Box<TextSubstr>` into a `String` without copying or allocating.
    #[inline]
    pub fn into_string(self: Box<Self>) -> String {
        let slice = Box::<[u8]>::from(self);
        unsafe { String::from_utf8_unchecked(slice.into_vec()) }
    }

    /// Converts a `Box<TextSubstr>` into a `TextSubstring` without copying or
    /// allocating.
    #[inline]
    pub fn into_text_string(self: Box<Self>) -> TextSubstring {
        unsafe { TextSubstring::from_text_unchecked(Self::into_string(self)) }
    }

    /// Creates a new [`TextSubstring`] by repeating a string `n` times.
    pub fn repeat(&self, n: usize) -> TextSubstring {
        unsafe { TextSubstring::from_text_vec_unchecked(self.as_bytes().repeat(n)) }
    }

    /// Return an iterator that escapes each `char` in `self` with
    /// [`char::escape_debug`].
    #[inline]
    pub fn escape_debug(&self) -> EscapeDebug<'_> {
        self.0.escape_debug()
    }

    /// Return an iterator that escapes each `char` in `self` with
    /// [`char::escape_default`].
    #[inline]
    pub fn escape_default(&self) -> EscapeDefault<'_> {
        self.0.escape_default()
    }

    /// Return an iterator that escapes each `char` in `self` with
    /// [`char::escape_unicode`].
    #[inline]
    pub fn escape_unicode(&self) -> EscapeUnicode {
        self.0.escape_unicode()
    }

    // TODO: make_ascii_uppercase, make_ascii_lowercase, replace*,
    // to_lowercase, to_uppercase, to_ascii_uppercase, to_ascii_lowercase;
    // determine whether these can be done without breaking NFC.
}

impl AsRef<[u8]> for TextSubstr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<OsStr> for TextSubstr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for TextSubstr {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<str> for TextSubstr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<Self> for TextSubstr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for &TextSubstr {
    #[inline]
    fn default() -> Self {
        unsafe { TextSubstr::from_text_unchecked("") }
    }
}

impl Default for &mut TextSubstr {
    #[inline]
    fn default() -> Self {
        unsafe { TextSubstr::from_text_bytes_unchecked_mut(&mut []) }
    }
}

impl Display for TextSubstr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Ord for TextSubstr {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd<Self> for TextSubstr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialOrd<Self> for TextSubstring {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// TODO: Pattern for TextSubstr

impl ToOwned for TextSubstr {
    type Owned = TextSubstring;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        TextSubstring(self.0.to_owned())
    }
}

impl ToSocketAddrs for TextSubstr {
    type Iter = vec::IntoIter<SocketAddr>;

    #[inline]
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        self.0.to_socket_addrs()
    }
}

impl From<Box<TextSubstr>> for Box<[u8]> {
    #[inline]
    fn from(s: Box<TextSubstr>) -> Self {
        let ptr = Box::into_raw(s);
        unsafe { Self::from_raw(ptr as *mut [u8]) }
    }
}

impl From<Box<TextSubstr>> for Box<str> {
    #[inline]
    fn from(s: Box<TextSubstr>) -> Self {
        let ptr = Box::into_raw(s);
        unsafe { Self::from_raw(ptr as *mut str) }
    }
}

impl From<Box<TextSubstr>> for TextSubstring {
    #[inline]
    fn from(s: Box<TextSubstr>) -> Self {
        s.into_text_string()
    }
}

impl From<&'_ Self> for TextSubstring {
    #[inline]
    fn from(s: &Self) -> Self {
        s.clone()
    }
}

impl From<&'_ mut TextSubstr> for TextSubstring {
    #[inline]
    fn from(s: &mut TextSubstr) -> Self {
        s.to_owned()
    }
}

impl From<&'_ TextSubstr> for TextSubstring {
    #[inline]
    fn from(s: &TextSubstr) -> Self {
        s.to_owned()
    }
}

impl From<Cow<'_, TextSubstr>> for Box<TextSubstr> {
    #[inline]
    fn from(cow: Cow<'_, TextSubstr>) -> Self {
        match cow {
            Cow::Borrowed(s) => Self::from(s),
            Cow::Owned(s) => Self::from(s),
        }
    }
}

impl From<TextSubstring> for Box<TextSubstr> {
    #[inline]
    fn from(s: TextSubstring) -> Self {
        s.into_boxed_text()
    }
}

impl Clone for Box<TextSubstr> {
    #[inline]
    fn clone(&self) -> Self {
        let buf: Box<[u8]> = self.as_bytes().into();
        unsafe { TextSubstr::from_boxed_text_bytes_unchecked(buf) }
    }
}

impl Default for Box<TextSubstr> {
    #[inline]
    fn default() -> Self {
        unsafe { TextSubstr::from_boxed_text_bytes_unchecked(Box::default()) }
    }
}

impl From<&TextSubstr> for Box<TextSubstr> {
    #[inline]
    fn from(s: &TextSubstr) -> Self {
        unsafe { TextSubstr::from_boxed_text_bytes_unchecked(Box::from(s.as_bytes())) }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for &'a TextSubstr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let size = u.arbitrary_len::<u8>()?;
        match TextSubstr::from_text_bytes(&u.peek_bytes(size).unwrap()) {
            Ok(s) => {
                u.bytes(size).unwrap();
                Ok(s)
            }
            Err(e) => {
                let i = e.valid_up_to();
                let valid = u.bytes(i).unwrap();
                let s = unsafe {
                    debug_assert!(TextSubstr::from_text_bytes(valid).is_ok());
                    TextSubstr::from_text_bytes_unchecked(valid)
                };
                Ok(s)
            }
        }
    }

    fn arbitrary_take_rest(u: arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let bytes = u.take_rest();
        TextSubstr::from_text_bytes(bytes)
            .map_err(|_| arbitrary::Error::IncorrectFormat)
            .map(Into::into)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::and(<usize as arbitrary::Arbitrary>::size_hint(depth), (0, None))
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for TextSubstring {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        <&TextSubstr as arbitrary::Arbitrary>::arbitrary(u).map(Into::into)
    }

    fn arbitrary_take_rest(u: arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        <&TextSubstr as arbitrary::Arbitrary>::arbitrary_take_rest(u).map(Into::into)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <&TextSubstr as arbitrary::Arbitrary>::size_hint(depth)
    }
}

impl Index<Range<usize>> for TextSubstring {
    type Output = TextSubstr;

    #[inline]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        unsafe { TextSubstr::from_text_unchecked(self.0.index(index)) }
    }
}

impl Index<Range<usize>> for TextSubstr {
    type Output = TextSubstr;

    #[inline]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        unsafe { TextSubstr::from_text_unchecked(self.0.index(index)) }
    }
}

impl Index<RangeTo<usize>> for TextSubstring {
    type Output = TextSubstr;

    #[inline]
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        unsafe { TextSubstr::from_text_unchecked(self.0.index(index)) }
    }
}

impl Index<RangeTo<usize>> for TextSubstr {
    type Output = TextSubstr;

    #[inline]
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        unsafe { TextSubstr::from_text_unchecked(self.0.index(index)) }
    }
}

impl Index<RangeFrom<usize>> for TextSubstring {
    type Output = TextSubstr;

    #[inline]
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        unsafe { TextSubstr::from_text_unchecked(self.0.index(index)) }
    }
}

impl Index<RangeFrom<usize>> for TextSubstr {
    type Output = TextSubstr;

    #[inline]
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        unsafe { TextSubstr::from_text_unchecked(self.0.index(index)) }
    }
}

#[test]
fn normalize_string() {
    let ring = "\u{30a}";
    let unnormal = "A\u{30a}";
    let unnormal_nl = "A\u{30a}\n";
    let composed = TextSubstr::from_text("\u{c5}").unwrap();
    let composed_nl = TextSubstr::from_text("\u{c5}\n").unwrap();

    assert_eq!(
        TextSubstr::from_text(unnormal).unwrap_err().valid_up_to(),
        1
    );
    TextSubstr::from_text(ring).unwrap();
    assert_eq!(composed, &TextSubstring::from_text_lossy(unnormal));
    assert_eq!(composed_nl, &TextSubstring::from_text_lossy(unnormal_nl));
}

#[test]
fn validate_string() {
    assert!(TextSubstr::from_text_bytes(b"").is_ok());
    assert_eq!(
        TextSubstr::from_text_bytes(b"\xff")
            .unwrap_err()
            .valid_up_to(),
        0
    );
}

#[test]
fn split_escape() {
    //assert_eq!(TextSubstr::from_text_bytes(b"\x1b[!p").unwrap_err().valid_up_to(), 0);
    assert_eq!(
        TextSubstr::from_text_bytes(b"\x1b[p")
            .unwrap_err()
            .valid_up_to(),
        0
    );
    assert_eq!(
        TextSubstr::from_text_bytes(b"\x1b[!")
            .unwrap_err()
            .valid_up_to(),
        0
    );
    assert_eq!(
        TextSubstr::from_text_bytes(b"\x1b[")
            .unwrap_err()
            .valid_up_to(),
        0
    );
    assert_eq!(
        TextSubstr::from_text_bytes(b"\x1b")
            .unwrap_err()
            .valid_up_to(),
        0
    );
}
