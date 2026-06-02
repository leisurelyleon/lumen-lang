# Architecture

`lumen-lang` is a Rust workspace implementing a tree-walking interpreter for
Lumen, organized as the classic three-stage pipeline with each stage a pure,
independently-testable transformation.

## Crates

```text
lumen-core   the language: lexer, recursive-descent parser, tree-walking interpreter
lumen-cli    the binary: a REPL and a file runner
```

## The pipeline

```text
source text  --lex-->  tokens  --parse-->  AST  --interpret-->  values + output
```

1. **Lexer** (`lexer.rs`) — a hand-written scanner turning `&str` into a `Vec`
   of `Token`s, each carrying its line and column. Pure.
2. **Parser** (`parser.rs`) — recursive descent turning tokens into an AST of
   `Stmt`/`Expr` nodes, with operator precedence encoded by one method per
   precedence level. Pure.
3. **Interpreter** (`interpreter.rs`) — walks the AST, evaluating expressions to
   `Value`s and executing statements against an `Environment` chain. `print`
   output is collected into a buffer rather than written directly, so evaluation
   is pure and testable.

## Scoping and closures

`Environment` is a name→value map with an optional parent, held as
`Rc<RefCell<Environment>>` so scopes can be shared and mutated (the standard
idiom for a tree-walking interpreter in Rust). A function value captures the
environment in which it was defined; calling it creates a fresh scope whose
parent is that captured environment — which is what makes scoping lexical and
closures work. See [`docs/language.md`](language.md) for the language itself.

## Error handling

Each stage has its own positioned error type (`LexError`, `ParseError`,
`RuntimeError`), unified by `LumenError`. Runtime errors are returned as values,
never panics — a malformed program produces a clean error, not a crash.
