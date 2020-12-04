# Background

## IETF RFC 8264 "PRECIS"

[PRECIS] is mostly focused on identifiers and has several restrictions that
are inappropriate for streams such as disallowing whitespace, but it also
has a Freeform Class (4.3) which is similar in spirit to, and one of the
inspirations of, the formats defined here.

[PRECIS]: https://tools.ietf.org/html/rfc8264

## Newlines

We interpret U+000A, and nothing else, to mean "newline".

Why not use the CRLF convention? It's what [IETF RFCs] use, and as of
ASCII-1986 / ECMA-6:1985 at least, its what ASCII itself uses.
 - POSIX is an IEEE standard, and it uses the newline convention.
 - C is an ISO standard, and it uses the newline convention in character
   constants and string literals.
 - The newline convention is only one byte, so it's simpler than CRLF and
   avoids corner-case concerns of what to do when standalone CR or LF are
   encountered in various situations.
 - All practical text editors and viewers today support either line-ending
   convention, [even Windows Notepad].

Text input implicitly translates CRLF into newline, and text output has an
option to translate newlines into CRLF, which is intended to ease
compatibility with CRLF environments and IETF RFCs.

Why not use U+0085 (NEL)? In theory it has the semantics we want, however:
 - It isn't used in many places, or supported in many environments.
 - It's in the C1 control block, which is otherwise obscure and obsolete.
 - It's a two-byte code in UTF-8, compared to U+000A being a one-byte code.

Why not follow the [Unicode Newline Guidelines' Recommendations]?
 - We generally don't know the exact usage of any *NLF*.
 - We effectively target a virtual platform where the platform *NLF* is newline.
 - PS and LS aren't widely recognized or used as line separators in plain text.
 - FF is debatable; leaving it out simplifies the system by reducing the set of
   things text can do, but that could be revisited.

[Unicode Newline Guidelines' Recommendations]: https://www.unicode.org/standard/reports/tr13/tr13-5.html#Recommendations
[IETF RFCs]: https://www.rfc-editor.org/old/EOLstory.txt
[even Windows Notepad]: https://devblogs.microsoft.com/commandline/extended-eol-in-notepad/
[Unicode Newline Guidelines' Recommendations]: https://www.unicode.org/standard/reports/tr13/tr13-5.html#Recommendations

## NFC

[Is including NFC the right thing to do?](nfc.md).

## Relationship to POSIX Text Files and Printable Files

A [*text file* in POSIX]:
 - consists of zero or more [*lines* in POSIX], which all end in newlines
 - excludes NUL
 - lines are at most `LINE_MAX` bytes long including the newline (TODO).

[Plain Text] excludes NUL (it's a C0 control), and requires content to
consist of lines which all end in newlines.

TODO: Should we have a `LINE_MAX`-like restriction?

A [*printable file* in POSIX] is a text file which contains no control
codes other than [*whitespace* in POSIX] (space, tab, newline, carriage-return (TODO),
form-feed (TODO), and vertical-tab (TODO)) and [*backspace* in POSIX] (typically U+0008) (TODO).

[Plain Text] excludes most of the same control codes. It doesn't include
carriage-return, form-feed, vertical-tab, or backspace, as line printer
commands aren't part of plain text content.

[*printable file* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_288
[*text file* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_403
[*lines* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_206
[*whitespace* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_442
[*backspace* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_38
[Plain Text]: plain-text.md
[Restricted Text]: restricted-text.md

## Relationship to Wikipedia's "plain text"

The plain text format here is intended to align with the use cases
described in the [Wikipedia article on plain text]. The character encoding is
known, all characters are either printable or have behavior relevant to
simple text display.

[Wikipedia article on plain text]: https://en.wikipedia.org/wiki/Plain_text

## Relationship to Unicode's "plain text"

The plain text format here is a more specific version of the
[Unicode definition of "plain text"]. Unicode says

> Plain text must contain enough information to permit the text
> to be rendered legibly, and nothing more.

however it include code points which ring the terminal bell and other
side effects, it often includes redundant ways to encode the same
logical content, it includes numerous compatibility mechanisms, and
it contains flexibility for parties with private agreements.

The [Plain Text] format here is more focused on being just a plain text
format with just enough information to permit the text to be rendered
legibly.

[Unicode definition of "plain text"]: https://www.unicode.org/versions/Unicode13.0.0/ch02.pdf#G642

## TODO

May be interesting to discuss:
 - https://tools.ietf.org/html/rfc678
 - https://tools.ietf.org/html/rfc7994
