# Restricted Text

The *Restricted Text* format is built on top of the [Plain Text] format, which
incorporates several restrictions which reduce the expressiveness of the
format in order to reduce visual ambiguity.

Content should not be implicitly converted into this format, as that may cause
meaning to be silently lost. Instead, errors should be reported when content
not meeting these restrictions is encountered, on input or output. See
[Plain Text] for an unrestricted alternative.

Restricted Text is required be in [NFKC], to satisfy the requirements
for [Moderately Restricted] text, and to exclude the following codepoints:
 - Annotation Characters, as defined in [Section 23.8 of the Unicode Standard],
   U+FFF9–UFFFB.
 - Default Ignorable Code Points, as defined in
   [Section 5.3 of the Unicode Standard].
 - Old Hangul Jamo, as defined in [Section 2.9 of RFC 5892].

Note that even though this excludes U+034F (COMBINING GRAPHEME JOINER), the
[Stream Safe Text Format] is still required; content must simply avoid using
excessively long sequences of non-starters.

TODO: If CGJ is disallowed, can we still have a way to safely concatenate?

TODO: "Moderately Restricted" [isn't stable over time](https://www.unicode.org/reports/tr39/#Migration).

TODO: [Mixed-Number Detection]

TODO: Unicode Security Mechanisms also specifies some [Optional Detection] rules.

TODO: U+2126 (OHM SIGN) normalizes to U+03A9 (GREEK CAPITAL LETTER OMEGA);
does "Moderately Restricted" permit this Greek letter to be mixed with
otherwise Latin script?

[NFKC]: https://unicode.org/reports/tr15/#Norm_Forms
[Moderately Restricted]: https://www.unicode.org/reports/tr39/#Restriction_Level_Detection
[Stream Safe Text Format]: https://unicode.org/reports/tr15/#Stream_Safe_Text_Format
[Section 2.9 of RFC 5892]: https://tools.ietf.org/html/rfc5892#section-2.9
[Section 5.3 of the Unicode Standard]: https://www.unicode.org/versions/Unicode13.0.0/ch05.pdf#G7730
[Section 23.8 of the Unicode Standard]: https://www.unicode.org/versions/Unicode13.0.0/ch23.pdf#G19635
[Plain Text]: plain-text.md
[Mixed-Number Detection]: https://www.unicode.org/reports/tr39/#Mixed_Number_Detection
[Optional Detection]: https://www.unicode.org/reports/tr39/#Optional_Detection
