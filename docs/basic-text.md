# Basic Text

The *basic text* format is built on top of the [UTF-8] format. It is intended
for general-purpose use most anywhere the informal notion of "plain text" is
intended.

Basic text does permit homoglyphs and other visual ambiguities; see
[Restricted Text] for an alternative which provides some mitigations.

## Definition

A string is in Basic Text form if:
 - it is in [Stream-Safe] [NFC] form, and
 - it does not contain any of the sequences listed in the [tables] where
   the Input rule is not *passthrough*.

A stream is is in Basic Text form if:
 - its contents are a string in Basic Text form, and
 - it is empty, or it ends with U+A.

 [tables]: #tables

## Tables

### Pre-NFC Table

| Sequence            | aka   | Input         | Output      |
| ------------------- | ----- | ------------- | ----------- |
| U+2126              | `Ω`   | passthrough   | *error*     |
| U+212A              | `K`   | passthrough   | *error*     |
| U+212B              | `Å`   | passthrough   | *error*     |
| U+2329              | `〈`  | passthrough   | *error*     |
| U+232A              | `〉`  | passthrough   | *error*     |
| [CJK Compatibility Ideographs] | | *standardized variation sequence* | *standardized variation sequence* |

### Main Table

| Sequence            | aka   | Input         | Output      |
| ------------------- | ----- | ------------- | ----------- |
| U+D U+A             | CRLF  | U+A           | *error*     |
| U+9                 | HT    | passthrough   | passthrough |
| U+A                 | NL    | passthrough   | *newline*   |
| U+D                 | CR    | U+A           | *error*     |
| U+C                 | FF    | U+20          | *error*     |
| U+1B U+5B \[U+20–U+3F\]\* U+6D | SGR | *color escape sequence* | *color escape sequence* |
| U+1B+ U+5B \[U+20–U+3F\]\* \[U+40–U+7E\]? | CSI | | *escape sequence* |
| U+1B+ U+5D \[\^U+7,U+18,U+1B\]\* \[U+7,U+18\]? | OSC | | *escape sequence* |
| U+1B+ \[U+40–U+7E\]? | ESC |          | *escape sequence* |
| U+1B+ U+5B U+5B \[U+–U+7F\]? | Linux ESC  | | *escape sequence* |
| \[U+0–U+1F\]        | C0    | U+FFFD        | *error*     |
| U+7F                | DEL   | U+FFFD        | *error*     |
| U+85                | NEL   | U+20          | *error*     |
| \[U+80–U+9F\]       | C1    | U+FFFD        | *error*     |
| U+149               | `ʼn`  | U+2BC U+6E    | *error*     |
| U+673               | `ا ٟ`  | U+627 U+65F   | *error*     |
| U+F77               | `◌ྲ◌ཱྀ`  | U+FB2 U+F81   | *error*     |
| U+F79               | `◌ླ◌ཱྀ`  | U+FB3 U+F81   | *error*     |
| U+17A3              | `អ`   | U+17A2        | *error*     |
| U+17A4              | `អា`  | U+17A2 U+17B6 | *error*     |
| U+17B4              |       | U+FFFD        | *error*     |
| U+17B5              |       | U+FFFD        | *error*     |
| U+17D8              |       | U+FFFD        | *error*     |
| U+FEFF              | BOM   | U+2060        | *error*     |
| U+FFFC              | ORC   | U+FFFD        | *error*     |
| \[U+FFF9–U+FFFB\]   | IA    | U+FFFD        | *error*     |
| [Noncharacters]                | | U+FFFD   | *error*     |
| [Deprecated Format Characters] | | U+FFFD   | *error*     |
| [Private-Use Characters]       | | U+FFFD   | *error*     |
| [Tag Characters]               | | U+FFFD   | *error*     |

## Conversion

To convert a [UTF-8] stream into a Basic Text stream:
 - On input, if the stream starts with U+FEFF, remove it.
 - Perform the actions from the [pre-NFC table].
 - Perform the [Stream-Safe Text Process (UAX15-D4)].
 - Apply `toNFC` with the [Normalization Process for Stabilized Strings].
 - Apply the actions from the [main table].
 - On output, if the stream is non-empty and doesn't end with U+A, error.
 - On input, if the stream is non-empty and doesn't end with U+A, append a U+A.

[pre-NFC table]: #pre-nfc-table
[main table]: #main-table

### Special Actions

| Name                    | Default Action  |
| ----------------------- | --------------- |
| *color escape sequence* |                 |
| *escape sequence*       |                 |
| *newline*               | U+A             |
| *standardized variation sequence* | Replace with the corresponding [Standardized Variation] |
| *error*                 | Report an error and abandon the stream |

## Optional Features

A number of features may be optionally enabled:

| Name                   | Direction       | Default | Description |
| ---------------------- | --------------- | ------- | ----------- |
| BOM compatiblity       | output          | off     | Prepend U+FEFF to the stream |
| CRLF compatiblity      | output          | off     | Translate *newline* as U+D U+A |
| Color escape sequences | input or output | off     | Translate *color escape sequecence* as *passthrough* |

## TODO

TODO: Basic Text strings should never start with a normalization-form non-starter.
`canonical_combining_class` doesn't know about the astral compositions like
U+11099 U+110BA => U+1109A. Restrict non-starters of that form too? Or a joiner
like WJ, CGJ, ZWJ, or ZWNJ? Or use unicode-segmentation to detect grapheme boundaries?

TODO: stream pushes should start new Basic Text strings
TODO: Validate/normalize [BiDi Controls]
TODO: Validate [variation sequences]

[NFC]: https://unicode.org/reports/tr15/#Norm_Forms
[Stream-Safe]: https://unicode.org/reports/tr15/#Stream_Safe_Text_Format
[Stream-Safe Text Process (UAX15-D4)]: https://unicode.org/reports/tr15/#UAX15-D4
[Standardized Variation]: https://www.unicode.org/Public/UNIDATA/StandardizedVariants.txt
[Normalization Process for Stabilized Strings]: https://unicode.org/reports/tr15/#Normalization_Process_for_Stabilized_Strings
[Noncharacters]: http://www.unicode.org/faq/private_use.html#noncharacters
[Deprecated Format Characters]: https://www.unicode.org/versions/Unicode13.0.0/ch23.pdf#G19593
[Private-Use Characters]: http://www.unicode.org/faq/private_use.html#private_use
[Tag Characters]: https://www.unicode.org/versions/Unicode13.0.0/ch23.pdf#G30110
[Restricted Text]: restricted-text.md
[UTF-8]: utf-8.md
[CJK Compatibility Ideographs]: http://www.unicode.org/versions/latest/ch23.pdf#G19053
[BiDi Controls]: https://unicode.org/reports/tr9/
[variation sequences]: http://unicode.org/faq/vs.html#3
