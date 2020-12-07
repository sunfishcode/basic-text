# Background

## NFC, Normalization

[Text] normalizes to NFC. See [this page](nfc.md) for details.

## Newlines

We interpret U+000A, and nothing else, to mean *newline*.

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
 - FF is debatable (see below).

PS and LS are valid in both [Text] and [Restricted Text], so higher-level
formats can use them, however they aren't treated as newlines as far as the
formats defined here are concerned.

One of the key observations here is that, at the layers these formats are
meant to be used, it isn't important to distinguish between paragraphs
and lines. That's a consideration for higher-level formats.

[Unicode Newline Guidelines' Recommendations]: https://www.unicode.org/standard/reports/tr13/tr13-5.html#Recommendations
[IETF RFCs]: https://www.rfc-editor.org/old/EOLstory.txt
[even Windows Notepad]: https://devblogs.microsoft.com/commandline/extended-eol-in-notepad/
[Unicode Newline Guidelines' Recommendations]: https://www.unicode.org/standard/reports/tr13/tr13-5.html#Recommendations

## Form Feed

Leaving U+000C (Form Feed) out simplifies the system by reducing the set of
things text can do. It does have [some uses], however it's fairly obscure and
in many higher-level protocols it either already doesn't work or there are
better alternatives.

And, there is some ambiguity about whether U+000C is meant to position the
cursor at the beginning of a line in the next page or at its previous column
in the next page, and about whether it should be counted as starting a
"new line", and it's not obviously worth the effort to try to describe what
this control code does.

And on devices where U+000C clears the current screen, that's a significant
side effect which could interfere with the visibility of other unrelated data.

U+0020 is chosen for translating U+000C so that it continues to function as
whitespace for parsing purposes, but doesn't indicate a new line or acquire any
new meaning.

[some uses]: https://en.wikipedia.org/wiki/Page_break#Semantic_use

## Horizontal Tab

It's tempting to disallow tab in a similar spirit of reducing the set of
things that text can do, however lots of text in practice uses and even
[depends on tab], so it's not practical to disallow.

[depends on tab]: https://www.gnu.org/software/make/manual/html_node/Recipe-Syntax.html

## Backspace, Delete, Vertical Tab

These appear in other "plain text" concepts. Here, plain text is meant
to mean text that doesn't include control codes for cursor positioning.

## Alert

Ringing the terminal bell is well outside the scope for plain text,
and theoretically could even be used for side-channel communication.

## Escape

Escape sequences can cause a wide variety of side effects. Plain text
shouldn't be able to have side effects.

## Deprecated codepoints

U+0149, U+0673, U+0F77, U+0F79, U+17A3, and U+17A4 are officially deprecated,
"their use is strongly discouraged", and they have recommended replacements.

U+2329 and U+232A have canonical equivalents with diffferent appearances
so their use is deprecated and it's not recommended to automatically replace
them with their canonical equivalents.

## Not-recomended unit name codepoints with singleton canonical decompositions

Unicode [recommends] the "regular letter" forms be used in preference
to the dedicated unit characters for U+2126 OHM SIGN, U+212A KELVIN SIGN,
and U+212B ANGSTROM SIGN.

[recommends]: https://www.unicode.org/versions/Unicode13.0.0/UnicodeStandard-13.0.pdf#G25.14143

## "Forbidden" codepoints

There were a few errors in the Unicode normalization algorithm in before
Unicode 4.1. The affected codepoints and sequences are identified as
[Forbidden Characters]. However, they are described as being very rare in
practice, and they're corrected since Unicode 4.1 published in 2005 (and
earlier in some cases), they're not restricted here.

[Forbidden Characters]: https://unicode.org/reports/tr15/#Forbidding_Characters

## Relationship to IETF RFC 8264 "PRECIS"

[PRECIS] is mostly focused on identifiers and has several restrictions that
are inappropriate for streams such as disallowing whitespace, but it also
has a [Freeform Class (4.3)] which is similar in spirit to, and one of the
inspirations of, the formats defined here.

PRECIS doesn't permit horizontal tab characters; we include them for the
reasons mentioned above.

[PRECIS]: https://tools.ietf.org/html/rfc8264
[Freeform Class (4.3)]: https://tools.ietf.org/html/rfc8264#section-4.3

## Relationship to POSIX Text Files and Printable Files

A [*text file* in POSIX]:
 - consists of zero or more [*lines* in POSIX], which all end in newlines
 - excludes NUL
 - lines are at most `LINE_MAX` bytes long including the newline (TODO).

[Text] excludes NUL (it's a C0 control), and requires content to consist of
lines which all end in newlines.

TODO: Should we have a `LINE_MAX`-like restriction?

A [*printable file* in POSIX] is a text file which contains no control
codes other than [*whitespace* in POSIX] (space, tab, newline, carriage-return (TODO),
form-feed (TODO), and vertical-tab (TODO)) and [*backspace* in POSIX] (typically U+0008) (TODO).

[Text] excludes most of the same control codes. It doesn't include
carriage-return, form-feed, vertical-tab, or backspace, as line printer
commands aren't part of plain text content.

[*printable file* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_288
[*text file* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_403
[*lines* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_206
[*whitespace* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_442
[*backspace* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_38
[Text]: text.md
[Restricted Text]: restricted-text.md

## Relationship to Wikipedia's "plain text"

The plain text format here is intended to align with the use cases
described in the [Wikipedia article on plain text]. The character encoding is
known, all characters are either printable or have behavior relevant to
simple text display.

[Wikipedia article on plain text]: https://en.wikipedia.org/wiki/Plain_text

## Relationship to Wikipedia's "text file"

TODO: https://en.wikipedia.org/wiki/Text_file

## Relationship to Unicode's "plain text"

The plain text format here is a more specific version of the
[Unicode definition of "plain text"]. Unicode says

> Plain text must contain enough information to permit the text
> to be rendered legibly, and nothing more.

however it include code points which ring the terminal bell and other
side effects, it often includes redundant ways to encode the same
logical content, it includes numerous compatibility mechanisms, and
it contains flexibility for parties with private agreements.

The [Text] format here is more focused on being just a plain text
format with just enough information to permit the text to be rendered
legibly.

[Unicode definition of "plain text"]: https://www.unicode.org/versions/Unicode13.0.0/ch02.pdf#G642

## TODO

May be interesting to discuss:
 - https://tools.ietf.org/html/rfc678
 - https://tools.ietf.org/html/rfc7994
