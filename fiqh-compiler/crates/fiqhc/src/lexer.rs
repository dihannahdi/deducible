//! Hand-written lexer for the `.fiqh` DSL. Dependency-free (std only).

use crate::ast::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Ident(String),
    Str(String),
    Num(u64),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    Semi,
    Comma,
    Dot,
    EqEq,
    Ne,
    Le,
    Ge,
    Lt,
    Gt,
    Eq,
    Plus,
    Minus,
    Star,
    Slash,
    Arrow,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    fn new(s: &str) -> Self {
        Lexer {
            chars: s.chars().collect(),
            i: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.i + 1).copied()
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.i).copied();
        if let Some(ch) = c {
            self.i += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }
}

pub fn lex(src: &str) -> Result<Vec<Token>, (String, Span)> {
    let mut lx = Lexer::new(src);
    let mut out: Vec<Token> = Vec::new();

    loop {
        // Skip whitespace and comments.
        loop {
            match lx.peek() {
                Some(c) if c.is_whitespace() => {
                    lx.bump();
                }
                Some('/') if lx.peek2() == Some('/') => {
                    while let Some(c) = lx.peek() {
                        if c == '\n' {
                            break;
                        }
                        lx.bump();
                    }
                }
                Some('/') if lx.peek2() == Some('*') => {
                    let start = lx.span();
                    lx.bump();
                    lx.bump();
                    loop {
                        match lx.peek() {
                            None => return Err(("unterminated block comment".to_string(), start)),
                            Some('*') if lx.peek2() == Some('/') => {
                                lx.bump();
                                lx.bump();
                                break;
                            }
                            Some(_) => {
                                lx.bump();
                            }
                        }
                    }
                }
                _ => break,
            }
        }

        let start = lx.span();
        let c = match lx.peek() {
            None => {
                out.push(Token {
                    tok: Tok::Eof,
                    span: start,
                });
                break;
            }
            Some(c) => c,
        };

        if c.is_ascii_alphabetic() || c == '_' {
            let mut s = String::new();
            while let Some(c) = lx.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    s.push(c);
                    lx.bump();
                } else {
                    break;
                }
            }
            out.push(Token {
                tok: Tok::Ident(s),
                span: start,
            });
        } else if c.is_ascii_digit() {
            let mut s = String::new();
            while let Some(c) = lx.peek() {
                if c.is_ascii_digit() {
                    s.push(c);
                    lx.bump();
                } else if c == '_' {
                    lx.bump();
                } else {
                    break;
                }
            }
            let v = s
                .parse::<u64>()
                .map_err(|_| (format!("invalid number literal '{}'", s), start))?;
            out.push(Token {
                tok: Tok::Num(v),
                span: start,
            });
        } else if c == '"' {
            lx.bump();
            let mut s = String::new();
            loop {
                match lx.peek() {
                    None => return Err(("unterminated string literal".to_string(), start)),
                    Some('"') => {
                        lx.bump();
                        break;
                    }
                    Some('\\') => {
                        lx.bump();
                        match lx.peek() {
                            Some(e) => {
                                let m = match e {
                                    'n' => '\n',
                                    't' => '\t',
                                    '"' => '"',
                                    '\\' => '\\',
                                    other => other,
                                };
                                s.push(m);
                                lx.bump();
                            }
                            None => return Err(("bad escape in string".to_string(), start)),
                        }
                    }
                    Some(c) => {
                        s.push(c);
                        lx.bump();
                    }
                }
            }
            out.push(Token {
                tok: Tok::Str(s),
                span: start,
            });
        } else {
            lx.bump();
            let tok = match c {
                '{' => Tok::LBrace,
                '}' => Tok::RBrace,
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                ':' => Tok::Colon,
                ';' => Tok::Semi,
                ',' => Tok::Comma,
                '.' => Tok::Dot,
                '+' => Tok::Plus,
                '*' => Tok::Star,
                '/' => Tok::Slash,
                '-' => {
                    if lx.peek() == Some('>') {
                        lx.bump();
                        Tok::Arrow
                    } else {
                        Tok::Minus
                    }
                }
                '=' => {
                    if lx.peek() == Some('=') {
                        lx.bump();
                        Tok::EqEq
                    } else {
                        Tok::Eq
                    }
                }
                '!' => {
                    if lx.peek() == Some('=') {
                        lx.bump();
                        Tok::Ne
                    } else {
                        return Err(("unexpected '!'".to_string(), start));
                    }
                }
                '<' => {
                    if lx.peek() == Some('=') {
                        lx.bump();
                        Tok::Le
                    } else {
                        Tok::Lt
                    }
                }
                '>' => {
                    if lx.peek() == Some('=') {
                        lx.bump();
                        Tok::Ge
                    } else {
                        Tok::Gt
                    }
                }
                other => return Err((format!("unexpected character '{}'", other), start)),
            };
            out.push(Token { tok, span: start });
        }
    }

    Ok(out)
}
