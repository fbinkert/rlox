# rlox

A tree-walk interpreter for the Lox language from Robert Nystrom’s book _Crafting Interpreters_, implemented in Rust.

This repository is my contribution to the _Crafting Interpreters Reading Group_ where we study the book together and implement the interpreter in parallel.

## Status

- [x] Scanner (lexer)
- [ ] Parser (AST)
- [ ] Interpreter (expressions & statements)
- [ ] Classes / inheritance (if not yet done)
- [ ] Standard library extensions

## Building

You’ll need a recent stable Rust toolchain (via [`rustup`](https://rustup.rs/)).

```bash
git clone https://github.com/fbinkert/rlox.git
cd rlox
cargo build --release
```

## Running

### REPL

```bash
cargo run
```

This starts an interactive Lox prompt.

### Run a file

```bash
cargo run -- path/to/script.lox
```

## References

- Book: [_Crafting Interpreters_](https://craftinginterpreters.com/) by Robert Nystrom
