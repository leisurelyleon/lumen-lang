//! `lumen` command-line entry point: a file runner and a REPL.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::ExitCode;

use clap::Parser as ClapParser;

use lumen_cli::cli::{Cli, Command};
use lumen_core::{Interpreter, Parser as LumenParser, interpret, lex};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { path } => run_file(&path),
        Command::Repl => run_repl(),
    }
}

fn run_file(path: &Path) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("lumen: cannot read {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
    };

    match interpret(&source) {
        Ok(output) => {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            for line in output {
                let _ = writeln!(out, "{line}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("lumen: {err}");
            ExitCode::FAILURE
        }
    }
}

/// A persistent-state REPL: each line is evaluated against the same globals.
fn run_repl() -> ExitCode {
    let mut interpreter = Interpreter::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("Lumen REPL — enter statements, Ctrl-D to exit.");
    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let line = match lines.next() {
            Some(Ok(line)) => line,
            Some(Err(_)) | None => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        match run_line(&mut interpreter, &line) {
            Ok(output) => {
                for entry in output {
                    println!("{entry}");
                }
            }
            Err(message) => eprintln!("{message}"),
        }
    }
    ExitCode::SUCCESS
}

fn run_line(interpreter: &mut Interpreter, line: &str) -> Result<Vec<String>, String> {
    let tokens = lex(line).map_err(|err| err.to_string())?;
    let statements = LumenParser::new(tokens)
        .parse()
        .map_err(|err| err.to_string())?;
    interpreter
        .run(&statements)
        .map_err(|err| err.to_string())?;
    Ok(interpreter.take_output())
}
