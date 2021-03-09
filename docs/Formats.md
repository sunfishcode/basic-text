# Formats

This repository describes and implements three formats.

 - [Unicode](Unicode.md)
   - Any sequence of [Unicode Scalar Values].
   - Invalid encodings are translated to the Replacement Character (U+FFFD) on
     input, and rejected on output.

 - [Basic Text](BasicText.md)
   - Supports the semantics of any practical Unicode text content.
   - This is intended to realize the intuitive phrases "text" or "plain text"
     which are used in various ways in many contexts. It excludes control
     characters and other content impractical for text.
   - Invalid scalar values and sequences are translated to replacement
     sequences on input, and rejected on output.

 - [Restricted Text](RestrictedText.md)
   - Like Basic Text, but aims to reduce visual ambiguity, trading off some
     support for historical scripts, multiple-script text, formatting, and
     symbols.
   - Invalid scalar values and sequences are rejected, even on input, since it
     isn't always possible to preserve intent automatically.

The [background information] contains rationale and source information.

[background information]: Background.md
[Unicode Scalar Values]: https://unicode.org/glossary/#unicode_scalar_value
