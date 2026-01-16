# rlox

A tree-walk interpreter for the Lox language from Robert Nystrom’s book _Crafting Interpreters_, implemented in Rust.

## Building

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
