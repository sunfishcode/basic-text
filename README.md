<div align="center">
  <h1><code>plain-text</code></h1>

  <p>
    <strong>Stream types and traits</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/plain-text/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/plain-text/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://crates.io/crates/plain-text"><img src="https://img.shields.io/crates/v/plain-text.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/plain-text"><img src="https://docs.rs/plain-text/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

This is an early experiment! The API and feature set are likely to
evolve significantly.

`plain-text` defines several byte-oriented stream types.

 - [`Utf8Reader`] and [`Utf8Writer`] implement [`ReadExt`] and [`WriteExt`] and
   wrap arbitrary `ReadExt` and `WriteExt` streams. `Utf8Reader` translates
   invalid UTF-8 encodings into replacements (U+FFFD), while `Utf8Writer`
   reports errors on invalid UTF-8 encodings. Both ensure that scalar values
   are never split at the end of a buffer. [`Utf8ReaderWriter`] is the same
   for `ReadWriteExt`.

 - [`TextReader`] and [`TextWriter`] are similar to `Utf8Reader` and
   `Utf8Writer` but are for "plain text", which should not contain most control
   codes, escape sequences, other other content which may have a special meaning
   for common consumers. [`TextReaderWriter`] is the same for
   `Utf8ReaderWriter`.

[`Utf8Reader`]: https://docs.rs/plain-text/latest/plain_text/struct.Utf8Reader.html
[`Utf8Writer`]: https://docs.rs/plain-text/latest/plain_text/struct.Utf8Writer.html
[`Utf8ReaderWriter`]: https://docs.rs/plain-text/latest/plain_text/struct.Utf8ReaderWriter.html
[`TextReader`]: https://docs.rs/plain-text/latest/plain_text/struct.TextReader.html
[`TextWriter`]: https://docs.rs/plain-text/latest/plain_text/struct.TextWriter.html
[`TextReaderWriter`]: https://docs.rs/plain-text/latest/plain_text/struct.TextReaderWriter.html
[`ReadExt`]: https://docs.rs/io-ext/latest/io_ext/trait.ReadExt.html
[`WriteExt`]: https://docs.rs/io-ext/latest/io_ext/trait.WriteExt.html
