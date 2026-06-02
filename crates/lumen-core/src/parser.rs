//! Recursive-descent parser: tokens -> AST, with correct operator precedence.
//!
//! Precedence, lowest to highest: assignment, `or`, `and`, equality,
//! comparison, term (`+ -`), factor (`* / %`), unary, call, primary. Each level
//! is its own method, so `1 + 2 * 3` parses as `1 + (2 * 3)`.

use crate::ast::{BinaryOp, Expr, LogicalOp, Stmt, UnaryOp};
use crate::error::ParseError;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parses the whole token stream into a list of statements.
    pub fn parse(mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    // --- Token cursor helpers ---

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        if !self.is_at_end() {
            self.pos += 1;
        }
        token
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn match_kind(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        if let TokenKind::Identifier(name) = &self.peek().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error(message))
        }
    }

    fn error(&self, message: &str) -> ParseError {
        let token = self.peek();
        ParseError { message: message.to_string(), line: token.line, col: token.col }
    }

    // --- Declarations & statements ---

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_kind(&TokenKind::Var) {
            self.var_declaration()
        } else if self.match_kind(&TokenKind::Fn) {
            self.function()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("expected variable name")?;
        let initializer =
            if self.match_kind(&TokenKind::Eq) { Some(self.expression()?) } else { None };
        self.consume(&TokenKind::Semicolon, "expected ';' after variable declaration")?;
        Ok(Stmt::Var { name, initializer })
    }

    fn function(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("expected function name")?;
        self.consume(&TokenKind::LParen, "expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                params.push(self.consume_identifier("expected parameter name")?);
                if !self.match_kind(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(&TokenKind::RParen, "expected ')' after parameters")?;
        self.consume(&TokenKind::LBrace, "expected '{' before function body")?;
        let body = self.block()?;
        Ok(Stmt::Function { name, params, body })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&TokenKind::RBrace, "expected '}' after block")?;
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_kind(&TokenKind::Print) {
            let value = self.expression()?;
            self.consume(&TokenKind::Semicolon, "expected ';' after value")?;
            Ok(Stmt::Print(value))
        } else if self.match_kind(&TokenKind::LBrace) {
            Ok(Stmt::Block(self.block()?))
        } else if self.match_kind(&TokenKind::If) {
            self.if_statement()
        } else if self.match_kind(&TokenKind::While) {
            self.while_statement()
        } else if self.match_kind(&TokenKind::Return) {
            self.return_statement()
        } else {
            let expr = self.expression()?;
            self.consume(&TokenKind::Semicolon, "expected ';' after expression")?;
            Ok(Stmt::Expression(expr))
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&TokenKind::LParen, "expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RParen, "expected ')' after if condition")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch =
            if self.match_kind(&TokenKind::Else) { Some(Box::new(self.statement()?)) } else { None };
        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&TokenKind::LParen, "expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RParen, "expected ')' after while condition")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let value =
            if self.check(&TokenKind::Semicolon) { None } else { Some(self.expression()?) };
        self.consume(&TokenKind::Semicolon, "expected ';' after return value")?;
        Ok(Stmt::Return(value))
    }

    // --- Expressions (precedence climbing) ---

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;
        if self.match_kind(&TokenKind::Eq) {
            let value = self.assignment()?;
            if let Expr::Variable(name) = expr {
                Ok(Expr::Assign { name, value: Box::new(value) })
            } else {
                Err(ParseError {
                    message: "invalid assignment target".into(),
                    line: self.previous().line,
                    col: self.previous().col,
                })
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;
        while self.match_kind(&TokenKind::Or) {
            let right = self.and()?;
            expr = Expr::Logical { left: Box::new(expr), op: LogicalOp::Or, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.match_kind(&TokenKind::And) {
            let right = self.equality()?;
            expr =
                Expr::Logical { left: Box::new(expr), op: LogicalOp::And, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        loop {
            let op = if self.match_kind(&TokenKind::EqEq) {
                BinaryOp::Eq
            } else if self.match_kind(&TokenKind::NotEq) {
                BinaryOp::NotEq
            } else {
                break;
            };
            let right = self.comparison()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        loop {
            let op = if self.match_kind(&TokenKind::Less) {
                BinaryOp::Less
            } else if self.match_kind(&TokenKind::LessEq) {
                BinaryOp::LessEq
            } else if self.match_kind(&TokenKind::Greater) {
                BinaryOp::Greater
            } else if self.match_kind(&TokenKind::GreaterEq) {
                BinaryOp::GreaterEq
            } else {
                break;
            };
            let right = self.term()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        loop {
            let op = if self.match_kind(&TokenKind::Plus) {
                BinaryOp::Add
            } else if self.match_kind(&TokenKind::Minus) {
                BinaryOp::Sub
            } else {
                break;
            };
            let right = self.factor()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        loop {
            let op = if self.match_kind(&TokenKind::Star) {
                BinaryOp::Mul
            } else if self.match_kind(&TokenKind::Slash) {
                BinaryOp::Div
            } else if self.match_kind(&TokenKind::Percent) {
                BinaryOp::Mod
            } else {
                break;
            };
            let right = self.unary()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_kind(&TokenKind::Bang) {
            let right = self.unary()?;
            Ok(Expr::Unary { op: UnaryOp::Not, right: Box::new(right) })
        } else if self.match_kind(&TokenKind::Minus) {
            let right = self.unary()?;
            Ok(Expr::Unary { op: UnaryOp::Negate, right: Box::new(right) })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        while self.match_kind(&TokenKind::LParen) {
            expr = self.finish_call(expr)?;
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                args.push(self.expression()?);
                if !self.match_kind(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(&TokenKind::RParen, "expected ')' after arguments")?;
        Ok(Expr::Call { callee: Box::new(callee), args })
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();
        match token.kind {
            TokenKind::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr::Str(s))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            TokenKind::Nil => {
                self.advance();
                Ok(Expr::Nil)
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::Variable(name))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(&TokenKind::RParen, "expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(ParseError {
                message: "unexpected token in expression".to_string(),
                line: token.line,
                col: token.col,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_expr(src: &str) -> Expr {
        let tokens = lex(src).unwrap();
        let statements = Parser::new(tokens).parse().unwrap();
        match statements.into_iter().next().unwrap() {
            Stmt::Expression(expr) => expr,
            other => panic!("expected an expression statement, got {other:?}"),
        }
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        // 1 + 2 * 3 -> Add(1, Mul(2, 3))
        let expr = parse_expr("1 + 2 * 3;");
        match expr {
            Expr::Binary { op: BinaryOp::Add, right, .. } => {
                assert!(matches!(*right, Expr::Binary { op: BinaryOp::Mul, .. }));
            }
            other => panic!("expected addition at the root, got {other:?}"),
        }
    }

    #[test]
    fn grouping_overrides_precedence() {
        // (1 + 2) * 3 -> Mul(Add(1, 2), 3)
        let expr = parse_expr("(1 + 2) * 3;");
        match expr {
            Expr::Binary { op: BinaryOp::Mul, left, .. } => {
                assert!(matches!(*left, Expr::Binary { op: BinaryOp::Add, .. }));
            }
            other => panic!("expected multiplication at the root, got {other:?}"),
        }
    }

    #[test]
    fn parses_comparison() {
        let expr = parse_expr("1 < 2;");
        assert!(matches!(expr, Expr::Binary { op: BinaryOp::Less, .. }));
    }

    #[test]
    fn parses_var_declaration() {
        let tokens = lex("var x = 5;").unwrap();
        let statements = Parser::new(tokens).parse().unwrap();
        assert!(matches!(statements[0], Stmt::Var { .. }));
    }

    #[test]
    fn parses_function_declaration() {
        let tokens = lex("fn add(a, b) { return a + b; }").unwrap();
        let statements = Parser::new(tokens).parse().unwrap();
        assert!(matches!(statements[0], Stmt::Function { .. }));
    }

    #[test]
    fn missing_semicolon_is_error() {
        let tokens = lex("var x = 1").unwrap();
        assert!(Parser::new(tokens).parse().is_err());
    }
}
