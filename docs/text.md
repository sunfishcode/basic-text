In addition to the transforms performed by `Utf8Reader`, an input text
stream implicitly applies the following transformations:
 - U+FEFF (BOM) scalar values are stripped
 - A '\n' is appended at the end of the stream if the stream wasn't
   empty and it doesn't already have one.
 - '\r' followed by '\n' is replaced by '\n'.
 - U+000C (FF) is replaced by ' '.
 - All other control codes other than '\n' and '\t' are replaced
   by U+FFFD (REPLACEMENT CHARACTER).
 - Text is transformed to Normalization Form C (NFC).
 - The Stream-Safe Text Process (UAX15-D4) is applied.
 - Streams never start or resume after a push with a normalization-form
   non-starter.

An output text stream enforces the following restrictions:
 - Data must be valid UTF-8.
 - U+FEFF (BOM) scalar values must not be present.
 - If any bytes have been written to the stream, '\n' is required at
   the end of the stream.
 - Control codes other than '\n' and '\t' most not be present.

An output text stream implicitly applies the following transformations:
 - Text is transformed to Normalization Form C (NFC).
 - The Stream-Safe Text Process (UAX15-D4) is applied.
 - Optionally, "\n" is translated to "\r\n".

FIXME: Disallow the following on output, which all have canonical-decomposition
singletons and are generally deprecated?
 - U+2000 - EN QUAD
 - U+2001 - EM QUAD
 - U+2126 - OHM SIGN
 - U+212A - KELVIN SIGN
 - U+212B - ANGSTROM SIGN
 - U+2329 - LEFT-POINTING ANGLE BRACKET
 - U+232A - RIGHT-POINTING ANGLE BRACKET

TODO: `canonical_combining_class` doesn't know about the astral
compositions like U+11099 U+110BA => U+1109A. Restrict non-starters
of that form too? Or use unicode-segmentation to detect grapheme boundaries.

TODO: support security restrictions? Or have a mode where they are supported?
  - [Unicode Restriction Levels](https://www.unicode.org/reports/tr39/#Restriction_Level_Detection)
  - [unicode-security crate](https://crates.io/crates/unicode-security)

TODO: Forbidden characters?
  - [11.4 Forbidden Characters](https://unicode.org/reports/tr15/#Forbidding_Characters)

TODO: Problem sequences?
  - [11.5 Problem Sequences](https://unicode.org/reports/tr15/#Corrigendum_5_Sequences)

TODO: Implement Stablized Strings
  - [12.1 Stablized Strings](https://unicode.org/reports/tr15/#Normalization_Process_for_Stabilized_Strings)

TODO: NFC is not closed under concatenation. Use CGJs to separate.

Reference IETF RFC8264 "PRECIS"
 - https://tools.ietf.org/html/rfc8264
 - PRECIS is mostly focused on identifiers and has several restrictions that
   are inappropriate for streams such as disallowing whitespace, but it also
   has a Freeform Class (4.3) which is roughly suitable for streams.
 - See 4.3.3 "Disallowed" and 4.3.4 "Unassigned"
     - Old Hangul Jamo (TODO: we can't disallow these)
     - Control code points (TODO: we need to allow \n and \t)
     - Ignorable code points (TODO: we can't disallow these)
     - Unassigned code points

CRLF handling:
 - Multics in 1964 seems to have invented the 0x0A = "newline" convention.
 - ASCII-1968 added "newline" as an alternative interpretation of 0x0A
 - ASCII-1986 / ECMA-6:1985 deprecated this, but the Unix world had already settled on '\n'.
 - To this day, IETF standads typically use CRLF.
    - https://www.rfc-editor.org/old/EOLstory.txt
 - Unix conventions remain popular, and most programs even on traditionally
   CRLF platforms can handle them, [even Notepad].
 - What about NEL? In theory it has the semantics we want, however:
    - It isn't used or supported in many places.
    - It's in C1 control block, which is otherwise obscure and obsolete.
    - It's a two-byte code in UTF-8, compared to U+000A being a one-byte code.
 - We have a CRLF output mode for conforming to IETF standards when needed,
   otherwise everything else is U+000A.

[even Nodepad]: https://devblogs.microsoft.com/commandline/extended-eol-in-notepad/

NFC. See [Is including NFC the right thing to do?](nfc.md).

FIXME: Reset the Right-to-left state?
 - That may not be our problem to solve?
 - https://unicode.org/reports/tr9/

FIXME: reserved codepoints

FIXME: Pull some NKFC normalizations into our NFC?
 - https://github.com/rust-lang/rust/issues/2253#issuecomment-29050949

# POSIX Text File
 - A [*text file* in POSIX]:
   - consists of zero or more [*lines* in POSIX], which all end in newlines
   - excludes NUL
   - lines are at most `LINE_MAX` bytes long including the newline (TODO).
 - A [*printable file*] in POSIX] is a text file which contains no control
   codes other than [*whitespace* in POSIX] (space, tab, newline, carriage-return (TODO),
   form-feed (TODO), and vertial-tab (TODO)) and [*backspace* in POSIX] (typically U+0008) (TODO).

[*printable file* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_288
[*text file* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_403
[*lines* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_206
[*whitespace* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_442
[*backspace* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_38
