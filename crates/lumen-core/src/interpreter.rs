//! The tree-walking evaluator, and the top-level `interpret` convenience.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expr, LogicalOp, Stmt, UnaryOp};
use crate::error::{LumenError, RuntimeError};
use crate::lexer::lex;
use crate::parser::Parser;
use crate::value::{Environment, Function, Value};

/// Internal control-flow signal: an error, or a `return` unwinding to a call.
enum Interrupt {
    Return(Value),
    Error(RuntimeError),
}

impl From<RuntimeError> for Interrupt {
    fn from(error: RuntimeError) -> Self {
        Interrupt::Error(error)
    }
}

/// The tree-walking interpreter. Collects `print` output into a buffer so the
/// evaluator stays pure (no direct stdout) and is deterministically testable.
#[derive(Default)]
pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    output: Vec<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    /// The output produced so far (one entry per `print`).
    pub fn output(&self) -> &[String] {
        &self.output
    }

    /// Drains the accumulated output (used by the REPL between lines).
    pub fn take_output(&mut self) -> Vec<String> {
        std::mem::take(&mut self.output)
    }

    /// Consumes the interpreter, returning all collected output.
    pub fn into_output(self) -> Vec<String> {
        self.output
    }

    /// Executes a program. A top-level `return` simply stops execution.
    pub fn run(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        let env = self.globals.clone();
        for stmt in statements {
            match self.execute(stmt, &env) {
                Ok(()) => {}
                Err(Interrupt::Return(_)) => return Ok(()),
                Err(Interrupt::Error(error)) => return Err(error),
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt, env: &Rc<RefCell<Environment>>) -> Result<(), Interrupt> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr, env)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr, env)?;
                self.output.push(value.to_string());
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr, env)?,
                    None => Value::Nil,
                };
                env.borrow_mut().define(name.clone(), value);
                Ok(())
            }
            Stmt::Block(statements) => {
                let block_env = Rc::new(RefCell::new(Environment::with_parent(env.clone())));
                for inner in statements {
                    self.execute(inner, &block_env)?;
                }
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&self.evaluate(condition, env)?) {
                    self.execute(then_branch, env)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch, env)
                } else {
                    Ok(())
                }
            }
            Stmt::While { condition, body } => {
                while is_truthy(&self.evaluate(condition, env)?) {
                    self.execute(body, env)?;
                }
                Ok(())
            }
            Stmt::Function { name, params, body } => {
                let function = Function {
                    name: Some(name.clone()),
                    params: params.clone(),
                    body: body.clone(),
                    closure: env.clone(),
                };
                env.borrow_mut()
                    .define(name.clone(), Value::Function(Rc::new(function)));
                Ok(())
            }
            Stmt::Return(expr) => {
                let value = match expr {
                    Some(expr) => self.evaluate(expr, env)?,
                    None => Value::Nil,
                };
                Err(Interrupt::Return(value))
            }
        }
    }

    fn evaluate(
        &mut self,
        expr: &Expr,
        env: &Rc<RefCell<Environment>>,
    ) -> Result<Value, Interrupt> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Nil => Ok(Value::Nil),
            Expr::Variable(name) => match env.borrow().get(name) {
                Some(value) => Ok(value),
                None => Err(RuntimeError::new(format!("undefined variable '{name}'")).into()),
            },
            Expr::Assign { name, value } => {
                let evaluated = self.evaluate(value, env)?;
                if env.borrow_mut().assign(name, evaluated.clone()) {
                    Ok(evaluated)
                } else {
                    Err(RuntimeError::new(format!("undefined variable '{name}'")).into())
                }
            }
            Expr::Unary { op, right } => {
                let value = self.evaluate(right, env)?;
                match op {
                    UnaryOp::Negate => match value {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(RuntimeError::new("operand of '-' must be a number").into()),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!is_truthy(&value))),
                }
            }
            Expr::Logical { left, op, right } => {
                let left_value = self.evaluate(left, env)?;
                match op {
                    // Short-circuit: return the left value without evaluating the right.
                    LogicalOp::Or if is_truthy(&left_value) => Ok(left_value),
                    LogicalOp::And if !is_truthy(&left_value) => Ok(left_value),
                    _ => self.evaluate(right, env),
                }
            }
            Expr::Binary { left, op, right } => {
                let left_value = self.evaluate(left, env)?;
                let right_value = self.evaluate(right, env)?;
                eval_binary(op, left_value, right_value).map_err(Interrupt::from)
            }
            Expr::Call { callee, args } => {
                let callee_value = self.evaluate(callee, env)?;
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(self.evaluate(arg, env)?);
                }
                self.call_function(callee_value, arg_values)
            }
        }
    }

    fn call_function(&mut self, callee: Value, args: Vec<Value>) -> Result<Value, Interrupt> {
        let function = match callee {
            Value::Function(function) => function,
            other => return Err(RuntimeError::new(format!("'{other}' is not callable")).into()),
        };

        if args.len() != function.params.len() {
            return Err(RuntimeError::new(format!(
                "expected {} argument(s) but got {}",
                function.params.len(),
                args.len()
            ))
            .into());
        }

        // A fresh scope whose parent is the function's CLOSURE — this is what
        // makes scoping lexical rather than dynamic.
        let call_env = Rc::new(RefCell::new(Environment::with_parent(
            function.closure.clone(),
        )));

        // Deliberate scoped block: bind the parameters, then RELEASE the mutable
        // borrow before executing the body (which borrows `call_env` again).
        // Holding both at once would panic at runtime — this avoids it.
        {
            let mut env_mut = call_env.borrow_mut();
            for (param, arg) in function.params.iter().zip(args) {
                env_mut.define(param.clone(), arg);
            }
        }

        for stmt in &function.body {
            match self.execute(stmt, &call_env) {
                Ok(()) => {}
                Err(Interrupt::Return(value)) => return Ok(value),
                Err(Interrupt::Error(error)) => return Err(Interrupt::Error(error)),
            }
        }
        Ok(Value::Nil)
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(b) => *b,
        _ => true,
    }
}

fn eval_binary(op: &BinaryOp, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match op {
        BinaryOp::Add => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
            _ => Err(RuntimeError::new(
                "operands of '+' must be two numbers or two strings",
            )),
        },
        BinaryOp::Sub => numeric(left, right, |a, b| a - b, "-"),
        BinaryOp::Mul => numeric(left, right, |a, b| a * b, "*"),
        BinaryOp::Div => match (left, right) {
            (Value::Number(_), Value::Number(0.0)) => Err(RuntimeError::new("division by zero")),
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
            _ => Err(RuntimeError::new("operands of '/' must be numbers")),
        },
        BinaryOp::Mod => match (left, right) {
            (Value::Number(_), Value::Number(0.0)) => Err(RuntimeError::new("modulo by zero")),
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
            _ => Err(RuntimeError::new("operands of '%' must be numbers")),
        },
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::NotEq => Ok(Value::Bool(left != right)),
        BinaryOp::Less => compare(left, right, std::cmp::Ordering::is_lt, "<"),
        BinaryOp::LessEq => compare(left, right, std::cmp::Ordering::is_le, "<="),
        BinaryOp::Greater => compare(left, right, std::cmp::Ordering::is_gt, ">"),
        BinaryOp::GreaterEq => compare(left, right, std::cmp::Ordering::is_ge, ">="),
    }
}

fn numeric(
    left: Value,
    right: Value,
    f: impl Fn(f64, f64) -> f64,
    sym: &str,
) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
        _ => Err(RuntimeError::new(format!(
            "operands of '{sym}' must be numbers"
        ))),
    }
}

fn compare(
    left: Value,
    right: Value,
    f: impl Fn(std::cmp::Ordering) -> bool,
    sym: &str,
) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => match a.partial_cmp(&b) {
            Some(ordering) => Ok(Value::Bool(f(ordering))),
            None => Err(RuntimeError::new(format!(
                "cannot compare operands with '{sym}'"
            ))),
        },
        _ => Err(RuntimeError::new(format!(
            "operands of '{sym}' must be numbers"
        ))),
    }
}

/// Lexes, parses, and runs `source`, returning the collected `print` output.
pub fn interpret(source: &str) -> Result<Vec<String>, LumenError> {
    let tokens = lex(source)?;
    let statements = Parser::new(tokens).parse()?;
    let mut interpreter = Interpreter::new();
    interpreter.run(&statements)?;
    Ok(interpreter.into_output())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<String> {
        interpret(src).expect("program should run without error")
    }

    #[test]
    fn arithmetic_respects_precedence() {
        assert_eq!(run("print 1 + 2 * 3;"), vec!["7"]);
    }

    #[test]
    fn grouping_changes_result() {
        assert_eq!(run("print (1 + 2) * 3;"), vec!["9"]);
    }

    #[test]
    fn string_concatenation() {
        assert_eq!(run("print \"foo\" + \"bar\";"), vec!["foobar"]);
    }

    #[test]
    fn variables_and_assignment() {
        assert_eq!(run("var x = 10; x = x + 5; print x;"), vec!["15"]);
    }

    #[test]
    fn if_else_branches() {
        assert_eq!(
            run("if (1 < 2) { print \"yes\"; } else { print \"no\"; }"),
            vec!["yes"]
        );
    }

    #[test]
    fn while_loop_accumulates() {
        let src = "var i = 0; var s = 0; while (i < 5) { s = s + i; i = i + 1; } print s;";
        assert_eq!(run(src), vec!["10"]);
    }

    #[test]
    fn function_call_and_return() {
        assert_eq!(
            run("fn add(a, b) { return a + b; } print add(3, 4);"),
            vec!["7"]
        );
    }

    #[test]
    fn recursion_computes_fibonacci() {
        let src = "fn fib(n) { if (n < 2) { return n; } return fib(n - 1) + fib(n - 2); } \
                   print fib(10);";
        assert_eq!(run(src), vec!["55"]);
    }

    #[test]
    fn closure_captures_mutable_state() {
        // The crown jewel: a returned closure increments a variable it captured.
        let src = "fn make() { var count = 0; fn next() { count = count + 1; return count; } \
                   return next; } var c = make(); print c(); print c(); print c();";
        assert_eq!(run(src), vec!["1", "2", "3"]);
    }

    #[test]
    fn closures_have_independent_state() {
        let src = "fn make() { var count = 0; fn next() { count = count + 1; return count; } \
                   return next; } var a = make(); var b = make(); \
                   print a(); print a(); print b();";
        assert_eq!(run(src), vec!["1", "2", "1"]);
    }

    #[test]
    fn logical_and_short_circuits() {
        // `bump` must NOT run, since `false and _` short-circuits.
        let src = "var x = 0; fn bump() { x = x + 1; return true; } false and bump(); print x;";
        assert_eq!(run(src), vec!["0"]);
    }

    #[test]
    fn modulo_operator() {
        assert_eq!(run("print 17 % 5;"), vec!["2"]);
    }

    #[test]
    fn comparison_yields_bool() {
        assert_eq!(run("print 3 <= 3;"), vec!["true"]);
    }

    #[test]
    fn undefined_variable_is_runtime_error() {
        assert!(interpret("print y;").is_err());
    }

    #[test]
    fn division_by_zero_is_runtime_error() {
        assert!(interpret("print 1 / 0;").is_err());
    }
}
