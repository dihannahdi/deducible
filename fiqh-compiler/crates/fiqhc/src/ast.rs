//! Abstract syntax for the `.fiqh` DSL.
//!
//! The grammar is deliberately small and declarative: a contract is described in
//! a fixed fiqh vocabulary (parties/roles, capital split, return mechanism, risk
//! allocation, named invariants, rescission options, lifecycle). The semantic
//! engine reasons over these structured facts — it does not evaluate an arbitrary
//! expression language. This is what makes "compliance by construction" a property
//! of the *language*: a contract whose declared economics contradict its declared
//! fiqh basis cannot be lowered to Solidity.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub name: String,
    pub class: String,
    pub sections: Vec<Section>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Section {
    Meta(Vec<Kv>),
    Parties(Vec<Party>),
    Capital(Vec<CapItem>),
    Returns(Vec<RetBlock>),
    Risk(Vec<Kv>),
    Oracle(Vec<Kv>),
    Dispute(Vec<Kv>),
    Zakat(Vec<Kv>),
    Contingency(Vec<Kv>),
    Invariant(Invariant),
    Rescission(Vec<RescBlock>),
    Lifecycle(Vec<Step>),
}

#[derive(Debug, Clone)]
pub struct Kv {
    pub key: String,
    pub val: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Party {
    pub name: String,
    pub role: String,
    pub flags: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum CapItem {
    Assign { party: String, bps: u64, span: Span },
    Require { expr: Expr, span: Span },
}

#[derive(Debug, Clone)]
pub struct RetBlock {
    pub kind: String,
    pub kvs: Vec<Kv>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RescBlock {
    pub kind: String,
    pub kvs: Vec<Kv>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Invariant {
    pub name: String,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub name: String,
    pub arg: Option<String>,
    pub span: Span,
}

// --- Composite contracts (al-'uqud al-murakkabah) ---
//
// A `bundle` describes several legs (sub-contracts) and the asset/cash flows between
// the parties. A single leg may be impeccable; their *composition* may still encode a
// ruse — most notably bay' al-'inah (a sale followed by a buy-back) or organized
// tawarruq, where the round-trip of the same asset disguises an interest-bearing loan.
// The semantic engine builds the flow graph and refuses cyclic structures that no
// single-contract check could see. This is the graph-based invariant checker.

#[derive(Debug, Clone)]
pub struct Bundle {
    pub name: String,
    pub sections: Vec<BundleSection>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum BundleSection {
    Meta(Vec<Kv>),
    Parties(Vec<Party>),
    Legs(Vec<Leg>),
}

/// One leg of a composite: `<id>: <kind> { from; to; asset; payment; price }`.
/// `kind` is a fiqh sale/agency form (murabahah, bay, musawamah, wakalah, …); the
/// engine reasons over the *flows*, not the label.
#[derive(Debug, Clone)]
pub struct Leg {
    pub id: String,
    pub kind: String,
    pub kvs: Vec<Kv>,
    pub span: Span,
}

impl Bundle {
    pub fn meta(&self) -> Vec<&Kv> {
        self.sections
            .iter()
            .find_map(|s| if let BundleSection::Meta(m) = s { Some(m) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn parties(&self) -> Vec<&Party> {
        self.sections
            .iter()
            .find_map(|s| if let BundleSection::Parties(p) = s { Some(p) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn legs(&self) -> Vec<&Leg> {
        self.sections
            .iter()
            .find_map(|s| if let BundleSection::Legs(l) = s { Some(l) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Str(String),
    Num(u64, Option<String>),
    Path(Vec<String>),
    Bin(Box<Expr>, BinOp, Box<Expr>),
    Paren(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Arrow,
}

impl Expr {
    /// A dotted path (`bank.share`) or bare identifier (treated as a 1-element path).
    pub fn as_path(&self) -> Option<&[String]> {
        match self {
            Expr::Path(p) => Some(p),
            Expr::Paren(e) => e.as_path(),
            _ => None,
        }
    }

    /// A bare identifier (`none`, `proportional_to_ownership`, `arbiter`).
    pub fn as_ident(&self) -> Option<&str> {
        match self {
            Expr::Path(p) if p.len() == 1 => Some(p[0].as_str()),
            Expr::Paren(e) => e.as_ident(),
            _ => None,
        }
    }

    pub fn as_num(&self) -> Option<u64> {
        match self {
            Expr::Num(n, _) => Some(*n),
            Expr::Paren(e) => e.as_num(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Expr::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Render an expression back to a compact source-like string, for diagnostics
    /// and as `@dev INVARIANT` comments in generated Solidity.
    pub fn render(&self) -> String {
        match self {
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::Num(n, Some(u)) => format!("{} {}", n, u),
            Expr::Num(n, None) => n.to_string(),
            Expr::Path(p) => p.join("."),
            Expr::Paren(e) => format!("({})", e.render()),
            Expr::Bin(l, op, r) => format!("{} {} {}", l.render(), op.symbol(), r.render()),
        }
    }
}

impl BinOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Le => "<=",
            BinOp::Ge => ">=",
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Arrow => "->",
        }
    }
}

impl Spec {
    pub fn parties(&self) -> Vec<&Party> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Parties(p) = s { Some(p) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn capital(&self) -> Vec<&CapItem> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Capital(c) = s { Some(c) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn returns(&self) -> Vec<&RetBlock> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Returns(r) = s { Some(r) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn risk(&self) -> Vec<&Kv> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Risk(r) = s { Some(r) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn oracle_cfg(&self) -> Vec<&Kv> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Oracle(o) = s { Some(o) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn dispute_cfg(&self) -> Vec<&Kv> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Dispute(o) = s { Some(o) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn zakat_cfg(&self) -> Vec<&Kv> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Zakat(z) = s { Some(z) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn contingency_cfg(&self) -> Vec<&Kv> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Contingency(c) = s { Some(c) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn rescission(&self) -> Vec<&RescBlock> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Rescission(r) = s { Some(r) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn lifecycle(&self) -> Vec<&Step> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Lifecycle(l) = s { Some(l) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn invariants(&self) -> Vec<&Invariant> {
        self.sections
            .iter()
            .filter_map(|s| if let Section::Invariant(i) = s { Some(i) } else { None })
            .collect()
    }

    pub fn has_invariant(&self, name: &str) -> bool {
        self.invariants().iter().any(|i| i.name == name)
    }

    pub fn meta(&self) -> Vec<&Kv> {
        self.sections
            .iter()
            .find_map(|s| if let Section::Meta(m) = s { Some(m) } else { None })
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}

/// Look up a key in a list of key/values.
pub fn kv_get<'a>(kvs: &'a [Kv], key: &str) -> Option<&'a Expr> {
    kvs.iter().find(|k| k.key == key).map(|k| &k.val)
}
