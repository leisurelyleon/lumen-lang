# 1. A tree-walking interpreter

- Status: Accepted
- Date: 2026-06

## Context

A language can be executed by walking its syntax tree, by compiling to bytecode
for a VM, or by generating native code. These differ enormously in complexity
and in how testable the result is.

## Decision

Implement Lumen as a tree-walking interpreter: parse to an AST and evaluate the
tree directly. No bytecode, no codegen.

## Consequences

- The implementation is small, clear, and maps directly to language semantics.
- Each stage (lex, parse, evaluate) is a pure transformation, tested in
  isolation.
- Execution is slower than a bytecode VM or native code; acceptable for the
  goal, which is a correct, legible language implementation.
