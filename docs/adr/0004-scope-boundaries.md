# 4. Language scope boundaries

- Status: Accepted
- Date: 2026-06

## Context

A language can grow without bound (types, modules, a standard library, a VM).
A portfolio implementation should be complete enough to be real, but bounded.

## Decision

Implement a complete, Turing-complete core: variables, arithmetic/logic/
comparison, strings, `if`/`else`, `while`, and first-class functions with
closures. Deliberately omit: a static type system, a bytecode VM or native
codegen, a module/import system, and a large standard library.

## Consequences

- The language is genuinely usable (recursion, closures, control flow) — not a
  toy calculator.
- The omitted features are well-understood extensions, documented as out of
  scope rather than half-built.
- The focus stays on demonstrating the core concepts clearly and correctly.
