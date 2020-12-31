# Restricted Text

The *Restricted Text* format is built on top of the [Basic Text] format. It
incorporates several restrictions which reduce the expressiveness of the
format in order to reduce visual ambiguity.

TODO: This isn't implemented yet.

This format does not define lossy conversions, as that may cause meaning to
be silently lost. Instead, errors should be reported when content not meeting
these restrictions is encountered, on input or output, in a context where
restricted text is required. See [Basic Text] for an unrestricted alternative.

## Definitions

A string is in Restricted Text form iff:
 - it is in [Basic Text] form, and
 - it is in [NFKC], and
 - it is [Moderately Restricted] text, and
 - it does not contain any of the sequences listed in the [Tables].

A stream is in Restricted Text form iff:
 - it is a stream in [Basic Text] form, and
 - it consists entirely of a string in Restricted Text form.

A buffered stream is in Restricted Text form iff:
 - the buffered stream is in [Basic Text] form, and
 - a flush of the buffer fails if the data up to that point is not a
   string in Restricted Text form.

Note that even though this excludes U+034F (COMBINING GRAPHEME JOINER), the
[Stream Safe Text Format] is still required; content must simply avoid using
excessively long sequences of non-starters.

[Tables]: #tables

## Tables

### Main Table

| Sequence            | Error                                                    |
| ------------------- | -------------------------------------------------------- |
| [U+FE00–U+FE0F]     | "Variation selectors are not required to be implemented" |
| [U+E0100–U+E01EF]   | "Variation selectors are not required to be implemented" |
| [Default Ignorable Code Points] | "Default Ignorable Code Points are not visually distinct" |
| [Old Hangul Jamo]   | "Conjoining Hangul Jamo are restricted in RFC5892" |
| [Tag Characters]    | "Tag Characters do not belong to textual content" |
| [Private-Use Characters] | "Private-use characters depend on private agreements" |

## Conversion

### String Conversion, Strict

To convert a [Basic Text] string into a Restricted Text string in a manner that
never loses information but may fail:
 - If performing `toNFKC` with the
   [Normalization Process for Stabilized Strings] would alter the contents,
   error with "Restricted Text must be in NFKC form".
 - Perform the Error actions from the [Main Table].

### Stream Conversion, Strict

To convert a [Basic Text] stream into a Restricted Text stream in a manner than
never loses information but may fail:
 - Perform [String Conversion, Strict].

[Main Table]: #main-table
[String Conversion, Lossy]: #string-conversion-lossy
[String Conversion, Strict]: #string-conversion-strict

## TODO

TODO: "Moderately Restricted" [isn't stable over time](https://www.unicode.org/reports/tr39/#Migration).

TODO: [Mixed-Number Detection]

TODO: Unicode Security Mechanisms also specifies some [Optional Detection] rules.

TODO: U+2126 (OHM SIGN) normalizes to U+03A9 (GREEK CAPITAL LETTER OMEGA);
does "Moderately Restricted" permit this Greek letter to be mixed with
otherwise Latin script?

TODO: Several codepoints such as U+2800, U+3164, U+1160, U+FFA0, U+115F,
U+16FE4, and possibly others, often display as whitespace despite not being
categorized as whitespace. Can we constraint them with a mixed-script
constraint, or some other mechanism?

[NFKC]: https://unicode.org/reports/tr15/#Norm_Forms
[Moderately Restricted]: https://www.unicode.org/reports/tr39/#Restriction_Level_Detection
[Stream Safe Text Format]: https://unicode.org/reports/tr15/#Stream_Safe_Text_Format
[Old Hangul Jamo]: https://tools.ietf.org/html/rfc5892#section-2.9
[Default Ignorable Code Points]: https://www.unicode.org/versions/Unicode13.0.0/ch05.pdf#G7730
[Section 23.8 of the Unicode Standard]: https://www.unicode.org/versions/Unicode13.0.0/ch23.pdf#G19635
[Basic Text]: BasicText.md
[Mixed-Number Detection]: https://www.unicode.org/reports/tr39/#Mixed_Number_Detection
[Optional Detection]: https://www.unicode.org/reports/tr39/#Optional_Detection
[Normalization Process for Stabilized Strings]: https://unicode.org/reports/tr15/#Normalization_Process_for_Stabilized_Strings
[Tag Characters]: https://www.unicode.org/versions/Unicode13.0.0/ch23.pdf#G30110
[Private-Use Characters]: http://www.unicode.org/faq/private_use.html#private_use
