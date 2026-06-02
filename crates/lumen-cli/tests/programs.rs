//! End-to-end tests: run complete Lumen programs (including the shipped example
//! files) through the full pipeline and assert their exact output. This proves
//! lexer, parser, and interpreter compose correctly on real programs.

use lumen_core::interpret;

fn run(src: &str) -> Vec<String> {
    interpret(src).expect("program should run without error")
}

#[test]
fn fibonacci_sequence_program() {
    // The logic of examples/fib.lum: print fib(0)..fib(9).
    let src = "
        fn fib(n) {
            if (n < 2) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        var i = 0;
        while (i < 10) {
            print fib(i);
            i = i + 1;
        }
    ";
    assert_eq!(
        run(src),
        vec!["0", "1", "1", "2", "3", "5", "8", "13", "21", "34"]
    );
}

#[test]
fn counter_closures_program() {
    // The logic of examples/closures.lum: two independent counters.
    let src = "
        fn make_counter() {
            var count = 0;
            fn increment() { count = count + 1; return count; }
            return increment;
        }
        var counter = make_counter();
        print counter();
        print counter();
        print counter();
        var other = make_counter();
        print other();
    ";
    assert_eq!(run(src), vec!["1", "2", "3", "1"]);
}

#[test]
fn fizzbuzz_program() {
    // The logic of examples/fizzbuzz.lum: FizzBuzz 1..15.
    let src = "
        var n = 1;
        while (n <= 15) {
            if (n % 3 == 0 and n % 5 == 0) { print \"FizzBuzz\"; }
            else if (n % 3 == 0) { print \"Fizz\"; }
            else if (n % 5 == 0) { print \"Buzz\"; }
            else { print n; }
            n = n + 1;
        }
    ";
    assert_eq!(
        run(src),
        vec![
            "1", "2", "Fizz", "4", "Buzz", "Fizz", "7", "8", "Fizz", "Buzz", "11", "Fizz", "13",
            "14", "FizzBuzz"
        ]
    );
}

#[test]
fn shipped_example_files_run() {
    // Read and execute the actual example files from disk, asserting they run
    // and produce output — keeping the shipped examples honest.
    for name in ["fib.lum", "closures.lum", "fizzbuzz.lum"] {
        let path = format!("{}/../../examples/{name}", env!("CARGO_MANIFEST_DIR"));
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("should read example {name}: {e}"));
        let output =
            interpret(&source).unwrap_or_else(|e| panic!("example {name} should run: {e}"));
        assert!(!output.is_empty(), "example {name} should produce output");
    }
}

#[test]
fn nested_scopes_resolve_correctly() {
    // A block shadows then restores an outer variable.
    let src = "
        var x = 1;
        { var x = 2; print x; }
        print x;
    ";
    assert_eq!(run(src), vec!["2", "1"]);
}
