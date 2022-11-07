# djot-rs

An experimental Rust implementation of the [Djot](https://djot.net) light markup
language.

## Design Rules

Djot is in development, this defines _current_ design rules:

1. 100% compatibility with the reference Lua implementation, bugs and all. We
   don't want to fork a language which barely exist.
2. Close source compatibility with the reference Lua implementation. We want to
   makes it easy to incorporate changes, so we are writing idiomatic Lua, not
   idiomatic Rust :)

Once we get 100% of test suite working, we might think about code divergence.

It might be ok to diverge earlier starting at the AST layer.

Currently this is very incomplete, feel free to submit PR to fill the blank
spaces, just try to be close to the original code. There are some tests, run
with `cargo test`.
