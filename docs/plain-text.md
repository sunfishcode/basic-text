# Plain Text

The *Plain Text* format is built on top of the [UTF-8] format. It is
intended for general-purpose use most anywhere the informal notion of
"plain text" is intended. It permits homoglyphs and other visual ambiguities;
see [Restricted Text] for an alternative which provides some mitigations.

On input and output, Plain Text is implicitly converted into [NFC] by the
following steps, in order:
 - Convert all CJK Compatibility Ideograph codepoints that have corresponding
   [Standardized Variations] into their corresponding standardized variation
   sequences.
 - Apply the [Stream-Safe Text Process (UAX15-D4)].
 - Apply `toNFC` according to the [Normalization Process for Stabilized Strings].

TODO: The sequencing of Stream Safe and NFC matters; test this!

TODO: Pull some NFKC translations in here? https://github.com/rust-lang/rust/issues/2253#issuecomment-29050949

On input, the following transformations are applied:
 - U+FEFF (BOM) scalar values are stripped
 - A U+000A is appended at the end of the stream if the stream wasn't
   empty and it doesn't already have one
 - U+000D followed by U+000A is replaced by U+000A (newline)
 - U+000C (FF) is replaced by U+0020 (space)
 - All other disallowed codepoints are replaced by U+FFFD (REPLACEMENT CHARACTER).
 - Disallowed codepoint sequences are replaced by U+FFFD (REPLACEMENT CHARACTER).

On output, before conversion to NFC, the following requirements are enforced:
 - If any bytes have been written to the stream, U+000A (newline) is required
   at the end of the stream.
 - The following codepoints must not be present:
    - The disallowed codepoints
    - U+2000 (EN QUAD)
    - U+2001 (EM QUAD)
    - U+2126 (OHM SIGN)
    - U+212A (KELVIN SIGN)
    - U+212B (ANGSTROM SIGN)
    - U+2329 (LEFT-POINTING ANGLE BRACKET)
    - U+232A (RIGHT-POINTING ANGLE BRACKET)
    - TODO: these aren't implemented yet
 - The disallowed codepoint sequences must not be present.
    - TODO: this isn't implemented yet

On output, the following transformations are applied:
 - As an option, off by default, "\n" may be translated to "\r\n".

TODO: Streams never start or resume after a push with a normalization-form non-starter.
`canonical_combining_class` doesn't know about the astral compositions like
U+11099 U+110BA => U+1109A. Restrict non-starters of that form too? Or use
unicode-segmentation to detect grapheme boundaries?

The *disallowed codepoints* are:
 - All C0, U+007F, and C1 control codes other than U+000A (newline) and
   U+0009 (horizontal tab)
 - U+FEFF (BOM)
 - The [Forbidden Characters] U+F951, U+2F868, U+2F874, U+2F91F, U+2F95F, and U+2F9BF
 - [Noncharacters]
    - TODO: this isn't implemented yet
 - [Deprecated Format Characters]
    - TODO: this isn't implemented yet
 - [Private-Use Characters]
    - TODO: this isn't implemented yet
 - [Tag Characters]
    - TODO: this isn't implemented yet

The *disallowed codepoint sequences* are:
 - [Corrigendum 5 Sequences]

TODO: NFC isn't closed under concatenation; can we restrict streams to starting with starters, or implicitly insert CGJs?

TODO: Should we say anything about bidi control codepoints? https://unicode.org/reports/tr9/

TODO: Should we say anything about inter-linear annotation codepoints?

TODO: Should we say anything about U+FFFC (object replacement character)?

TODO: Should we say anything about unrecognized and/or IVD variation selectors?

[NFC]: https://unicode.org/reports/tr15/#Norm_Forms
[Stream-Safe Text Process (UAX15-D4)]: https://unicode.org/reports/tr15/#UAX15-D4
[Standardized Variations]: http://unicode.org/faq/vs.html
[Normalization Process for Stabilized Strings]: https://unicode.org/reports/tr15/#Normalization_Process_for_Stabilized_Strings
[Forbidden Characters]: https://unicode.org/reports/tr15/#Forbidding_Characters
[Corrigendum 5 Sequences]: https://unicode.org/reports/tr15/#Corrigendum_5_Sequences
[Noncharacters]: http://www.unicode.org/faq/private_use.html#noncharacters
[Deprecated Format Characters]: https://www.unicode.org/versions/Unicode13.0.0/ch23.pdf#G19593
[Private-Use Characters]: http://www.unicode.org/faq/private_use.html#private_use
[Tag Characters]: https://www.unicode.org/versions/Unicode13.0.0/ch23.pdf#G30110
[Restricted Text]: restricted-text.md
[UTF-8]: utf-8.md
