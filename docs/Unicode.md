# Unicode

Here, the *Unicode* format is just a sequence of [Unicode Scalar Values].

Unicode permits control codes and other non-textual content; see [Basic Text]
for a subset focused on textual content.

Currently Basic Text is based on Unicode 15.1.

## Definitions

A string is in Unicode form iff:
 - it encodes a sequence of [Unicode Scalar Values].

A stream is in Unicode form iff:
 - it consists entirely of a string in Unicode form

## Conversion

### From byte sequence to Unicode string

To convert a byte sequence into a Unicode string in a manner that always
succeeds but potentially loses information about invalid encodings:
 - Perform [U+FFFD Substitution of Maximal Subparts].

[From byte sequence to Unicode string]: #from-byte-sequence-to-unicode-string
[Basic Text]: BasicText.md
[Unicode Scalar Values]: https://unicode.org/glossary/#unicode_scalar_value
[U+FFFD Substitution of Maximal Subparts]: https://www.unicode.org/versions/Unicode15.1.0/ch03.pdf#G66453
