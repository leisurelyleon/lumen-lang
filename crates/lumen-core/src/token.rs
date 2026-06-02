//! The token: the lexer's output unit.

/// The kind of a lexical token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    Str(String),
    Identifier(String),

    // Keywords
    Var,
    Fn,
    If,
    Else,
    While,
    Return,
    Print,
    True,
    False,
    Nil,
    And,
    Or,

    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,        // =
    EqEq,      // ==
    NotEq,     // !=
    Less,      // 
    LessEq,    // <=
    Greater,   // >
    GreaterEq, // >=
    Bang,      // !

    Eof,
}

/// A token with its source position (1-based line and column).
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}
