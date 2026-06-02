//! The Lumen language: a tree-walking interpreter.
//!
//! The pipeline is three pure stages: [`lex`] turns source into tokens,
//! [`Parser`] turns tokens into an AST, and [`Interpreter`] evaluates the AST
//! against an environment chain, with functions capturing their lexical scope
//! as closures. [`interpret`] runs all three and returns the program's output.

pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod value;

pub use ast::{BinaryOp, Expr, LogicalOp, Stmt, UnaryOp};
pub use error::{LexError, LumenError, ParseError, RuntimeError};
pub use interpreter::{Interpreter, interpret};
pub use lexer::lex;
pub use parser::Parser;
pub use token::{Token, TokenKind};
pub use value::{Environment, Function, Value};
