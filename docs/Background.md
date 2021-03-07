# Background

This document explains the decisions behind the Basic Text format and provides
links to related standards, documentation, and other resources.

## Overall approach to Basic Text

Basic Text's initial goal is to disallow as much as it can be without
compromising the meaning of any practical Unicode text. Be conservative in
what you accept, and be liberal in uncovering implicit assumptions.

As the format gains real-world exposure, we may discover places where it's
too strict and relax it as needed.

## Rationale

### NFC, Normalization

[Basic Text] normalizes to NFC, using a special algorithm to minimize loss
of intent. See [this page](NFC.md) for motivation and rationale.

### Newlines

In [Basic Text] content, U+A, and nothing else, is a *line terminator*,
sometimes also called a *newline*.

Why not use the CRLF convention? That's what [IETF RFCs] use, and after
ASCII-1986 / ECMA-6:1985 at least, that's what ASCII itself uses.
 - U+A is what IEEE [POSIX] and ISO C and C++ use in program data.
 - The newline convention is only one scalar, so it's simpler than CRLF and
   avoids corner-case concerns of what to do when CR and LF are split apart.
 - The newline convention is also only one byte in UTF-8, so it can be
   recognized without full UTF-8 decoding.
 - All practical text editors and viewers today support the U+A newline
   convention, [even Windows Notepad].

Lossy text conversion implicitly translates plain CR and CRLF into newline,
which is a common convention.

By default, lossy text conversion translates NEL, LS, and PS into U+20 which,
which for those rare formats which recognize these scalars at all, is
compatible with how they're typically treated. As options, lossy text
conversion can also translate NEL, or LS and PS, into newlines, for example to
support the text conventions used in [XML 1.1] and [JavaScript source code],
respectively.

[POSIX]: http://get.posixcertified.ieee.org/
[XML 1.1]: https://www.w3.org/TR/2006/REC-xml11-20060816/#sec-line-ends
[JavaScript source code]: https://www.ecma-international.org/ecma-262/5.1/#sec-7.3

By default, strict text conversion rejects CRLF and other line terminator
sequences other than U+A. As an option, strict text conversion can translate
U+A into CRLF, for example to support the text conventions used in [IETF RFCs].

Why not follow the [Unicode Newline Guidelines' Recommendations]?
 - We effectively target a virtual platform with U+A as the platform *NLF*.
 - Plain text does not have an inherent concept of paragraphs, so
   recommendation R2 isn't meaningful. Paragraphs are only meaningful in
   higher-level protocols (for example, see HTML's `<br>` and `<p>`).
 - Recommendation R4's inclusion of FF, LS, and PS seems to be universally
   ignored in line-reading functions of all mainstream programming languages
   we've surveyed.
 - NEL, LS, and PS are rare in practice, and formats which even recognize them
   are rare in practice.
 - Also, see the section on Form Feed below.

Plain text uses line *terminators*, rather than line *separators*. This means
that plain text streams end with a line terminator (if they are non-empty).
Lossy conversion implicitly adds a line terminator at the end if needed, and
strict conversion requires a line terminator at the end if needed.

[POSIX uses]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_243
[Unicode Newline Guidelines' Recommendations]: https://www.unicode.org/versions/Unicode13.0.0/ch05.pdf#G10213
[IETF RFCs]: https://www.rfc-editor.org/old/EOLstory.txt
[even Windows Notepad]: https://devblogs.microsoft.com/commandline/extended-eol-in-notepad/

### Form Feed

Pagination control is primarily a feature of higher-level protocols, and not
part of most informal notions of "plain text". U+C does have [some uses] in
practice, however it's fairly obscure, and often not recognized.

And even in places where U+C is recognized, there is ambiguity about what it
means. Implementations differ on whether it's meant to position the cursor at
the beginning of a line in the next page or at its previous column in
the next page. And, they differ on whether it should be considered a line
terminator.

And, on devices where U+C clears the current screen, that's a significant
side effect which could interfere with the visibility of other unrelated data.

So [Basic Text] excludes U+C. Lossy conversion translates it to U+20 so that it
continues to function as whitespace for parsing purposes, but all ambiguity
about its meaning is resolved.

[some uses]: https://en.wikipedia.org/wiki/Page_break#Semantic_use

### Tab

It might be tempting to disallow tab on the basis of it being a control
code primarily concerned with how text is aligned on the screen, which is
typically considered a feature of higher-level protocols. However, Tab's effects
are much more mild than other control codes, and in practice it has several uses,
some of which [require it], so we allow it.

We can refer to it as just "Tab" though, rather than "Horizontal Tab", since
[Basic Text] excludes Vertical Tab.

[require it]: https://www.gnu.org/software/make/manual/html_node/Recipe-Syntax.html

### Backspace, Delete, Vertical Tab

These do appear in some other "plain text" concepts, however they're rare in
practice. Here, plain text is meant to mean text that doesn't include control
codes for cursor positioning. Cursor positioning controls are widely used with
terminals, but that's a different use case than what [Basic Text] is targeting.

### Alert

Ringing the terminal bell is well outside the scope for plain text,
and theoretically could even be used for side-channel communication.

### Escape

Escape sequences can cause a wide variety of side effects. Plain text
shouldn't be able to have side effects.

Basic Text includes some fairly conservative regular expressions for matching
not just the U+1B, but also the sequences which commonly make up escape sequences,
such as CSI and OSC, so that entire sequences are cleanly ignored, as is common
with unrecognized escape sequences.

### Deprecated scalar values

U+149, U+673, U+F77, U+F79, U+17A3, and U+17A4 are officially deprecated,
"their use is strongly discouraged", and they have recommended replacements.

U+2329 and U+232A have canonical equivalents with different appearances
so their use is deprecated and it's not recommended to automatically replace
them with their canonical equivalents.

### Unassigned Mathematical Alphanumeric Symbols

In the Mathematical Alphanumeric Symbols block, the codepoint U+1D455 would be
the place for `ℎ`, however unicode already had an `ℎ` at U+210E, so U+1D455
was left unassigned.

Several other characters are treated similarly: U+9E4, U+9E5, U+A64, U+A65,
U+AE4, U+AE5, U+B64, U+B65, U+BE4, U+BE5, U+C64, U+C65, U+CE4, U+CE5, U+D64,
U+D65, U+2072, U+2073, U+1D455, U+1D49D, U+1D4A0, U+1D4A1, U+1D4A3, U+1D4A4,
U+1D4A7, U+1D4A8, U+1D4AD, U+1D4BA, U+1D4BC, U+1D4C4, U+1D506, U+1D50B,
U+1D50C, U+1D515, U+1D51D, U+1D53A, U+1D53F, U+1D545, U+1D547, U+1D548,
U+1D549, and U+1D551.

Unicode considers these codepoints unassigned, so they could potentially be
assigned new meanings in the future. Consequently, in Basic Text they convert
to U+FFFD rather than their designated replacements.

### Not-recommended scalar values with singleton canonical decompositions

Unicode [recommends] the "regular letter" forms be used in preference
to the dedicated unit characters for U+2126 OHM SIGN, U+212A KELVIN SIGN,
and U+212B ANGSTROM SIGN. They already canonically decompose to the regular
letter forms, so they're already excluded from NFC. Rejecting them in
strict conversion means that any assumptions about them being handled
differently from the regular letter forms will be promptly corrected.

[recommends]: https://www.unicode.org/versions/Unicode13.0.0/UnicodeStandard-13.0.pdf#G25.14143

### Characters Whose Use Is Discouraged

Khmer scalar values U+17B4 and U+17B5
"should be considered errors in the encoding". Also,
"the use of U+17D8 Khmer sign beyyal is discouraged".

For the Cyrillic value U+2DF5, Unicode [prefers] the sequence U+2DED U+2DEE.

[prefers]: https://www.unicode.org/versions/Unicode13.0.0/UnicodeStandard-13.0.pdf#G10.28571

### "Forbidden Characters"

There were a few errors in the Unicode normalization algorithm in before
Unicode 4.1. The affected scalar values and sequences are identified as
[Forbidden Characters]. However, they are described as being rare in practice,
and they're corrected since Unicode 4.1 published in 2005 (and earlier in some
gases), they're not restricted here.

[Forbidden Characters]: https://unicode.org/reports/tr15/#Forbidding_Characters

### "Ghost Characters"

[Ghost characters] are characters which don't correspond to any existing
written characters, and seem to have been created by accident. It's tempting
to restrict them, however, Unicode itself has not deprecated them, and it's
possible that they'll acquire meanings, so we don't restrict them here.

[Ghost characters]: https://www.dampfkraft.com/ghost-characters.html

### Hangul Compatibility Jamo

The Hangul Compatibility Jamo block in Unicode is one of the blocks added
to Unicode for compatibility with other standards, however it also turns
out to be practical, for example for [displaying isolated Jamo], so we
don't restrict them here.

[displaying isolated Jamo]: http://gernot-katzers-spice-pages.com/var/korean_hangul_unicode.html

### Noncharacters

[Noncharacters] are like [Private-Use Characters], except they are not intended
for interchange. These characters are not widely used, and when they are used,
there is often confusion about what they mean or whether they are valid. Since
they aren't text, we exclude them here to avoid the confusion.

Also, some implementations are unable to handle U+FFFE because in UTF-16 it
can interfere with endianness detection.

See also [Noncharacters in Markup].

Along with U+0, U+FFFC, and U+FFF9–U+FFFB, applications wishing to use these
for private use should use the plain [Unicode] format rather than the
[Basic Text] format.

[Noncharacters]: http://www.unicode.org/faq/private_use.html#noncharacters
[Private-Use Characters]: http://www.unicode.org/faq/private_use.html#pua1
[Noncharacters in Markup]: http://www.unicode.org/reports/tr20/tr20-9.html#Noncharacters

### Variation sequences

Basic Text does not restrict the [Variation sequences], because Unicode may
add new variation sequences over time. Restricted Text excludes the
variation sequences entirely.

[variation sequences]: http://unicode.org/faq/vs.html#3

### Characters requiring out-of-band information

Some characters require additional data not described by Unicode to properly
display.

U+FFFC (OBJECT REPLACEMENT CHARACTER) has no way to indicate which object it
references. See also [Object Replacement Character in Markup].

U+FFF9–U+FFFB, the Interlinear Annotation Characters, refer to external
information, and ignoring them may change the meaning of a text. See also
[Interlinear Annotation Characters in Markup].

[Object Replacement Character in Markup]: http://www.unicode.org/reports/tr20/tr20-9.html#Object
[Interlinear Annotation Characters in Markup]: http://www.unicode.org/reports/tr20/tr20-9.html#Interlinear

### C1 controls

See [Newlines] for more information about U+85.

The rest of the C1 controls are non-printing control codes rather than text.

[Newlines]: #newlines

## Relationships to other standards and conventions

### Relationship to IETF RFC 8264 "PRECIS"

[PRECIS] is mostly focused on identifiers and has several restrictions that
are inappropriate for streams such as disallowing whitespace, but it also
has a [Freeform Class (4.3)] which is similar in spirit to, and one of the
inspirations of, the formats defined here.

PRECIS doesn't permit tabs; we include them for the reasons mentioned above.

[PRECIS]: https://tools.ietf.org/html/rfc8264
[Freeform Class (4.3)]: https://tools.ietf.org/html/rfc8264#section-4.3

### Relationship to POSIX Text Files and Printable Files

A [*text file* in POSIX]:
 - consists of zero or more [*lines* in POSIX], which all end in newlines
 - excludes NUL
 - lines are at most `LINE_MAX` bytes long including the newline.

[Basic Text] excludes NUL (it's a C0 control), and requires content to consist
of lines which all end in newlines.

[Basic Text] has no `LINE_MAX`-like restriction.

A [*printable file* in POSIX] is a text file which contains no control
codes other than [*whitespace* in POSIX] (space, tab, newline, carriage-return,
form-feed, and vertical-tab) and [*backspace* in POSIX] (typically U+8).

[Basic Text] excludes most of the same control codes. It doesn't include
carriage-return, form-feed, vertical-tab, or backspace, as line printer
commands aren't part of plain text content.

[*printable file* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_288
[*text file* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_403
[*lines* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_206
[*whitespace* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_442
[*backspace* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_38
[Unicode]: Unicode.md
[Basic Text]: BasicText.md
[Restricted Text]: RestrictedText.md

### Relationship to Wikipedia's "plain text"

The plain text format here is intended to align with the use cases described in
the [Wikipedia article on plain text]. The character encoding is known, all
characters are either printable or have behavior relevant to simple text
display.

[Wikipedia article on plain text]: https://en.wikipedia.org/wiki/Plain_text

### Relationship to Unicode's "plain text"

The plain text format here is a more specific version of the
[Unicode definition of "plain text"]. Unicode says

> Plain text must contain enough information to permit the text
> to be rendered legibly, and nothing more.

however it include scalars which ring the terminal bell and other side effects,
it often includes redundant ways to encode the same logical content, it
includes numerous compatibility mechanisms, and it contains flexibility for
parties with private agreements.

The [Basic Text] format here is more focused on being just a plain text format
with just enough information to permit the text to be rendered legibly.

[Unicode definition of "plain text"]: https://www.unicode.org/versions/Unicode13.0.0/ch02.pdf#G642

### Relationship to "What makes a Unicode code point safe?"

The blog post ["What makes a Unicode code point safe?"] has a list of safety
criteria with much in common with the plain text format here. Both exclude
unassigned codepoints, noncharacters, private-use characters, surrogate
codepoints, and most control codes. And both require text be stable under
normalization.

The [Basic Text] format here permits format characters, whitespace characters,
punctuation, and combining characters, as they are commonly used in plain text
documents.

The [Restricted Text] format requires NFKC, which excludes many, though not
all, whitespace and formatting characters.

["What makes a Unicode code point safe?"]: https://qntm.org/safe

### Relationship to "Canonical Equivalence in Applications"

[Unicode Technical Note #5] describes various considerations related to
normalization, including two alternate normalization forms, called FCD
and FCC. We aren't using these here, mainly because we're using NFC (and
NFKC) and FCD and FCC aren't fully compatible with NFC.

[Unicode Technical Note #5]: https://www.unicode.org/notes/tn5/

### Relationship to Markup

[Unicode in XML and other Markup Languages] describes the relationship between
Unicode and markup languages. It includes recommendatations about
[Characters not Suitable for use With Markup]. Many of these recommendataions
are incoproated into [Basic Text], however some are specific to the needs of
markup languages, and Basic Text intends to be useful for plain text as well.

For example, Basic Text includes the Bidi control characters even though
[they are duplicated by markdup features].

[Unicode in XML and other Markup Languages]: http://www.unicode.org/reports/tr20/tr20-9.html#Object
[Characters not Suitable for use With Markup]: http://www.unicode.org/reports/tr20/tr20-9.html#Suitable
[they are duplicated by markdup features]: http://www.unicode.org/reports/tr20/tr20-9.html#Bidi
