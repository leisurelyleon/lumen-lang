//! Runtime values and the lexical environment chain.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::ast::Stmt;

/// A runtime value.
#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Str(String),
    Nil,
    Function(Rc<Function>),
}

/// A callable function value, capturing its defining environment (closure).
pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Nil => write!(f, "nil"),
            Value::Function(func) => match &func.name {
                Some(name) => write!(f, "<fn {name}>"),
                None => write!(f, "<fn>"),
            },
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// A lexical scope: a name->value map with an optional parent scope.
#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    /// A new scope nested inside `parent`.
    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Binds `name` to `value` in this scope.
    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.values.insert(name.into(), value);
    }

    /// Looks up `name`, walking up the parent chain.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    /// Assigns to an existing binding, walking up the chain. Returns whether a
    /// binding was found and updated.
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_and_get() {
        let mut env = Environment::new();
        env.define("x", Value::Number(1.0));
        assert_eq!(env.get("x"), Some(Value::Number(1.0)));
        assert_eq!(env.get("y"), None);
    }

    #[test]
    fn child_sees_parent_binding() {
        let parent = Rc::new(RefCell::new(Environment::new()));
        parent.borrow_mut().define("x", Value::Number(1.0));
        let child = Environment::with_parent(parent.clone());
        assert_eq!(child.get("x"), Some(Value::Number(1.0)));
    }

    #[test]
    fn assign_walks_to_parent() {
        let parent = Rc::new(RefCell::new(Environment::new()));
        parent.borrow_mut().define("x", Value::Number(1.0));
        let mut child = Environment::with_parent(parent.clone());
        assert!(child.assign("x", Value::Number(9.0)));
        assert_eq!(parent.borrow().get("x"), Some(Value::Number(9.0)));
    }

    #[test]
    fn assign_unknown_returns_false() {
        let mut env = Environment::new();
        assert!(!env.assign("nope", Value::Nil));
    }

    #[test]
    fn child_shadows_parent() {
        let parent = Rc::new(RefCell::new(Environment::new()));
        parent.borrow_mut().define("x", Value::Number(1.0));
        let mut child = Environment::with_parent(parent.clone());
        child.define("x", Value::Number(2.0));
        assert_eq!(child.get("x"), Some(Value::Number(2.0)));
        assert_eq!(parent.borrow().get("x"), Some(Value::Number(1.0)));
    }
}
