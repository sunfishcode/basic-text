# Is including NFC the right thing to do?

At this time, it does seem to be. The following are some notes.

## What are the advantages of normalizing?

 - Portability - Text that isn't normalized is sometimes interpreted and
   displayed in different ways, depending on the environment. Normalization
   ensures that, in aspects related to normalization, content is independent of
   the environment.

   A common argument for non-normalized text is that some fonts render them
   differently from their normalized counterparts, and users may specifically
   wish to use the non-normalized versions. However, content that does this may
   not display properly in other environments using different fonts, so we
   specifically want to avoid such situations.

 - Avoiding common application bugs - Normalization eliminates some situations
   where two strings that look the same contain different scalar values, making
   content easier to work with.

## Which normalization form?

NFC seems to be [by far the most widely used for text interchange], and mostly
preserves the meaning of all practical Unicode text (see the following sections
for more discussion), so it seems the best choice for the [Basic Text] format.

Requiring that everything be compatibility-normalized can eliminate several
cases of visual ambiguity, and NFKC is a subset of NFC, so it seems the best
choice for the [Restricted Text] format.

[by far the most widely used for text interchange]: https://sites.google.com/site/macchiato/unicode/nfc-faq#TOC-How-much-text-is-already-NFC-

## Specific concerns

The following are some notes from researching situations where NFC has been
thought to be semantically lossy.

### CJK Compatibility Ideographs

Unicode includes 1002 CJK Compatibility Ideograph scalar values which were
originally intended only for use in preserving round-trip compatibility with
other character set standards. However, many of them are associated with
slightly different appearances, and this has led to a lot of confusion and some
dispute.

For example, the scalar value U+2F8A6 canonically decomposes to U+6148. This
means that Unicode considers these two scalar values to be canonically
equivalent, such that they are required have
[the same visual appearance and behavior]. Some systems do treat them this way,
however many popular systems today display them slightly differently.

[the same visual appearance and behavior]: https://unicode.org/reports/tr15/#Canon_Compat_Equivalence

Users understandably expect that the difference in appearance is significant
and will use non-canonical forms specifically for their unique appearance:

 - https://lists.w3.org/Archives/Public/public-i18n-core/2009JanMar/0216.html
 - https://www.w3.org/wiki/I18N/CanonicalNormalizationIssues#Problems_with_canonical_singletons

At one point, the Unicode committee even
[considered defining "variant normal forms"] which would be identical to NFC
and NFD except for excluding these CJK Compatibility Ideographs, however did
not end up pursuing the idea.

[considered defining "variant normal forms"]: https://www.unicode.org/review/pr-7b.html

[As of Unicode 6.3], all 1002 of these scalar values have standardized
variations which allow them to be normalized into a form which records the
scalar value they were normalized from. Permissive conversion into [Basic Text]
includes a rule which uses these variation sequences instead of the standard
canonical decompositions, which produces valid NFC output, but unlike plain
`toNFC` preserves the information about which specific CJK Compatibility
Ideographs were used.

At this time, it appears most implementations don't currently implement these
variation sequences, so the characters in this form still unfortunately will
often not display as intended. But at least this way, all the information is
preserved, so if implementations wish to implement them, they can be displayed
as intended.

[As of Unicode 6.3]: http://www.unicode.org/versions/Unicode6.3.0/#Summary

### Biblical Hebrew

According to Unicode, this was once a problem, but [there's a fix now].

[there's a fix now]: https://unicode.org/faq/normalization.html#10

### Bugs in implementations and fonts

Many apparent issues with NFC turn out to be issues with specific
implementations or specific fonts, which tend to fade away over time as
software is updated. Such issues are not considered here.

An example of this is [here](https://phabricator.wikimedia.org/T7948).

### Greek Polytonic Support

Early versions of Unicode appear to have used a confusing appearance for the
TONOS mark, and several fonts developed at the time did as well. See:

 - https://www.opoudjis.net/unicode/unicode_gkbkgd.html

Unicode was updated to use a different appearance, and newer fonts seem to use
it, and this seems to be a satisfactory solution.

### Greek Ano Teleia (U+387)

Unicode canonicalizes the Greek Ano Teleia (U+387) into Middle Dot (U+B7),
which doesn't preserve its appearance and creates problems with parsing because
the Greek actual ano teleia is considered punctuation, however Middle Dot is
considered an identifier character (reflecting its usage in Catalan, for
example). See the following links for details:

 - http://archives.miloush.net/michkap/archive/2011/05/20/10166588.html
 - https://op111.net/2008/03/17/linux-greek-punctuation-ano-teleia/

The Unicode Standard's explanation, in section 7.2 Greek, paragraph
Compatibility Punctuation, is:

> ISO/IEC 8859-7 and most vendor code pages for Greek simply make use of
[...] middle dot for the punctuation in question. Therefore, use of [...] U+387
is not necessary for interoperating with legacy Greek data, and [its] use is
not generally encouraged for representation of Greek punctuation.

According to (English) Wikipedia, U+387 is [infrequently encountered]. And Greek
Wikipedia seems to use U+387 and U+B7 [interchangeably].

[infrequently encountered]: https://en.wikipedia.org/wiki/Interpunct#Greek
[interchangeably]: https://el.wikipedia.org/wiki/%CE%86%CE%BD%CF%89_%CF%84%CE%B5%CE%BB%CE%B5%CE%AF%CE%B1

### W3C Guidance

The W3C says specs should not specify normalization for storage/interchange:

 - https://www.w3.org/TR/charmod-norm/#normalizationChoice

and suggests an approach where specs normalize only when needed, and ideally
only internally to other algorithms that need it.

The rationale can be summed up as:

> Normalization can remove distinctions that the users applied intentionally.

As discussed in the above sections, almost all of the places where information
about such distinctions seem to be lost either have satisfactory solutions,
or are caused by bugs or missing features in fonts or Unicode implementations.

There is also a difference in priorities; this plain text format is all about
building the foundations of a platform for the future, while the W3C is
concerned about helping users use the Web today, so this format is more
inclined to accept problems if they are believed to merely be limitations
of today's environments that can be fixed.

[Restricted Text]: RestrictedText.md
[Basic Text]: BasicText.md
