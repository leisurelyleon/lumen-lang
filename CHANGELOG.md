# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial workspace scaffold: lumen-core, lumen-cli.

## [0.1.0] - TBD

### Added
- Hand-written lexer turning source text into tokens with positions.
- Recursive-descent parser with correct operator precedence, producing an AST.
- Tree-walking interpreter with variables, arithmetic, comparisons, logic,
  if/else, while loops, and first-class functions with lexical closures.
- A REPL and a file runner.
- Example Lumen programs: recursive fibonacci, closures, fizzbuzz.

[Unreleased]: https://github.com/leisurelyleon/lumen-lang/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/leisurelyleon/lumen-lang/releases/tag/v0.1.0
