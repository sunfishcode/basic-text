# Formats

This repository describes and implements three formats.

 - [UTF-8](utf-8.md)
   - Anything which is valid UTF-8.

 - [Plain Text](plain-text.md)
   - Supports the semantics of any practical Unicode text content.
   - (not to be confused with cryptography's unrelated concept of "plaintext")
   - The phrase "Plain text" is used informally in many contexts, but
     here it is used to refer to a specific format.
   
 - [Restricted Text](restricted-text.md)
   - Plain Text with restrictions.
   - Reduced support for historical scripts, multiple-script text,
     formatting, and symbols in exchange for reduced visual ambiguity
     and simplified processing.
   - TODO: This isn't implemented yet.

The [background information] contains rationale and source information.

[background information]: background.md
