# Unicode

Here, the *Unicode* format is just a sequence of [Unicode Scalar Values].

Unicode permits control codes and other non-textual content; see [Basic Text]
for an alternative which focused on textual content.

## Definitions

A string is in Unicode form iff:
 - it encodes a sequence of [Unicode Scalar Values].

A substring is in Unicode form iff:
 - it is itself a string in Unicode form

A stream is in Unicode form iff:
 - it consists entirely of a string in Unicode form

A buffered stream is in Unicode form iff:
 - the stream is in Unicode form, and
 - a successful buffer flush produces output which is a substring in Unicode
   form.

## Conversion

### String Conversion, Lossy

To convert a byte sequence into a Unicode String in a manner that always
succeeds but potentially loses information:
 - Perform [U+FFFD Substitution of Maximal Subparts].

### String Conversion, Strict

To convert a byte sequence into a Unicode String in a manner that always
succeeds but potentially loses information:
 - Report an error if an invalid encoding is encountered.

[Basic Text]: BasicText.md
[Unicode Scalar Values]: https://unicode.org/glossary/#unicode_scalar_value
[U+FFFD Substitution of Maximal Subparts]: https://www.unicode.org/versions/Unicode13.0.0/ch03.pdf#G66453
