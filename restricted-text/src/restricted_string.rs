use crate::{
    RestrictedReader, RestrictedWriter
};
use layered_io::{LayeredReader, LayeredWriter};
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
    io::{self, Read},
    net::{SocketAddr, ToSocketAddrs},
    ops::{Add, AddAssign, Deref, DerefMut},
    path::Path,
    str::{self, Bytes, CharIndices, Chars, EncodeUtf16, FromStr, Lines, Utf8Error},
    string::FromUtf8Error,
    vec,
};
use utf8_io::{Utf8Reader, Utf8Writer, WriteStr};
use basic_text::{TextStr, TextString, TextReader, TextWriter};

/// A Restricted Text encoded, growable string.
#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct RestrictedString(TextString);

/// `RestrictedStr` is to `RestrictedString` as `TextStr` is to `TextString`.
#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct RestrictedStr(TextStr);

/// `RestrictedError` is to `RestrictedString` as `TextError` is to `TextString`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct RestrictedError {
    // TODO: `valid_up_to`?
// But `write_str` (and `write_all`) don't currently report how many bytes
// they wrote before an error.
}

/// `FromRestrictedError` is to `RestrictedString` as `FromTextError` is to `TextString`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromRestrictedError {
    bytes: Vec<u8>,
    error: RestrictedError,
}

impl RestrictedString {
    /// Creates a new empty `RestrictedString`.
    #[inline]
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Creates a new empty `RestrictedString` with a particular capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    /// Converts a vector of bytes to a `RestrictedString`.
    #[inline]
    pub fn from_text_vec(vec: Vec<u8>) -> Result<Self, FromRestrictedError> {
        Self::from_text(String::from_utf8(vec)?)
    }

    /// Converts a `String` to a `RestrictedString`.
    #[inline]
    pub fn from_text(s: String) -> Result<Self, FromRestrictedError> {
        let bytes: Vec<u8> = Vec::new();
        let mut writer = RestrictedWriter::new(TextWriter::new(Utf8Writer::new(LayeredWriter::new(bytes))));
        writer.write_str(&s).map_err(|_err| FromRestrictedError {
            bytes: s.into_bytes(),
            error: RestrictedError {},
        })?;
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

    // TODO: from_raw_parts, from_utf16*

    /// Converts a vector of bytes to a `RestrictedString` without checking that the
    /// string contains valid Restricted Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Restricted Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&RestrictedStr`s
    /// are valid Restricted Text.
    #[inline]
    pub unsafe fn from_text_vec_unchecked(vec: Vec<u8>) -> Self {
        Self::from_text_unchecked(String::from_utf8_unchecked(vec))
    }

    /// Converts a `String` to a `RestrictedString` without checking that the string
    /// contains valid Restricted Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Restricted Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&RestrictedStr`s
    /// are valid Restricted Text.
    #[inline]
    pub const unsafe fn from_text_unchecked(s: String) -> Self {
        Self(s)
    }

    /// Converts a `RestrictedString` into a `String`.
    #[inline]
    pub fn into_utf8(self) -> String {
        self.0
    }

    /// Converts a String into a byte vector.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    /// Extracts a UTF-8 string slice containing the entire `RestrictedString`.
    #[inline]
    pub fn as_utf8(&self) -> &str {
        &self.0
    }

    /// Extracts a Restricted Text string slice containing the entire `RestrictedString`.
    #[inline]
    pub fn as_str(&self) -> &RestrictedStr {
        self
    }

    /// Converts a `RestrictedString` into a mutable UTF-8 string slice.
    #[inline]
    pub fn as_mut_utf8(&mut self) -> &mut str {
        &mut self.0
    }

    /// Converts a `RestrictedString` into a mutable Restricted Text string slice.
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut RestrictedStr {
        self
    }

    /// Appends a given string slice onto the end of this `RestrictedString`.
    ///
    /// But wait, NFKC isn't closed under concatenation! This is true, but
    /// Restricted Text has additional restrictions, including that strings start
    /// with non-combining codepoints, so it is closed under concatenation.
    #[inline]
    pub fn push_str(&mut self, s: &RestrictedStr) {
        self.0.push_str(&s.0);
    }

    /// Returns this `RestrictedString`'s capacity, in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Ensures that this `RestrictedString`'s capacity is at least `additional`
    /// bytes larger than its length.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Ensures that this `RestrictedString`'s capacity is `additional` bytes larger
    /// than its length.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    /// Tries to reserve capacity for at least `additional` more elements to
    /// be inserted in the given `RestrictedString`.
    #[cfg(try_reserve)]
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// Tries to reserves the minimum capacity for exactly `additional` more
    /// elements to be inserted in the given `RestrictedString`.
    #[cfg(try_reserve)]
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    /// Shrinks the capacity of this `RestrictedString` to match its length.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Shrinks the capacity of this `String` with a lower bound.
    #[cfg(shrink_to)]
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity)
    }

    // TODO: push? But think about how to maintain NFC and other guarantees

    /// Returns a byte slice of this `RestrictedString`'s contents.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    // TODO: truncate, pop, remove, retain, insert? ditto
    // TODO: insert_str? We could do CGJ's where needed there?

    /// Returns a mutable reference to the contents of this `RestrictedString`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Restricted Text. If this constraint is violated, it may
    /// cause memory unsafety issues with future users of the String, as the
    /// rest of this crate assumes that `RestrictedString`s are valid Restricted Text.
    #[inline]
    pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        self.0.as_mut_vec()
    }

    /// Returns the length of this `RestrictedString`, in bytes, not `char`s or
    /// graphemes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if this `RestrictedString` has a length of zero, and `false`
    /// otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // TODO: split_off?

    /// Truncates this `String`, removing all contents.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    // TODO: drain, replace_range?

    /// Converts this `RestrictedString` into a `Box<str>`.
    #[inline]
    pub fn into_boxed_utf8(self) -> Box<str> {
        self.0.into_boxed_str()
    }

    /// Converts this `RestrictedString` into a `Box<RestrictedStr>`.
    #[inline]
    pub fn into_boxed_str(self) -> Box<RestrictedStr> {
        let slice = self.into_boxed_utf8();
        unsafe { RestrictedStr::from_boxed_text_unchecked(slice) }
    }
}

impl Default for RestrictedString {
    #[inline]
    fn default() -> Self {
        Self(String::default())
    }
}

impl Deref for RestrictedString {
    type Target = RestrictedStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { RestrictedStr::from_text_unchecked(&*self.0) }
    }
}

impl DerefMut for RestrictedString {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { RestrictedStr::from_text_unchecked_mut(&mut *self.0) }
    }
}

impl Borrow<RestrictedStr> for RestrictedString {
    #[inline]
    fn borrow(&self) -> &RestrictedStr {
        self
    }
}

impl BorrowMut<RestrictedStr> for RestrictedString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut RestrictedStr {
        self
    }
}

impl AsMut<RestrictedStr> for RestrictedString {
    #[inline]
    fn as_mut(&mut self) -> &mut RestrictedStr {
        self
    }
}

impl Add<&RestrictedStr> for RestrictedString {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &RestrictedStr) -> Self::Output {
        self.push_str(other);
        self
    }
}

impl AddAssign<&RestrictedStr> for RestrictedString {
    #[inline]
    fn add_assign(&mut self, other: &RestrictedStr) {
        self.push_str(other)
    }
}

impl<'a> PartialEq<&'a RestrictedStr> for RestrictedString {
    #[inline]
    fn eq(&self, other: &&'a RestrictedStr) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, RestrictedStr>> for RestrictedString {
    #[inline]
    fn eq(&self, other: &Cow<'a, RestrictedStr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<RestrictedString> for Cow<'a, RestrictedStr> {
    #[inline]
    fn eq(&self, other: &RestrictedString) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<str> for RestrictedString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<String> for RestrictedString {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<RestrictedString> for String {
    #[inline]
    fn eq(&self, other: &RestrictedString) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> PartialEq<RestrictedString> for &'a str {
    #[inline]
    fn eq(&self, other: &RestrictedString) -> bool {
        self.eq(&other.0)
    }
}

// TODO: impl Extend for RestrictedString?

impl RestrictedStr {
    /// Converts a slice of bytes to a text string slice.
    #[inline]
    pub fn from_text_bytes(b: &[u8]) -> Result<&Self, RestrictedError> {
        Self::from_text(str::from_utf8(b)?)
    }

    /// Converts a string slice to a text string slice.
    #[inline]
    pub fn from_text(s: &str) -> Result<&Self, RestrictedError> {
        // TODO: Do this without constructing temporaries.
        if RestrictedString::from_text(s.to_string())
            .map_err(|e| e.text_error())?
            .as_utf8()
            != s
        {
            return Err(RestrictedError {});
        }
        Ok(unsafe { Self::from_text_unchecked(s) })
    }

    /// Converts a mutable slice of bytes to a mutable text string slice.
    #[inline]
    pub fn from_text_bytes_mut(b: &mut [u8]) -> Result<&mut Self, RestrictedError> {
        Self::from_text_mut(str::from_utf8_mut(b)?)
    }

    /// Converts a mutable string slice to a mutable text string slice.
    #[inline]
    pub fn from_text_mut(s: &mut str) -> Result<&mut Self, RestrictedError> {
        // TODO: Do this without constructing temporaries.
        if RestrictedString::from_text((*s).to_string())
            .map_err(|e| e.text_error())?
            .as_utf8()
            != s
        {
            return Err(RestrictedError {});
        }
        Ok(unsafe { Self::from_text_unchecked_mut(s) })
    }

    /// Converts a slice of bytes to a text string slice without checking that
    /// the string contains valid Restricted Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Restricted Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&RestrictedStr`s
    /// are valid Restricted Text.
    #[inline]
    pub unsafe fn from_text_bytes_unchecked(b: &[u8]) -> &Self {
        Self::from_text_unchecked(str::from_utf8_unchecked(b))
    }

    /// Converts a string slice to a text string slice without checking that
    /// the string contains valid Restricted Text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Restricted Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&RestrictedStr`s
    /// are valid Restricted Text.
    #[inline]
    pub unsafe fn from_text_unchecked(s: &str) -> &Self {
        &*(s as *const str as *const Self)
    }

    /// Converts a slice of bytes to a text string slice without checking that
    /// the string contains valid Restricted Text; mutable version.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Restricted Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&RestrictedStr`s
    /// are valid Restricted Text.
    #[inline]
    pub unsafe fn from_text_bytes_unchecked_mut(b: &mut [u8]) -> &mut Self {
        Self::from_text_unchecked_mut(str::from_utf8_unchecked_mut(b))
    }

    /// Converts a string slice to a text string slice without checking that
    /// the string contains valid Restricted Text; mutable version.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the string
    /// passed to it is valid Restricted Text. If this constraint is violated,
    /// undefined behavior results, as the rest of this crate assumes that
    /// `&RestrictedStr`s are valid Restricted Text.
    #[inline]
    pub unsafe fn from_text_unchecked_mut(s: &mut str) -> &mut Self {
        &mut *(s as *mut str as *mut Self)
    }

    /// Converts a boxed slice of bytes to a boxed text string slice without
    /// checking that the string contains valid basic text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Restricted Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&RestrictedStr`s
    /// are valid Restricted Text.
    #[inline]
    pub unsafe fn from_boxed_text_bytes_unchecked(v: Box<[u8]>) -> Box<Self> {
        Box::from_raw(Box::into_raw(v) as *mut Self)
    }

    /// Converts a boxed string slice to a boxed text string slice without
    /// checking that the string contains valid basic text.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid Restricted Text. If this constraint is violated, undefined
    /// behavior results, as the rest of this crate assumes that `&RestrictedStr`s
    /// are valid Restricted Text.
    #[inline]
    pub unsafe fn from_boxed_text_unchecked(v: Box<str>) -> Box<Self> {
        Box::from_raw(Box::into_raw(v) as *mut Self)
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
    /// Restricted Text before the borrow ends and the underlying `RestrictedStr` is used.
    ///
    /// Use of a `RestrictedStr` whose contents are not valid Restricted Text is undefined
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

    /// Extracts a UTF-8 string slice containing the entire `RestrictedStr`.
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

    /// Returns an iterator of `u16` over the string encoded as Restricted Text.
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
        // TODO: Is there a way we could use `TryFrom<&RestrictedStr>` to allow
        // parsers to work from a `RestrictedStr` instead of just a `str`?
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

    /// Converts a `Box<RestrictedStr`> into a `Box<[u8]>` without copying or
    /// allocating.
    #[inline]
    pub fn into_boxed_bytes(self: Box<Self>) -> Box<[u8]> {
        self.into_boxed_str().into_boxed_bytes()
    }

    /// Converts a `Box<RestrictedStr>` into a `Box<str>` without copying or
    /// allocating.
    #[inline]
    pub fn into_boxed_str(self: Box<Self>) -> Box<str> {
        self.into()
    }

    /// Converts a `Box<RestrictedStr>` into a `String` without copying or allocating.
    #[inline]
    pub fn into_utf8(self: Box<Self>) -> String {
        let slice = Box::<[u8]>::from(self);
        unsafe { String::from_utf8_unchecked(slice.into_vec()) }
    }

    /// Converts a `Box<RestrictedStr>` into a `RestrictedString` without copying or
    /// allocating.
    #[inline]
    pub fn into_string(self: Box<Self>) -> RestrictedString {
        unsafe { RestrictedString::from_text_unchecked(Self::into_utf8(self)) }
    }

    // TODO: make_ascii_uppercase, make_ascii_lowercase, escape_debug,
    // escape_default, escape_unicode, replace*, to_lowercase, to_uppercase,
    // into_string, repeat, to_ascii_uppercase, to_ascii_lowercase; determine
    // whether these can be done without breaking NFKC.
}

impl AsRef<[u8]> for RestrictedStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<OsStr> for RestrictedStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for RestrictedStr {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<str> for RestrictedStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<RestrictedStr> for RestrictedStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for &RestrictedStr {
    #[inline]
    fn default() -> Self {
        unsafe { RestrictedStr::from_text_unchecked("") }
    }
}

impl Default for &mut RestrictedStr {
    #[inline]
    fn default() -> Self {
        unsafe { RestrictedStr::from_text_bytes_unchecked_mut(&mut []) }
    }
}

impl Display for RestrictedStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

// TODO: impl Index* for RestrictedStr?

impl Ord for RestrictedStr {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<'a> PartialEq<Cow<'a, RestrictedStr>> for RestrictedStr {
    #[inline]
    fn eq(&self, other: &Cow<'a, Self>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, 'b> PartialEq<Cow<'a, RestrictedStr>> for &'b RestrictedStr {
    #[inline]
    fn eq(&self, other: &Cow<'a, RestrictedStr>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a> PartialEq<str> for RestrictedStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<String> for RestrictedStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<RestrictedStr> for String {
    #[inline]
    fn eq(&self, other: &RestrictedStr) -> bool {
        self.eq(&other.0)
    }
}

// TODO: all the PartialEq impls for RestrictedStr

impl PartialOrd<RestrictedStr> for RestrictedStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// TODO: Pattern for RestrictedStr

impl ToOwned for RestrictedStr {
    type Owned = RestrictedString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        RestrictedString(self.0.to_owned())
    }
}

impl ToSocketAddrs for RestrictedStr {
    type Iter = vec::IntoIter<SocketAddr>;

    #[inline]
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        self.0.to_socket_addrs()
    }
}

impl RestrictedError {
    // TODO: valid_up_to etc.?
}

impl From<Utf8Error> for RestrictedError {
    fn from(_err: Utf8Error) -> Self {
        Self {}
    }
}

impl Display for RestrictedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "TODO: Display for RestrictedError: {:?}", self)
    }
}

impl Error for RestrictedError {}

impl FromRestrictedError {
    /// Returns a slice of `u8`s bytes that were attempted to convert to a
    /// `RestrictedString`.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the bytes that were attempted to convert to a `RestrictedString`.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Fetch a `RestrictedError` to get more details about the conversion failure.
    #[inline]
    pub fn text_error(&self) -> RestrictedError {
        self.error
    }
}

impl From<FromUtf8Error> for FromRestrictedError {
    #[inline]
    fn from(err: FromUtf8Error) -> Self {
        let error = err.utf8_error().into();
        let bytes = err.into_bytes();
        Self { bytes, error }
    }
}

impl Display for FromRestrictedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "TODO: Display for FromRestrictedError: {:?}", self)
    }
}

impl Error for FromRestrictedError {}

impl From<Box<RestrictedStr>> for Box<[u8]> {
    #[inline]
    fn from(s: Box<RestrictedStr>) -> Self {
        unsafe { Self::from_raw(Box::into_raw(s) as *mut [u8]) }
    }
}

impl From<Box<RestrictedStr>> for Box<str> {
    #[inline]
    fn from(s: Box<RestrictedStr>) -> Self {
        unsafe { Self::from_raw(Box::into_raw(s) as *mut str) }
    }
}

impl From<Box<RestrictedStr>> for RestrictedString {
    #[inline]
    fn from(s: Box<RestrictedStr>) -> Self {
        s.into_string()
    }
}

impl From<Cow<'_, RestrictedStr>> for Box<RestrictedStr> {
    #[inline]
    fn from(cow: Cow<'_, RestrictedStr>) -> Self {
        match cow {
            Cow::Borrowed(s) => Self::from(s),
            Cow::Owned(s) => Self::from(s),
        }
    }
}

impl From<RestrictedString> for Box<RestrictedStr> {
    #[inline]
    fn from(s: RestrictedString) -> Self {
        s.into_boxed_str()
    }
}

impl Clone for Box<RestrictedStr> {
    #[inline]
    fn clone(&self) -> Self {
        let buf: Box<[u8]> = self.as_bytes().into();
        unsafe { RestrictedStr::from_boxed_text_bytes_unchecked(buf) }
    }
}

impl Default for Box<RestrictedStr> {
    #[inline]
    fn default() -> Self {
        unsafe { RestrictedStr::from_boxed_text_bytes_unchecked(Box::default()) }
    }
}

impl From<&RestrictedStr> for Box<RestrictedStr> {
    #[inline]
    fn from(s: &RestrictedStr) -> Self {
        unsafe { RestrictedStr::from_boxed_text_bytes_unchecked(Box::from(s.as_bytes())) }
    }
}

#[test]
fn validate_string() {
    assert!(RestrictedStr::from_text_bytes(b"").is_ok());
}
