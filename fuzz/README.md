# Fuzz targets for `bytestreams`

This crate defines various `libFuzzer` targets for `bytestreams` which can be run
via `cargo-fuzz` plugin.

## Example

1. Install `cargo-fuzz` plugin:

```
cargo install cargo-fuzz
```

2. Install `nightly` toolchain:

```
rustup toolchain add nightly
```

3. Fuzz away:

```
cargo +nightly fuzz run text_reader
```
