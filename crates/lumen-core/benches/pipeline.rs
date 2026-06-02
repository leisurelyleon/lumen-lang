//! Pipeline throughput: how fast each stage processes a representative program.
//!
//! Measures lexing, parsing, and full interpretation separately, so the cost of
//! each stage is visible. The workload is a recursive Fibonacci, which stresses
//! function calls, environment creation, and the evaluator's hot path.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use lumen_core::{Parser, interpret, lex};

const PROGRAM: &str = "
fn fib(n) {
    if (n < 2) { return n; }
    return fib(n - 1) + fib(n - 2);
}
var i = 0;
while (i < 15) {
    fib(i);
    i = i + 1;
}
";

fn bench_lex(c: &mut Criterion) {
    c.bench_function("lex", |b| {
        b.iter(|| {
            let tokens = lex(black_box(PROGRAM)).unwrap();
            black_box(tokens.len())
        });
    });
}

fn bench_parse(c: &mut Criterion) {
    let tokens = lex(PROGRAM).unwrap();
    c.bench_function("parse", |b| {
        b.iter(|| {
            let statements = Parser::new(black_box(tokens.clone())).parse().unwrap();
            black_box(statements.len())
        });
    });
}

fn bench_interpret(c: &mut Criterion) {
    c.bench_function("interpret", |b| {
        b.iter(|| {
            let output = interpret(black_box(PROGRAM)).unwrap();
            black_box(output.len())
        });
    });
}

criterion_group!(benches, bench_lex, bench_parse, bench_interpret);
criterion_main!(benches);
