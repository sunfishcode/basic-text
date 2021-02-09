<div align="center">
  <h1><code>restricted-text</code></h1>

  <p>
    <strong>Restricted Text strings and I/O streams</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/restricted-text/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/restricted-text/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://crates.io/crates/restricted-text"><img src="https://img.shields.io/crates/v/restricted-text.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/restricted-text"><img src="https://docs.rs/restricted-text/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

*This is not yet even an early experiment! The Restricted Text format is 
actively evolving and the code doesn't work at all yet!*

`restricted-text` defines several utilities for working with a subset of Unicode
called [Restricted Text]:

 - [`RestrictedString`] and [`RestrictedStr`] are similar to [`TextString`] and
   [`TextStr`] using Restricted Text. Restricted Text excludes some forms of
   visual ambiguity.
   
 - [`RestrictedReader`] and [`RestrictedWriter`] are input and output streams
   which use the Restricted Text format. On input *and* output, non-restricted
   content is diagnosed as errors.

 - [`RestrictedDuplexer`] is a [`Duplex`] for reading and writing on an
   interactive stream using Restricted Text.

[`RestrictedString`]: https://docs.rs/basic-text/latest/basic_text/struct.RestrictedString.html
[`RestrictedStr`]: https://docs.rs/basic-text/latest/basic_text/struct.RestrictedStr.html
[`RestrictedReader`]: https://docs.rs/basic-text/latest/basic_text/struct.RestrictedReader.html
[`RestrictedWriter`]: https://docs.rs/basic-text/latest/basic_text/struct.RestrictedWriter.html
[`RestrictedDuplexer`]: https://docs.rs/basic-text/latest/basic_text/struct.RestrictedDuplexer.html
[`TextString`]: https://docs.rs/basic-text/latest/basic_text/struct.TextString.html
[`TextStr`]: https://docs.rs/basic-text/latest/basic_text/struct.TextStr.html
[`Duplex`]: https://docs.rs/duplex/latest/duplex/trait.Duplex.html
[Restricted Text]: https://github.com/sunfishcode/basic-text/blob/main/docs/RestrictedText.md
