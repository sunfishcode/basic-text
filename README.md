<div align="center">
  <h1><code>text-streams</code></h1>

  <p>
    <strong>Plain and restricted text streams</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/text-streams/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/text-streams/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://crates.io/crates/text-streams"><img src="https://img.shields.io/crates/v/text-streams.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/text-streams"><img src="https://docs.rs/text-streams/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

This is an early experiment! The API and feature set are likely to
evolve significantly.

`text-streams` defines several byte-oriented stream types.

 - [`Utf8Reader`] and [`Utf8Writer`] implement [`ReadExt`] and [`WriteExt`] and
   wrap arbitrary `ReadExt` and `WriteExt` streams. `Utf8Reader` translates
   invalid UTF-8 encodings into replacements (U+FFFD), while `Utf8Writer`
   reports errors on invalid UTF-8 encodings. Both ensure that scalar values
   are never split at the end of a buffer. [`Utf8ReaderWriter`] is the same
   for `ReadWriteExt`.

 - [`TextReader`] and [`TextWriter`] are similar to `Utf8Reader` and
   `Utf8Writer` but use the [Text] format, which disallowed control codes,
   deprecated characters, and other undesirable content. [`TextReaderWriter`]
   is the same for `Utf8ReaderWriter`.

[`Utf8Reader`]: https://docs.rs/text-streams/latest/text_streams/struct.Utf8Reader.html
[`Utf8Writer`]: https://docs.rs/text-streams/latest/text_streams/struct.Utf8Writer.html
[`Utf8ReaderWriter`]: https://docs.rs/text-streams/latest/text_streams/struct.Utf8ReaderWriter.html
[`TextReader`]: https://docs.rs/text-streams/latest/text_streams/struct.TextReader.html
[`TextWriter`]: https://docs.rs/text-streams/latest/text_streams/struct.TextWriter.html
[`TextReaderWriter`]: https://docs.rs/text-streams/latest/text_streams/struct.TextReaderWriter.html
[`ReadExt`]: https://docs.rs/io-ext/latest/io_ext/trait.ReadExt.html
[`WriteExt`]: https://docs.rs/io-ext/latest/io_ext/trait.WriteExt.html
[Text]: docs/text.md
