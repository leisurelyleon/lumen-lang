# The Lumen Language

Lumen is a small, dynamically-typed language. This is its reference.

## Values

- **number** — a 64-bit float (`42`, `3.14`)
- **string** — double-quoted (`"hello"`)
- **bool** — `true`, `false`
- **nil** — the absence of a value
- **function** — first-class; can be stored, passed, and returned

## Variables

```text
var x = 10;
x = x + 1;
```

A `var` declares a variable in the current scope; assignment updates the nearest
enclosing binding.

## Operators, by precedence (lowest to highest)

1. assignment `=`
2. logical `or`
3. logical `and`
4. equality `==` `!=`
5. comparison `<` `<=` `>` `>=`
6. term `+` `-`
7. factor `*` `/` `%`
8. unary `!` `-`
9. call `f(...)`

`+` concatenates two strings. Logical `and`/`or` short-circuit and return one of
their operands. `/` and `%` by zero are runtime errors.

## Control flow

```text
if (condition) { ... } else { ... }
while (condition) { ... }
```

A value is "truthy" unless it is `false` or `nil`.

## Functions and closures

```text
fn add(a, b) { return a + b; }
fn make_counter() {
var count = 0;
fn next() { count = count + 1; return count; }
return next;          // a closure capturing count
}
```

Functions are declared with `fn`, are first-class values, and capture their
defining scope. A function with no explicit `return` yields `nil`.

## Grammar (EBNF-style sketch)

```text
program     = declaration* ;
declaration = varDecl | funDecl | statement ;
varDecl     = "var" IDENT ( "=" expression )? ";" ;
funDecl     = "fn" IDENT "(" params? ")" block ;
statement   = exprStmt | printStmt | block | ifStmt | whileStmt | returnStmt ;
block       = "{" declaration* "}" ;
expression  = assignment ;
assignment  = IDENT "=" assignment | logicOr ;
logicOr     = logicAnd ( "or" logicAnd )* ;
logicAnd    = equality ( "and" equality )* ;
equality    = comparison ( ( "==" | "!=" ) comparison )* ;
comparison  = term ( ( "<" | "<=" | ">" | ">=" ) term )* ;
term        = factor ( ( "+" | "-" ) factor )* ;
factor      = unary ( ( "" | "/" | "%" ) unary ) ;
unary       = ( "!" | "-" ) unary | call ;
call        = primary ( "(" args? ")" )* ;
primary     = NUMBER | STRING | "true" | "false" | "nil" | IDENT | "(" expression ")" ;
```
