# Introduction

*Plain text* is an intuitive concept that plays an important role in
computing.

Reading plain text content shouldn't have side effects. This may seem obvious,
however Unicode contains multiple sets of control codes which effectively
form a bytecode language, with a variety of loosely-defined and
non-standardized side effects. Historically, text content and control codes were
often mixed together in the same encoding standards, and now, Unicode itself
must maintain compatibility with those standards. And, many of them continue
to be recognized, particularly in virtual terminals, such that one must be
careful about even displaying text from untrusted sources, for example in
[CVE-2017-10906], [CVE-2019-8325], and others.

[CVE-2017-10906]: https://nvd.nist.gov/vuln/detail/CVE-2017-10906
[CVE-2019-8325]: https://nvd.nist.gov/vuln/detail/CVE-2019-8325

Other than a few control codes for line endings and a few other things, most
of these control codes are obsolete and almost never used for normal purposes.
For cases where we want to work with plain text, and be sure it really is
just text, it would be useful to have a checkable *subset* of Unicode, which
excludes problematic control characters, and anything else not necessary for
modern practical plain text use cases.

And if we're defining a Unicode subset for plain text, we also have an
opportunity to rationalize line endings so that users don't have to think about
the old CRLF vs LF problem anymore. And we can restrict codepoints that Unicode
itself deprecates or discourages, but which Unicode itself can't drop because
of its need for round-trip compatibility with other character sets, so that
consumers that only need to work with Unicode have fewer things to think about.

There are existing standard subsets of Unicode doing similar things, such as
[PRECIS FreeformClass], [*printable files* in POSIX], and others, however
they either restrict codepoints that are frequently used in "plain text"
content, or don't restrict deprecated codepoints.

Basic Text is a subset of Unicode aiming to make it as simple as possible
(and no simpler) to work with plain text. It is not yet standardized, and
may evolve, but it is usable for many purposes.

## Development

Development of these pages, as well as a prototype implementation, is hosted
in [the Github repo].

[the Github repo]: https://github.com/sunfishcode/basic-text/

## Restricted Text

Basic Text can still be visually ambiguous. There are numerous ways that
two different codepoint sequences can have identical or similar appearances.
There are several techniques for mitigating this, but some of them are too
restrictive for general-purpose plain text, and thus too restrictive for
Basic Text. Restricted Text is a hypothetical format which collects such
techniques, and could in theory be developed into an actual format.

[PRECIS FreeformClass]: https://datatracker.ietf.org/doc/html/rfc8264#section-4.3
[*printable files* in POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_288
