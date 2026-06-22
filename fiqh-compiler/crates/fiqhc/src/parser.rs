//! Recursive-descent parser for the `.fiqh` DSL. Dependency-free (std only).

use crate::ast::*;
use crate::lexer::{Tok, Token};

pub struct ParseErr {
    pub msg: String,
    pub span: Span,
}

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

pub fn parse(toks: Vec<Token>) -> Result<Spec, ParseErr> {
    let mut p = Parser::new(toks);
    p.parse_spec()
}

impl Parser {
    fn new(toks: Vec<Token>) -> Self {
        Parser { toks, pos: 0 }
    }

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn span(&self) -> Span {
        self.toks[self.pos].span
    }

    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn err<T>(&self, msg: impl Into<String>) -> Result<T, ParseErr> {
        Err(ParseErr {
            msg: msg.into(),
            span: self.span(),
        })
    }

    fn expect(&mut self, t: &Tok) -> Result<(), ParseErr> {
        if self.peek() == t {
            self.bump();
            Ok(())
        } else {
            self.err(format!("expected {:?}, found {:?}", t, self.peek()))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseErr> {
        match self.peek().clone() {
            Tok::Ident(s) => {
                self.bump();
                Ok(s)
            }
            other => self.err(format!("expected identifier, found {:?}", other)),
        }
    }

    fn expect_num(&mut self) -> Result<u64, ParseErr> {
        match self.peek().clone() {
            Tok::Num(n) => {
                self.bump();
                Ok(n)
            }
            other => self.err(format!("expected number, found {:?}", other)),
        }
    }

    fn is_ident(&self, kw: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s == kw)
    }

    fn parse_spec(&mut self) -> Result<Spec, ParseErr> {
        let span = self.span();
        if !self.is_ident("instrument") {
            return self.err("expected 'instrument' to begin a specification");
        }
        self.bump();
        let name = self.expect_ident()?;
        self.expect(&Tok::Colon)?;
        let class = self.expect_ident()?;
        self.expect(&Tok::LBrace)?;
        let mut sections = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unexpected end of file inside instrument body");
            }
            sections.push(self.parse_section()?);
        }
        self.expect(&Tok::RBrace)?;
        Ok(Spec {
            name,
            class,
            sections,
            span,
        })
    }

    fn parse_section(&mut self) -> Result<Section, ParseErr> {
        let kw = match self.peek().clone() {
            Tok::Ident(s) => s,
            other => return self.err(format!("expected a section keyword, found {:?}", other)),
        };
        match kw.as_str() {
            "meta" => {
                self.bump();
                self.expect(&Tok::LBrace)?;
                let kvs = self.parse_kv_list()?;
                Ok(Section::Meta(kvs))
            }
            "parties" => self.parse_parties(),
            "capital" => self.parse_capital(),
            "returns" => self.parse_returns(),
            "risk" => {
                self.bump();
                self.expect(&Tok::LBrace)?;
                let kvs = self.parse_kv_list()?;
                Ok(Section::Risk(kvs))
            }
            "oracle" => {
                self.bump();
                self.expect(&Tok::LBrace)?;
                let kvs = self.parse_kv_list()?;
                Ok(Section::Oracle(kvs))
            }
            "invariant" => self.parse_invariant(),
            "rescission" => self.parse_rescission(),
            "lifecycle" => self.parse_lifecycle(),
            other => self.err(format!("unknown section '{}'", other)),
        }
    }

    /// Parse `key: expr;` entries until (and consuming) the closing `}`.
    fn parse_kv_list(&mut self) -> Result<Vec<Kv>, ParseErr> {
        let mut kvs = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unexpected end of file inside block");
            }
            let span = self.span();
            let key = self.expect_ident()?;
            self.expect(&Tok::Colon)?;
            let val = self.parse_expr()?;
            self.expect(&Tok::Semi)?;
            kvs.push(Kv { key, val, span });
        }
        self.expect(&Tok::RBrace)?;
        Ok(kvs)
    }

    fn parse_parties(&mut self) -> Result<Section, ParseErr> {
        self.bump();
        self.expect(&Tok::LBrace)?;
        let mut v = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unexpected end of file inside parties");
            }
            let span = self.span();
            let name = self.expect_ident()?;
            self.expect(&Tok::Colon)?;
            let role = self.expect_ident()?;
            let mut flags = Vec::new();
            while let Tok::Ident(_) = self.peek() {
                flags.push(self.expect_ident()?);
            }
            self.expect(&Tok::Semi)?;
            v.push(Party {
                name,
                role,
                flags,
                span,
            });
        }
        self.expect(&Tok::RBrace)?;
        Ok(Section::Parties(v))
    }

    fn parse_capital(&mut self) -> Result<Section, ParseErr> {
        self.bump();
        self.expect(&Tok::LBrace)?;
        let mut v = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unexpected end of file inside capital");
            }
            let span = self.span();
            if self.is_ident("require") {
                self.bump();
                let expr = self.parse_expr()?;
                self.expect(&Tok::Semi)?;
                v.push(CapItem::Require { expr, span });
            } else {
                let party = self.expect_ident()?;
                self.expect(&Tok::Colon)?;
                let bps = self.expect_num()?;
                let unit = self.expect_ident()?;
                if unit != "bps" {
                    return self.err(format!("capital must be denominated in bps, found '{}'", unit));
                }
                self.expect(&Tok::Semi)?;
                v.push(CapItem::Assign { party, bps, span });
            }
        }
        self.expect(&Tok::RBrace)?;
        Ok(Section::Capital(v))
    }

    fn parse_returns(&mut self) -> Result<Section, ParseErr> {
        self.bump();
        self.expect(&Tok::LBrace)?;
        let mut v = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unexpected end of file inside returns");
            }
            let span = self.span();
            let kind = self.expect_ident()?;
            self.expect(&Tok::LBrace)?;
            let kvs = self.parse_kv_list()?;
            v.push(RetBlock { kind, kvs, span });
        }
        self.expect(&Tok::RBrace)?;
        Ok(Section::Returns(v))
    }

    fn parse_invariant(&mut self) -> Result<Section, ParseErr> {
        self.bump();
        let span = self.span();
        let name = self.expect_ident()?;
        self.expect(&Tok::LBrace)?;
        let expr = self.parse_expr()?;
        self.expect(&Tok::RBrace)?;
        Ok(Section::Invariant(Invariant { name, expr, span }))
    }

    fn parse_rescission(&mut self) -> Result<Section, ParseErr> {
        self.bump();
        self.expect(&Tok::LBrace)?;
        let mut v = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unexpected end of file inside rescission");
            }
            let span = self.span();
            let kind = self.expect_ident()?;
            self.expect(&Tok::LBrace)?;
            let kvs = self.parse_kv_list()?;
            v.push(RescBlock { kind, kvs, span });
        }
        self.expect(&Tok::RBrace)?;
        Ok(Section::Rescission(v))
    }

    fn parse_lifecycle(&mut self) -> Result<Section, ParseErr> {
        self.bump();
        self.expect(&Tok::LBrace)?;
        let mut v = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unexpected end of file inside lifecycle");
            }
            let span = self.span();
            let name = self.expect_ident()?;
            let mut arg = None;
            if *self.peek() == Tok::LParen {
                self.bump();
                arg = Some(self.expect_ident()?);
                self.expect(&Tok::RParen)?;
            }
            self.expect(&Tok::Semi)?;
            v.push(Step { name, arg, span });
        }
        self.expect(&Tok::RBrace)?;
        Ok(Section::Lifecycle(v))
    }

    // --- expressions ---

    fn parse_expr(&mut self) -> Result<Expr, ParseErr> {
        self.parse_arrow()
    }

    fn parse_arrow(&mut self) -> Result<Expr, ParseErr> {
        let mut left = self.parse_cmp()?;
        while *self.peek() == Tok::Arrow {
            self.bump();
            let r = self.parse_cmp()?;
            left = Expr::Bin(Box::new(left), BinOp::Arrow, Box::new(r));
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseErr> {
        let left = self.parse_add()?;
        let op = match self.peek() {
            Tok::EqEq => Some(BinOp::Eq),
            Tok::Ne => Some(BinOp::Ne),
            Tok::Le => Some(BinOp::Le),
            Tok::Ge => Some(BinOp::Ge),
            Tok::Lt => Some(BinOp::Lt),
            Tok::Gt => Some(BinOp::Gt),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let r = self.parse_add()?;
            Ok(Expr::Bin(Box::new(left), op, Box::new(r)))
        } else {
            Ok(left)
        }
    }

    fn parse_add(&mut self) -> Result<Expr, ParseErr> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => Some(BinOp::Add),
                Tok::Minus => Some(BinOp::Sub),
                _ => None,
            };
            match op {
                Some(op) => {
                    self.bump();
                    let r = self.parse_mul()?;
                    left = Expr::Bin(Box::new(left), op, Box::new(r));
                }
                None => break,
            }
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseErr> {
        let mut left = self.parse_primary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => Some(BinOp::Mul),
                Tok::Slash => Some(BinOp::Div),
                _ => None,
            };
            match op {
                Some(op) => {
                    self.bump();
                    let r = self.parse_primary()?;
                    left = Expr::Bin(Box::new(left), op, Box::new(r));
                }
                None => break,
            }
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseErr> {
        match self.peek().clone() {
            Tok::Str(s) => {
                self.bump();
                Ok(Expr::Str(s))
            }
            Tok::Num(n) => {
                self.bump();
                let unit = if let Tok::Ident(u) = self.peek().clone() {
                    self.bump();
                    Some(u)
                } else {
                    None
                };
                Ok(Expr::Num(n, unit))
            }
            Tok::Ident(first) => {
                self.bump();
                let mut parts = vec![first];
                while *self.peek() == Tok::Dot {
                    self.bump();
                    parts.push(self.expect_ident()?);
                }
                Ok(Expr::Path(parts))
            }
            Tok::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                self.expect(&Tok::RParen)?;
                Ok(Expr::Paren(Box::new(e)))
            }
            other => self.err(format!("expected an expression, found {:?}", other)),
        }
    }
}
