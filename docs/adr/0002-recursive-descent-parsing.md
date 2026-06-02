# 2. Recursive-descent parsing

- Status: Accepted
- Date: 2026-06

## Context

The parser must turn a token stream into an AST while respecting operator
precedence and associativity. Options include a parser generator or a
hand-written parser.

## Decision

Hand-write a recursive-descent parser with one method per precedence level
(assignment → or → and → equality → comparison → term → factor → unary → call →
primary). Each level calls the next-higher one, so precedence falls out of the
call structure.

## Consequences

- Precedence and associativity are explicit and easy to verify.
- No build-time code generation or grammar tooling.
- Adding an operator means slotting it into the right precedence method.
