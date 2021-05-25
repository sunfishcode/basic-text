# Unicode

Here, the *Unicode* format is just a sequence of [Unicode Scalar Values].

Unicode permits control codes and other non-textual content; see [Basic Text]
for a subset focused on textual content.

## Definitions

A string is in Unicode form iff:
 - it encodes a sequence of [Unicode Scalar Values].

A stream is in Unicode form iff:
 - it consists entirely of a string in Unicode form

A buffered stream is in Unicode form iff:
 - the stream is in Unicode form, and
 - a flush of the buffer fails if the data up to that point is not a
   string in Unicode form.

## Conversion

### From byte sequence to Unicode string

To convert a byte sequence into a Unicode string in a manner that always
succeeds but potentially loses information about invalid encodings:
 - Perform [U+FFFD Substitution of Maximal Subparts].

### From byte sequence buffered stream to Unicode buffered stream

To convert a byte sequence buffered stream into a Unicode buffered stream in a
manner than always succeeds, discarding information not encodable as Unicode:
 - Perform [From byte sequence to Unicode string].
 - At each buffer flush:
   - Perform [From byte sequence to Unicode string] on the stream up to that
     point.

[From byte sequence to Unicode string]: #from-byte-sequence-to-unicode-string
[Basic Text]: BasicText.md
[Unicode Scalar Values]: https://unicode.org/glossary/#unicode_scalar_value
[U+FFFD Substitution of Maximal Subparts]: https://www.unicode.org/versions/Unicode13.0.0/ch03.pdf#G66453
