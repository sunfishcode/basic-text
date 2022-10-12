# Restricted Text

The *Restricted Text* format is a subset of the [Basic Text] format. It
incorporates several restrictions which reduce the expressiveness of the format
in order to reduce visual ambiguity.

*This format is entirely hypothetical at this time*. It's formed from a
loose collection of ideas from a variety of sources, and is not yet ready for
any practical purpose.

This format does not define conversion from Basic Text or other less
restrictive formats, as that may cause meaning to be silently lost. Instead,
errors should be reported when content not meeting these restrictions is
encountered in any context where restricted text is expected. See
[Basic Text] for an unrestricted alternative.

## Definitions

A string is in Restricted Text form iff:
 - it is in [Basic Text] form, and
 - it is in [NFKC] form, and
 - it is [Moderately Restricted] text, and
 - it does not contain any of the sequences listed in the [Sequence Table].

A stream is in Restricted Text form iff:
 - it is a stream in [Basic Text] form, and
 - it consists entirely of a string in Restricted Text form.

A buffered stream is in Restricted Text form iff:
 - the buffered stream is in [Basic Text] form

Note that even though this excludes U+34F (COMBINING GRAPHEME JOINER), the
[Stream Safe Text Format] is still required; content must simply avoid using
excessively long sequences of non-starters.

## Sequence Table

| Sequence            | Error                                                 |
| ------------------- | ----------------------------------------------------- |
| \[U+FE00–U+FE0F\]   | "Variation selectors are not always visually distinct" |
| \[U+E0100–U+E01EF\] | "Variation selectors are not always visually distinct" |
| [Default Ignorable Code Points] | "Default Ignorable Code Points are not visually distinct" |
| [Old Hangul Jamo]   | "Conjoining Hangul Jamo are restricted in RFC5892"    |
| [Tag Characters]    | "Tag Characters are not permitted"                    |
| [Private-Use Characters] | "Private-use characters depend on private agreements" |

## Conversion

### From Basic Text string to Restricted Text string

To convert a [Basic Text] string into a Restricted Text string in a manner that
never loses information but may fail:
 - If performing `toNFKC` with the
   [Normalization Process for Stabilized Strings] would alter the contents,
   error with "Restricted Text must be in NFKC form".
 - If [Restriction Level Detection] classifies the string as less than
   Moderately Restricted, error with "Restricted Text must be Moderately
   Restricted".
 - Perform the Error actions from the [Sequence Table].

[Sequence Table]: #sequence-table

### From Basic Text stream to Restricted Text stream

To convert a [Basic Text] stream into a Restricted Text stream in a manner than
never loses information but may fail:
 - Perform [From Basic Text string to Restricted Text string].

[From Basic Text string to Restricted Text string]: #from-basic-text-string-to-restricted-text-string

## TODO

TODO: "Moderately Restricted" [isn't stable over time](https://www.unicode.org/reports/tr39/#Migration).

TODO: [Mixed-Number Detection]

TODO: Unicode Security Mechanisms also specifies some [Optional Detection] rules.

TODO: U+2126 (OHM SIGN) normalizes to U+3A9 (GREEK CAPITAL LETTER OMEGA);
does "Moderately Restricted" permit this Greek letter to be mixed with
otherwise Latin script?

TODO: Several Braille scalars have visual similarities with other scalars, such
as U+2800 and U+20, U+2802 and U+B7, and so on.

TODO: Several scalars such as U+1160, U+2062, U+FFA0, U+115F, U+16FE4, and
possibly others, may display as whitespace despite not being categorized as
whitespace. Can we constrain them with a mixed-script constraint, or some
other mechanism?

TODO: [Implicit Directional Marks] have no display.

[NFKC]: https://unicode.org/reports/tr15/#Norm_Forms
[Moderately Restricted]: https://www.unicode.org/reports/tr39/#Restriction_Level_Detection
[Restriction Level Detection]: https://www.unicode.org/reports/tr39/#Restriction_Level_Detection
[Stream Safe Text Format]: https://unicode.org/reports/tr15/#Stream_Safe_Text_Format
[Old Hangul Jamo]: https://tools.ietf.org/html/rfc5892#section-2.9
[Default Ignorable Code Points]: https://www.unicode.org/versions/Unicode15.0.0/ch05.pdf#G7730
[Basic Text]: BasicText.md
[Mixed-Number Detection]: https://www.unicode.org/reports/tr39/#Mixed_Number_Detection
[Optional Detection]: https://www.unicode.org/reports/tr39/#Optional_Detection
[Normalization Process for Stabilized Strings]: https://unicode.org/reports/tr15/#Normalization_Process_for_Stabilized_Strings
[Tag Characters]: https://www.unicode.org/versions/Unicode15.0.0/ch23.pdf#G30110
[Private-Use Characters]: http://www.unicode.org/faq/private_use.html#private_use
[Implicit Directional Marks]: https://unicode.org/reports/tr9/#Implicit_Directional_Marks
