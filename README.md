<div align="center">
  <h1><code>textual</code></h1>

  <p>
    <strong>Plain and restricted text I/O and strings</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/textual/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/textual/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://crates.io/crates/textual"><img src="https://img.shields.io/crates/v/textual.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/textual"><img src="https://docs.rs/textual/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

This is an early experiment! The API and feature set are likely to
evolve significantly.

`textual` defines several byte-oriented stream types.

 - [`Utf8Reader`] and [`Utf8Writer`] implement [`ReadExt`] and [`WriteExt`] and
   wrap arbitrary `ReadExt` and `WriteExt` implementations. `Utf8Reader`
   translates invalid UTF-8 encodings into replacements (U+FFFD), while
   `Utf8Writer` reports errors on invalid UTF-8 encodings. Both ensure that
   scalar values are never split at the end of a buffer. [`Utf8Interactor`] is
   the same for `InteractExt`.

 - [`TextReader`] and [`TextWriter`] are similar to `Utf8Reader` and
   `Utf8Writer` but use the [Text] format, which disallowed control codes,
   deprecated characters, and other undesirable content. [`TextInteractor`]
   is the same for `Utf8Interactor`.

 - [`TextStr`] and [`TextString`] are similar to [`str`] and [`String`] but
   again using the [`Text] format.

[`Utf8Reader`]: https://docs.rs/textual/latest/textual/struct.Utf8Reader.html
[`Utf8Writer`]: https://docs.rs/textual/latest/textual/struct.Utf8Writer.html
[`Utf8Interactor`]: https://docs.rs/textual/latest/textual/struct.Utf8Interactor.html
[`TextReader`]: https://docs.rs/textual/latest/textual/struct.TextReader.html
[`TextWriter`]: https://docs.rs/textual/latest/textual/struct.TextWriter.html
[`TextInteractor`]: https://docs.rs/textual/latest/textual/struct.TextInteractor.html
[`TextString`]: https://docs.rs/textual/latest/textual/struct.TextString.html
[`TextStr`]: https://docs.rs/textual/latest/textual/struct.TextStr.html
[`ReadExt`]: https://docs.rs/io-ext/latest/io_ext/trait.ReadExt.html
[`WriteExt`]: https://docs.rs/io-ext/latest/io_ext/trait.WriteExt.html
[`str`]: https://doc.rust-lang.org/std/primitive.str.html
[`String`]: https://doc.rust-lang.org/std/string/struct.String.html
[Text]: docs/text.md
