# lumen-lang

> A tree-walking interpreter for Lumen, a small dynamically-typed language.

Lumen is a small, dynamically-typed language with variables, arithmetic and
logic, comparisons, `if`/`else`, `while` loops, and first-class functions with
lexical closures — a complete, Turing-complete language implemented as a
tree-walking interpreter in Rust.

## A taste of Lumen

```
fn make_counter() {
    var count = 0;
    fn next() {
        count = count + 1;
        return count;
    }
    return next;
}

var counter = make_counter();
print counter(); // 1
print counter(); // 2
```

The closure `next` captures `count` from its defining scope — calling the
returned function repeatedly increments the captured variable.

## Architecture

```
lumen-core   the language: lexer, recursive-descent parser, tree-walking evaluator
lumen-cli    the binary: a REPL and a file runner
```

The implementation is the classic three-stage pipeline, each stage a pure
transformation that is unit-tested independently:

1. **Lexer** — source text becomes a sequence of tokens with positions.
2. **Parser** — tokens become an abstract syntax tree, with correct operator
   precedence via recursive descent.
3. **Interpreter** — the tree is evaluated against an environment chain, with
   functions capturing their lexical scope as closures.

See [`docs/architecture.md`](docs/architecture.md) and the language reference in
[`docs/language.md`](docs/language.md).

## Build & Test

```bash
cargo build
cargo test
```

## Run

```bash
# Start the REPL
cargo run -p lumen-cli -- repl

# Run a Lumen program
cargo run -p lumen-cli -- run examples/fib.lum
```

## License

MIT — see [LICENSE](LICENSE).
