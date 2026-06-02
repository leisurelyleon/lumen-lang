# 3. Lexical scoping via an environment chain

- Status: Accepted
- Date: 2026-06

## Context

Functions must be first-class and must close over the variables in scope where
they were defined (lexical scoping), including being able to mutate captured
state across calls.

## Decision

Represent scopes as `Environment` nodes (a name→value map plus an optional
parent), held as `Rc<RefCell<Environment>>` so scopes can be shared and mutated.
A function value captures its defining environment; a call creates a child scope
whose parent is that captured environment.

## Consequences

- Closures capture and mutate state correctly and independently.
- `Rc<RefCell<…>>` moves some borrow checks to runtime; the evaluator takes and
  releases borrows in tight scopes to avoid conflicts.
- Scoping is lexical (by definition site), not dynamic (by call site).
