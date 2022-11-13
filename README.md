# djot-rs

An experimental Rust implementation of the [Djot](https://djot.net) light markup
language.

## Design Rules

Djot is in development, this defines _current_ design rules:

1. 100% compatibility with the reference Lua implementation, bugs and all. We
   don't want to fork a language which barely exist.
2. Reasonable source compatibility with the reference Lua implementation. We
   want to makes it easy to incorporate changes, though we don't necessary want
   to bend Rust to be lua.

Currently this is very incomplete, feel free to submit PR to fill the blank
spaces, just try to be close to the original code.

There are some tests, run with `cargo test`. We are using the same test suite as
the upstream project (see `.test` files in `tests/data`)
