//! The hand-written lexer: source text -> tokens. Pure: `&str` in, tokens out.

use crate::error::LexError;
use crate::token::{Token, TokenKind};

/// Tokenizes Lumen source into a token stream terminated by `Eof`.
pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;

    while i < chars.len() {
        let c = chars[i];
        let start_col = col;
        match c {
            ' ' | '\t' | '\r' => {
                i += 1;
                col += 1;
            }
            '\n' => {
                i += 1;
                line += 1;
                col = 1;
            }
            // Line comment: skip to end of line (the newline arm handles the break).
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                    col += 1;
                }
            }
            '(' => push_single(
                &mut tokens,
                TokenKind::LParen,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            ')' => push_single(
                &mut tokens,
                TokenKind::RParen,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '{' => push_single(
                &mut tokens,
                TokenKind::LBrace,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '}' => push_single(
                &mut tokens,
                TokenKind::RBrace,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            ';' => push_single(
                &mut tokens,
                TokenKind::Semicolon,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            ',' => push_single(
                &mut tokens,
                TokenKind::Comma,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '+' => push_single(
                &mut tokens,
                TokenKind::Plus,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '-' => push_single(
                &mut tokens,
                TokenKind::Minus,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '*' => push_single(
                &mut tokens,
                TokenKind::Star,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '/' => push_single(
                &mut tokens,
                TokenKind::Slash,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '%' => push_single(
                &mut tokens,
                TokenKind::Percent,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '=' => push_maybe_eq(
                &chars,
                &mut tokens,
                TokenKind::EqEq,
                TokenKind::Eq,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '!' => push_maybe_eq(
                &chars,
                &mut tokens,
                TokenKind::NotEq,
                TokenKind::Bang,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '<' => push_maybe_eq(
                &chars,
                &mut tokens,
                TokenKind::LessEq,
                TokenKind::Less,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '>' => push_maybe_eq(
                &chars,
                &mut tokens,
                TokenKind::GreaterEq,
                TokenKind::Greater,
                line,
                start_col,
                &mut i,
                &mut col,
            ),
            '"' => {
                let str_col = start_col;
                i += 1;
                col += 1;
                let mut text = String::new();
                loop {
                    if i >= chars.len() || chars[i] == '\n' {
                        return Err(LexError {
                            message: "unterminated string".into(),
                            line,
                            col: str_col,
                        });
                    }
                    let ch = chars[i];
                    i += 1;
                    col += 1;
                    if ch == '"' {
                        break;
                    }
                    text.push(ch);
                }
                tokens.push(Token {
                    kind: TokenKind::Str(text),
                    line,
                    col: str_col,
                });
            }
            c if c.is_ascii_digit() => {
                let mut num = String::new();
                let mut seen_dot = false;
                while i < chars.len() {
                    let ch = chars[i];
                    if ch.is_ascii_digit() {
                        num.push(ch);
                        i += 1;
                        col += 1;
                    } else if ch == '.'
                        && !seen_dot
                        && i + 1 < chars.len()
                        && chars[i + 1].is_ascii_digit()
                    {
                        seen_dot = true;
                        num.push(ch);
                        i += 1;
                        col += 1;
                    } else {
                        break;
                    }
                }
                let value: f64 = num.parse().map_err(|_| LexError {
                    message: format!("invalid number '{num}'"),
                    line,
                    col: start_col,
                })?;
                tokens.push(Token {
                    kind: TokenKind::Number(value),
                    line,
                    col: start_col,
                });
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    ident.push(chars[i]);
                    i += 1;
                    col += 1;
                }
                tokens.push(Token {
                    kind: keyword_or_identifier(&ident),
                    line,
                    col: start_col,
                });
            }
            other => {
                return Err(LexError {
                    message: format!("unexpected character '{other}'"),
                    line,
                    col: start_col,
                });
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        line,
        col,
    });
    Ok(tokens)
}

fn push_single(
    tokens: &mut Vec<Token>,
    kind: TokenKind,
    line: usize,
    col: usize,
    i: &mut usize,
    col_cursor: &mut usize,
) {
    tokens.push(Token { kind, line, col });
    *i += 1;
    *col_cursor += 1;
}

#[allow(clippy::too_many_arguments)]
fn push_maybe_eq(
    chars: &[char],
    tokens: &mut Vec<Token>,
    two_char: TokenKind,
    one_char: TokenKind,
    line: usize,
    col: usize,
    i: &mut usize,
    col_cursor: &mut usize,
) {
    if *i + 1 < chars.len() && chars[*i + 1] == '=' {
        tokens.push(Token {
            kind: two_char,
            line,
            col,
        });
        *i += 2;
        *col_cursor += 2;
    } else {
        tokens.push(Token {
            kind: one_char,
            line,
            col,
        });
        *i += 1;
        *col_cursor += 1;
    }
}

fn keyword_or_identifier(ident: &str) -> TokenKind {
    match ident {
        "var" => TokenKind::Var,
        "fn" => TokenKind::Fn,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "return" => TokenKind::Return,
        "print" => TokenKind::Print,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "nil" => TokenKind::Nil,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,
        _ => TokenKind::Identifier(ident.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        lex(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn distinguishes_keywords_from_identifiers() {
        assert_eq!(
            kinds("var foo"),
            vec![
                TokenKind::Var,
                TokenKind::Identifier("foo".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn lexes_numbers_including_decimals() {
        assert_eq!(kinds("42"), vec![TokenKind::Number(42.0), TokenKind::Eof]);
        assert_eq!(kinds("2.5"), vec![TokenKind::Number(2.5), TokenKind::Eof]);
    }

    #[test]
    fn lexes_strings() {
        assert_eq!(
            kinds("\"hi\""),
            vec![TokenKind::Str("hi".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn skips_line_comments() {
        assert_eq!(
            kinds("1 // ignored\n2"),
            vec![
                TokenKind::Number(1.0),
                TokenKind::Number(2.0),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn lexes_two_char_operators() {
        assert_eq!(
            kinds("== != <= >="),
            vec![
                TokenKind::EqEq,
                TokenKind::NotEq,
                TokenKind::LessEq,
                TokenKind::GreaterEq,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn lexes_arithmetic_operators() {
        assert_eq!(
            kinds("+ - * / %"),
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn unterminated_string_is_error() {
        assert!(lex("\"abc").is_err());
    }
}
