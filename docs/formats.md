# Formats

This repository describes and implements three formats.

 - [UTF-8](utf-8.md)
   - Anything which is valid UTF-8.
   - Invalid bytes are translated to the Replacement Character (U+FFFD) on
     input, and rejected on output.

 - [Basic Text](basic-text.md)
   - Supports the semantics of any practical Unicode text content.
   - This is intended to realize the intuitive phrases "text" or "plain text"
     which are used in various ways in many contexts. It excludes control
     characters and other content impractical for text.
   - Invalid scalar values and sequences are translated to replacement
     sequences on input, and rejected on output.

 - [Restricted Text](restricted-text.md)
   - Like Basic Text, but aims to reduce visual ambiguity, trading off
     some support for historical scripts, multiple-script text, formatting,
     and symbols.
   - Invalid scalar values and sequences are rejected, even on input, since it
     isn't always possible to preserve intent automatically.
   - TODO: This isn't implemented yet.

The [background information] contains rationale and source information.

[background information]: background.md
