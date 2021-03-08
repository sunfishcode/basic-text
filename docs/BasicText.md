# Basic Text

The *Basic Text* format is a subset of the [Unicode] format and meant to
fulfill common notions of "plain text".

Basic Text permits homoglyphs and other visual ambiguities; see
[Restricted Text] for an alternative which provides some mitigations.

For rationale and background information, see [Background].

[Background]: Background.md

## Definitions

A string is in Basic Text form iff:
 - it is a [Unicode] string in [Stream-Safe] [NFC] form, and
 - it doesn't start with a [non-starter], or a scalar value with a
   `Grapheme_Cluster_Break` of `ZWJ`, `SpacingMark` or `Extend`, and
 - it doesn't end with a scalar value with a `Grapheme_Cluster_Break` of `ZWJ`
   or `Prepend`, and
 - it doesn't attempt [Bidirectional formatting character] nesting less than 0
   or greater than the [Unicode Bidirectional Algorithm `max_depth`], and
 - it doesn't contain any of the sequences listed in the [Tables].

A stream is in Basic Text form iff:
 - it consists entirely of a string in Basic Text form, and
 - it is empty or ends with U+A.

A buffered stream is in Basic Text form iff:
 - the stream is in Basic Text form, and
 - a flush of the buffer fails if the data up to that point is not a
   string in Basic Text form.

[Tables]: #tables
[non-starter]: https://unicode.org/reports/tr15/#Description_Norm
[Bidirectional formatting characters]:  http://www.unicode.org/reports/tr9/
[Unicode Bidirectional Algorithm `max_depth`]: http://www.unicode.org/reports/tr9/#BD2

## Tables

### Pre-NFC Table

| Sequence            | aka  | Replacement       | Error                           |
| ------------------- | ---- | ----------------- | ------------------------------- |
| U+9E4               | `।`  | U+FFFD            | "Use U+964 instead of U+9E4"    |
| U+9E5               | `॥`  | U+FFFD            | "Use U+965 instead of U+9E5"    |
| U+A64               | `।`  | U+FFFD            | "Use U+964 instead of U+A64"    |
| U+A65               | `॥`  | U+FFFD            | "Use U+965 instead of U+A65"    |
| U+AE4               | `।`  | U+FFFD            | "Use U+964 instead of U+AE4"    |
| U+AE5               | `॥`  | U+FFFD            | "Use U+965 instead of U+AE5"    |
| U+B64               | `।`  | U+FFFD            | "Use U+964 instead of U+B64"    |
| U+B65               | `॥`  | U+FFFD            | "Use U+965 instead of U+B65"    |
| U+BE4               | `।`  | U+FFFD            | "Use U+964 instead of U+BE4"    |
| U+BE5               | `॥`  | U+FFFD            | "Use U+965 instead of U+BE5"    |
| U+C64               | `।`  | U+FFFD            | "Use U+964 instead of U+C64"    |
| U+C65               | `॥`  | U+FFFD            | "Use U+965 instead of U+C65"    |
| U+CE4               | `।`  | U+FFFD            | "Use U+964 instead of U+CE4"    |
| U+CE5               | `॥`  | U+FFFD            | "Use U+965 instead of U+CE5"    |
| U+D64               | `।`  | U+FFFD            | "Use U+964 instead of U+D64"    |
| U+D65               | `॥`  | U+FFFD            | "Use U+965 instead of U+D65"    |
| U+2072              | `²`  | U+FFFD            | "Use U+B2 instead of U+2072"    |
| U+2073              | `³`  | U+FFFD            | "Use U+B3 instead of U+2073"    |
| U+2126              | `Ω`  | U+3A9             | "Use U+3A9 instead of U+2126"   |
| U+212A              | `K`  | U+4B              | "Use U+4B instead of U+212A"    |
| U+212B              | `Å`  | U+C5              | "Use U+C5 instead of U+212B"    |
| U+2329              | `⟨`  | U+FFFD            | "Use U+27E8 instead of U+2329"  |
| U+232A              | `⟩`  | U+FFFD            | "Use U+27E9 instead of U+232A"  |
| U+FB00              | `ff` | U+66 U+66         | "Use U+66 U+66 instead of U+FB00 |
| U+FB01              | `fi` | U+66 U+69         | "Use U+66 U+69 instead of U+FB01 |
| U+FB02              | `fl` | U+66 U+6C         | "Use U+66 U+6C instead of U+FB02 |
| U+FB03              | `ffi`| U+66 U+66 U+66    | "Use U+66 U+66 U+69 instead of U+FB03 |
| U+FB04              | `ffl`| U+66 U+66 U+6C    | "Use U+66 U+66 U+6C instead of U+FB04 |
| U+FB05              | `ſt` | U+17F U+74        | "Use U+17F U+74 instead of U+FB05 |
| U+FB06              | `st` | U+73 U+74         | "Use U+73 U+74 instead of U+FB06 |
| U+1D455             | `ℎ`  | U+FFFD            | "Use U+210E instead of U+1D455" |
| U+1D49D             | `ℬ`  | U+FFFD            | "Use U+212C instead of U+1D49D" |
| U+1D4A0             | `ℰ`  | U+FFFD            | "Use U+2130 instead of U+1D4A0" |
| U+1D4A1             | `ℱ`  | U+FFFD            | "Use U+2131 instead of U+1D4A1" |
| U+1D4A3             | `ℋ`  | U+FFFD            | "Use U+210B instead of U+1D4A3" |
| U+1D4A4             | `ℐ`  | U+FFFD            | "Use U+2110 instead of U+1D4A4" |
| U+1D4A7             | `ℒ`  | U+FFFD            | "Use U+2112 instead of U+1D4A7" |
| U+1D4A8             | `ℳ`  | U+FFFD            | "Use U+2133 instead of U+1D4A8" |
| U+1D4AD             | `ℛ`  | U+FFFD            | "Use U+211B instead of U+1D4AD" |
| U+1D4BA             | `ℯ`  | U+FFFD            | "Use U+212F instead of U+1D4BA" |
| U+1D4BC             | `ℊ`  | U+FFFD            | "Use U+210A instead of U+1D4BC" |
| U+1D4C4             | `ℴ`  | U+FFFD            | "Use U+2134 instead of U+1D4C4" |
| U+1D506             | `ℭ`  | U+FFFD            | "Use U+212D instead of U+1D506" |
| U+1D50B             | `ℌ`  | U+FFFD            | "Use U+210C instead of U+1D50B" |
| U+1D50C             | `ℑ`  | U+FFFD            | "Use U+2111 instead of U+1D50C" |
| U+1D515             | `ℜ`  | U+FFFD            | "Use U+211C instead of U+1D515" |
| U+1D51D             | `ℨ`  | U+FFFD            | "Use U+2128 instead of U+1D51D" |
| U+1D53A             | `ℂ`  | U+FFFD            | "Use U+2102 instead of U+1D53A" |
| U+1D53F             | `ℍ`  | U+FFFD            | "Use U+210D instead of U+1D53F" |
| U+1D545             | `ℕ`  | U+FFFD            | "Use U+2115 instead of U+1D545" |
| U+1D547             | `ℙ`  | U+FFFD            | "Use U+2119 instead of U+1D547" |
| U+1D548             | `ℚ`  | U+FFFD            | "Use U+211A instead of U+1D548" |
| U+1D549             | `ℝ`  | U+FFFD            | "Use U+211D instead of U+1D549" |
| U+1D551             | `ℤ`  | U+FFFD            | "Use U+2124 instead of U+1D551" |
| [CJK Compatibility Ideographs] | | [Standardized Variant] | "Use Standardized Variants instead of CJK Compatibility Ideographs" |

### Main Table

| Sequence            | aka  | Replacement       | Error                                     |
| ------------------- | ---- | ----------------- | ----------------------------------------- |
| U+D U+A             | CRLF | U+A               | "Use U+A to terminate a line"             |
| U+D                 | CR   | U+A               | "Use U+A to terminate a line"             |
| U+C                 | FF   | U+20              | "Control character not valid in text"     |
| U+1B U+5B \[U+20–U+3F\]\* U+6D                     | SGR | | "Color escape sequences are not enabled" |
| \[U+1B\]+ U+5B \[U+20–U+3F\]\* \[U+40–U+7E\]?      | CSI | | "Unrecognized escape sequence" |
| \[U+1B\]+ U+5D \[\^U+7,U+18,U+1B\]\* \[U+7,U+18\]? | OSC | | "Unrecognized escape sequence" |
| \[U+1B\]+ \[U+40–U+7E\]?                           | ESC | | "Unrecognized escape sequence" |
| \[U+1B\]+ U+5B U+5B \[U+–U+7F\]?                   |     | | "Unrecognized escape sequence" |
| \[U+0–U+8,U+B,U+E–U+1F\] | C0 | U+FFFD         | "Control character not valid in text"     |
| U+7F                | DEL  | U+FFFD            | "Control character not valid in text"     |
| U+85                | NEL  | U+20              | "Control character not valid in text"     |
| \[U+80–U+84,U+86–U+9F\] | C1 | U+FFFD          | "Control character not valid in text"     |
| U+149               | `ʼn` | U+2BC U+6E        | "Use U+2BC U+6E instead of U+149"         |
| U+673               | `ا ٟ` | U+627 U+65F       | "Use U+627 U+65F instead of U+673"        |
| U+F77               | `◌ྲ◌ཱྀ` | U+FB2 U+F81       | "Use U+FB2 U+F81 instead of U+F77"        |
| U+F79               | `◌ླ◌ཱྀ` | U+FB3 U+F81       | "Use U+FB3 U+F81 instead of U+F79"        |
| U+17A3              | `អ`  | U+17A2            | "Use U+17A2 instead of U+17A3"            |
| U+17A4              | `អា` | U+17A2 U+17B6     | "Use U+17A2 U+17B6 instead of U+17A4"     |
| U+17B4              |      | U+FFFD            | "Unicode discourages use of U+17B4"       |
| U+17B5              |      | U+FFFD            | "Unicode discourages use of U+17B5"       |
| U+17D8              |      | U+FFFD            | "Unicode discourages use of U+17D8"       |
| U+2028              | LS   | U+20              | "Line separation is a rich-text function" |
| U+2029              | PS   | U+20              | "Paragraph separation is a rich-text function" |
| \[U+206A–U+206F\]   |      | U+FFFD            | "Deprecated Format Characters are deprecated" |
| U+2DF5              | ` ⷵ`  | U+2DED U+2DEE     | "Use U+2DED U+2DEE instead of U+2DF5"     |
| \[U+FDD0–U+FDEF\]   |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| U+FEFF              | BOM  | U+2060            | "U+FEFF is not necessary in Basic Text"   |
| \[U+FFF9–U+FFFB\]   |      | U+FFFD            | "Interlinear Annotations depend on out-of-band information" |
| U+FFFC              | ORC  | U+FFFD            | "U+FFFC depends on out-of-band information" |
| \[U+FFFE,U+FFFF\]   |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+1FFFE,U+1FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+2FFFE,U+2FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+3FFFE,U+3FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+4FFFE,U+4FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+5FFFE,U+5FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+6FFFE,U+6FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+7FFFE,U+7FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+8FFFE,U+8FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+9FFFE,U+9FFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+AFFFE,U+AFFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+BFFFE,U+BFFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+CFFFE,U+CFFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+DFFFE,U+DFFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| U+E0001             |      | U+FFFD            | "Language tagging is a deprecated mechanism" |
| \[U+EFFFE,U+EFFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+FFFFE,U+FFFFF\] |      | U+FFFD            | "Noncharacters are intended for internal use only" |
| \[U+10FFFE,U+10FFFF\] |    | U+FFFD            | "Noncharacters are intended for internal use only" |

## Conversion

### From Unicode string to Basic Text string

To convert a [Unicode] string into a Basic Text string in a manner that always
succeeds, discarding information not usually considered meaningful or valid in
plain text:
 - If the string starts with a [non-starter] or U+200D (ZWJ), replace it with
   U+FFFD.
 - Perform the Replacement actions from the [Pre-NFC Table].
 - Perform the [Stream-Safe Text Process (UAX15-D4)].
 - Perform `toNFC` with the [Normalization Process for Stabilized Strings].
 - When [*NEL Compatibility*] is enabled, replace any U+85 with U+A.
 - When [*LSPS Compatibility*] is enabled, replace any U+2028 or U+2029 with
   U+A.
 - Perform the Replacement actions from the [Main Table].

[*NEL Compatibility*]: #options
[*LSPS Compatibility*]: #options

#### Options

The following boolean options may be enabled:

| Name               | Type    | Default |
| ------------------ | ------- | ------- |
| NEL Compatibility  | Boolean | `false` |
| LSPS Compatibility | Boolean | `false` |

### From Unicode string to Basic Text string, strict

To convert a [Unicode] string into a Basic Text string in a manner that
discards information not usually considered meaningful and otherwise fails if
the content is not valid Basic Text:
 - If the string starts with a [non-starter] or U+200D (ZWJ), error with
   "Basic Text string must begin with a starter other than ZWJ"
 - Perform the Error actions from the [Pre-NFC Table].
 - Perform the [Stream-Safe Text Process (UAX15-D4)].
 - Perform `toNFC` with the [Normalization Process for Stabilized Strings].
 - Perform the Error actions from the [Main Table].
 - When [*CRLF Compatibility*] is enabled, replace any U+A with U+D U+A.

[*CRLF Compatibility*]: #options

#### Options

The following options may be enabled:

| Name               | Type    | Default |
| ------------------ | ------- | ------- |
| CRLF Compatibility | Boolean | `false` |

### From Unicode stream to Basic Text stream

To convert a [Unicode] stream into a Basic Text stream in a manner than always
succeeds, discarding information not usually considered meaningful or valid in
plain text:
 - If the stream starts with U+FEFF, remove it.
 - Perform [From Unicode string to Basic Text string].
 - If the stream is non-empty and doesn't end with U+A, append a U+A.

### From Unicode stream to Basic Text stream, strict

To convert a [Unicode] stream into a Basic Text stream in a manner that
discards information not usually considered meaningful and otherwise fails if
the content is not valid Basic Text:
 - When [*BOM Compatibility*] is enabled, insert a U+FEFF at the beginning of
   the stream.
 - Perform [From Unicode string to Basic Text string, strict].
 - If the stream is non-empty and doesn't end with U+A, error with
   "Basic Text stream must be empty or end with newline".

[Pre-NFC Table]: #pre-nfc-table
[Main Table]: #main-table
[From Unicode string to Basic Text string]: #from-unicode-string-to-basic-text-string
[From Unicode string to Basic Text string, strict]: #from-unicode-string-to-basic-text-string-strict
[*BOM Compatibility*]: #options

#### Options

The following options may be enabled:

| Name               | Type    | Default |
| ------------------ | ------- | ------- |
| BOM Compatibility  | Boolean | `false` |

[NFC]: https://unicode.org/reports/tr15/#Norm_Forms
[Stream-Safe]: https://unicode.org/reports/tr15/#Stream_Safe_Text_Format
[Stream-Safe Text Process (UAX15-D4)]: https://unicode.org/reports/tr15/#UAX15-D4
[Standardized Variant]: https://www.unicode.org/Public/UNIDATA/StandardizedVariants.txt
[Normalization Process for Stabilized Strings]: https://unicode.org/reports/tr15/#Normalization_Process_for_Stabilized_Strings
[Restricted Text]: RestrictedText.md
[Unicode]: Unicode.md
[CJK Compatibility Ideographs]: http://www.unicode.org/versions/latest/ch23.pdf#G19053
