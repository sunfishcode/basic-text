<div align="center">
  <h1><code>basic-text</code></h1>

  <p>
    <strong>Basic Text strings and I/O streams</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/basic-text/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/basic-text/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://crates.io/crates/basic-text"><img src="https://img.shields.io/crates/v/basic-text.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/basic-text"><img src="https://docs.rs/basic-text/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

*This is an early experiment! Both the code and the Basic Text format are actively evolving!*

`basic-text` defines several utilities for working with a subset of Unicode
called [Basic Text]:

 - [`TextString`] and [`TextStr`] are similar to [`String`] and [`str`] using
   Basic Text. Basic Text excludes things like unprintable control characters,
   escape sequences, and non-canonical sequences, but preserves any practical
   Unicode textual content.

 - [`TextReader`] and [`TextWriter`] are input and output streams which use the
   Basic Text format, and which ensure a consistent line-ending convention.
   On input, non-text content is lossily converted, and on output, non-text
   content is diagnosed as errors.

 - [`TextDuplexer`] is a [`Duplex`] for reading and writing on an interactive
   stream using Basic Text.

[`TextString`]: https://docs.rs/basic-text/latest/basic_text/struct.TextString.html
[`TextStr`]: https://docs.rs/basic-text/latest/basic_text/struct.TextStr.html
[`TextReader`]: https://docs.rs/basic-text/latest/basic_text/struct.TextReader.html
[`TextWriter`]: https://docs.rs/basic-text/latest/basic_text/struct.TextWriter.html
[`TextDuplexer`]: https://docs.rs/basic-text/latest/basic_text/struct.TextDuplexer.html
[`str`]: https://doc.rust-lang.org/std/primitive.str.html
[`String`]: https://doc.rust-lang.org/std/string/struct.String.html
[`Duplex`]: https://docs.rs/duplex/latest/duplex/trait.Duplex.html
[Basic Text]: docs/BasicText.md
