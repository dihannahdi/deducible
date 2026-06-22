# The `.fiqh` Language Specification

Version 0.1 · for the `fiqhc` compiler · transparent by design, for security peer review.

`.fiqh` is a declarative domain-specific language for *compliance-by-construction* legal
instruments. A specification names an instrument class, its parties, capital, return mechanism,
risk allocation, named invariants, dispute resolution, and lifecycle — in a fixed legal
vocabulary. The compiler proves the specification is **consistent with a declared, human-authored,
citation-bearing rule-base** and, only if so, lowers it to an enforcing artifact (Solidity, a
portable invariant manifest, …).

> **Epistemic boundary (normative).** The engine issues **no fatwa and no legal opinion**. It
> proves *internal consistency with rule-base R*; the validity of R — and of instruments that
> carry *khilaf* — belongs to qualified scholars and lawyers. Every citation the engine carries
> is flagged `[scholar-verify]` (fiqh) or `[verify]` (common law). "Consistent" never means
> "halal" or "enforceable in court." *Allahu a'lam.*

This document specifies the language as implemented in `crates/fiqhc` (`lexer.rs`, `parser.rs`,
`ast.rs`, `sema.rs`, `codegen.rs`). It is intended to be precise enough for an independent
re-implementation and a cyber-security audit.

---

## 1. Notation

Grammar is given in EBNF: `::=` definition, `|` alternation, `( )` grouping, `*` zero-or-more,
`+` one-or-more, `?` optional. Terminals in `"quotes"`. Token classes in `UPPERCASE`.

## 2. Lexical structure

Source is UTF-8. The lexer (`lexer.rs`) produces a flat token stream; whitespace and comments are
discarded but every token carries a `Span { line, col }` (both 1-based).

```
COMMENT   ::= "//" <any except newline>*            (* line comment *)
            | "/*" <any>* "*/"                       (* block comment, non-nested *)
IDENT     ::= [A-Za-z_] [A-Za-z0-9_]*
NUMBER    ::= [0-9] [0-9_]*                           (* underscores are ignored: 10_000 == 10000 *)
STRING    ::= '"' ( ESC | <any except '"' or '\'> )* '"'
ESC       ::= '\' ( 'n' | 't' | '"' | '\' | <any> )   (* \n \t \" \\ ; other: literal char *)
```

Punctuation / operators (longest match): `{ } ( ) : ; , .` and
`== != <= >= < > = + - * / ->`. The two-character operators (`==`, `!=`, `<=`, `>=`, `->`) take
precedence over their one-character prefixes. A bare `!` not followed by `=` is a lexical error,
as is any other unexpected character, an unterminated string, or an unterminated block comment.

Identifiers are **not** reserved at the lexical level; section keywords (`meta`, `parties`, …) and
class names are ordinary identifiers disambiguated by grammatical position. NUMBER literals are
`u64`; an overflowing literal is a lexical error.

## 3. Grammar

```
spec        ::= "instrument" IDENT ":" IDENT "{" section* "}"     (* name ":" class *)

section     ::= meta | parties | capital | returns | risk
              | oracle | dispute | invariant | rescission | lifecycle

meta        ::= "meta"       "{" kv* "}"
risk        ::= "risk"       "{" kv* "}"
oracle      ::= "oracle"     "{" kv* "}"
dispute     ::= "dispute"    "{" kv* "}"
parties     ::= "parties"    "{" party* "}"
capital     ::= "capital"    "{" capitem* "}"
returns     ::= "returns"    "{" block* "}"
rescission  ::= "rescission" "{" block* "}"
lifecycle   ::= "lifecycle"  "{" step* "}"
invariant   ::= "invariant"  IDENT "{" expr "}"

kv          ::= IDENT ":" expr ";"
party       ::= IDENT ":" IDENT IDENT* ";"            (* name ":" role flag* *)
capitem     ::= IDENT ":" NUMBER "bps" ";"            (* party share *)
              | "require" expr ";"                     (* a free constraint, carried for docs *)
block       ::= IDENT "{" kv* "}"                      (* e.g. rent { … }, khiyar_al_shart { … } *)
step        ::= IDENT ( "(" IDENT ")" )? ";"           (* lifecycle action, optional one arg *)
```

The `invariant` and `oracle`/`dispute` sections may appear more than once / be omitted; section
order is not significant. `meta`, `parties`, `capital`, `risk`, `lifecycle` are conventionally
present, but their *requirement* is a semantic matter (§6), not a grammatical one.

## 4. Expressions

Expressions appear as `kv` values, `capitem` `require` bodies, and `invariant` bodies. Precedence
(loosest to tightest), matching `parser.rs`:

```
expr     ::= arrow
arrow    ::= cmp ( "->" cmp )*                         (* left-assoc; models a transfer A -> B *)
cmp      ::= add ( ( "==" | "!=" | "<=" | ">=" | "<" | ">" ) add )?   (* non-associative *)
add      ::= mul ( ( "+" | "-" ) mul )*                (* left-assoc *)
mul      ::= primary ( ( "*" | "/" ) primary )*        (* left-assoc *)
primary  ::= STRING
           | NUMBER UNIT?                               (* UNIT is an IDENT immediately after, e.g. `8000 bps` *)
           | path
           | "(" expr ")"
path     ::= IDENT ( "." IDENT )*                       (* e.g. bank.share, oracle.fairValue *)
```

`cmp` is deliberately non-associative: `a == b == c` is a parse error. A NUMBER may be followed by
a single unit identifier (`8000 bps`, `1 per_bps_period`); the unit is retained on the node.
Expressions are **not evaluated**: the engine reasons over their structure (a path, an identifier,
a comparison), never over a runtime value.

## 5. Abstract syntax (AST)

Defined in `ast.rs`; every node carries a `Span`.

```
Spec      { name: String, class: String, sections: Vec<Section>, span }
Section   = Meta(Vec<Kv>) | Parties(Vec<Party>) | Capital(Vec<CapItem>)
          | Returns(Vec<RetBlock>) | Risk(Vec<Kv>) | Oracle(Vec<Kv>)
          | Dispute(Vec<Kv>) | Invariant(Invariant)
          | Rescission(Vec<RescBlock>) | Lifecycle(Vec<Step>)
Kv        { key: String, val: Expr, span }
Party     { name: String, role: String, flags: Vec<String>, span }
CapItem   = Assign { party: String, bps: u64, span } | Require { expr: Expr, span }
RetBlock  { kind: String, kvs: Vec<Kv>, span }         (* also used for rescission blocks *)
Invariant { name: String, expr: Expr, span }
Step      { name: String, arg: Option<String>, span }
Expr      = Str(String) | Num(u64, Option<String>) | Path(Vec<String>)
          | Bin(Box<Expr>, BinOp, Box<Expr>) | Paren(Box<Expr>)
BinOp     = Eq | Ne | Lt | Gt | Le | Ge | Add | Sub | Mul | Div | Arrow
Span      { line: usize, col: usize }                  (* 1-based *)
```

Helper accessors (`Expr::as_path / as_ident / as_num / as_str / render`) give the semantic engine
a small, total view of expressions without a general evaluator.

## 6. Parser mechanics

`parser.rs` is a hand-written recursive-descent parser over the token vector with a single
lookahead (`peek`) and no backtracking. Each `section` is dispatched by its leading keyword.
Expression parsing is precedence-climbing exactly as in §4. Errors are values, not panics:

```
ParseErr { msg: String, span: Span }
```

The parser returns the first error encountered (fail-fast) with the span of the offending token.
The lexer likewise returns `(message, span)`. The compiler never panics on malformed input — this
is enforced by the fuzz harness (`fiqhc fuzz`, `tests/fuzz.rs`): random bytes, random token soup,
and structured mutations of valid specs are run through the full front-end inside `catch_unwind`.

## 7. Semantic model — the legal invariant engine

`sema.rs` separates the **engine** (mechanical, regime-neutral) from the **rule-base** (the law).

### 7.1 Regimes and classes
The class (after `:`) selects a regime: `musharakah_mutanaqisah`, `mudarabah`, `ijarah_imbt`
belong to the `islamic` regime; `commercial_escrow` to `common_law`. A `meta { regime: … }` that
contradicts the class is rejected (`REGIME-1`). An unknown class is `CLASS-1`.

### 7.2 The fact model
The engine extracts structured facts from the AST via total accessors — party roles, the capital
split, the present return mechanisms and their `basis`/`price`/`split`, the `risk.loss` and
`risk.capital_guarantee` values, declared invariant names, the dispute remedy, the lifecycle
steps. Rules are predicates over these facts; diagnostics carry the offending node's **precise
span** (e.g. `RIBA-1` points at the `capital_guarantee` line, not the header).

### 7.3 Built-in rule-bases (per class)
- **musharakah_mutanaqisah** — `capital_guarantee == none` (`RIBA-1`); `loss ==
  proportional_to_ownership` (`RISK-1`); rent basis is not principal/capital (`RIBA-2`); a rent
  and a buyout mechanism exist (`RENT-2`/`BUYOUT-2`, fields `RENT-1`/`BUYOUT-1`); buyout price is
  oracle-derived (`GHARAR-1`); required invariants `ownership_conserved`, `rent_on_living_share`,
  `loss_follows_capital`, `price_attested` (`INV-1`); capital sums to 10000 bps with ≥2 partners
  (`CAP-1`/`CAP-2`); an independent oracle party exists (`ROLE-1`/`ROLE-2`).
- **mudarabah** — `capital_guarantee == none` (`RIBA-1`); `loss == on_rabb_al_mal` (`RISK-2`);
  the mudarib contributes labour, not capital (`MUD-1`), with both roles present (`MUD-2`/`MUD-3`)
  and the rabb funding 100% (`MUD-4`); profit split by ratio, oracle-attested (`PROFIT-1`/
  `PROFIT-2`/`GHARAR-2`); required invariants `capital_from_rabb_al_mal_only`, `profit_by_ratio`,
  `loss_on_rabb_al_mal`, `no_guaranteed_profit`.
- **ijarah_imbt** — rent for usufruct, not principal (`RIBA-2`/`IJARAH-1`/`RENT-1`/`RENT-2`);
  `loss == on_lessor` (`RISK-3`); a distinct `transferOwnership` lifecycle step (`IJARAH-2`); no
  late-payment penalty to the lessor (`RIBA-3`); required invariants `rent_for_usufruct`,
  `lessor_bears_ownership_risk`, `transfer_separate_from_lease`, `no_late_penalty_interest`.
- **commercial_escrow** (common law) — depositor/beneficiary/arbiter present (`PARTY-1`);
  consideration moves between distinct parties (`CONSID-1`); a definite amount and condition
  (`TERMS-1`/`CERTAINTY-1`/`CERTAINTY-2`); damages are liquidated, not a penalty (`PENALTY-1`);
  an arbiter-ruling remedy (`DISPUTE-1`); required invariants `certainty_of_terms`,
  `no_penalty_clause`, `consideration_present`, `dispute_resolution_present`.

The common-law rule-base parallels the Islamic one (certainty ↔ gharar; penalty doctrine ↔ riba;
consideration ↔ ʿiwaḍ) — the *same engine*, a different R.

### 7.4 Optional consensus-oracle configuration
`oracle { mode: consensus; committee: N; quorum: K; gharar_bound_bps: B; }` is validated by
`ORACLE-1..5` (mode known; `1 ≤ K ≤ N`; `0 < B < 10000`; all three present). It directs codegen to
wire a `ConsensusValuationOracle`, in which the gharar boundary is the on-chain dispersion bound.

### 7.5 Diagnostics
```
Severity   = Error | Warning
Diagnostic { code, severity, message, citation, span }
```
Codegen is gated on the absence of any `Error`. Warnings (`META-1`, `IJARAH-1`) do not block.

## 8. Pluggable rule-base modules

An authority publishes a rule module as data (`rules/<name>.rules.json`); the engine checks any
spec against it (`fiqhc check --rules <name>` or `meta { rules: "<name>"; }`). Schema:

```jsonc
{
  "authority": "AAOIFI", "version": "2017",
  "regimes": { "<regime>": { "classes": { "<class>": {
    "required_invariants": ["…"],
    "constraints": [ { "code": "RIBA-1", "field": "risk.capital_guarantee",
                       "op": "eq" | "ne" | "gt", "value": <json>, "citation": "… [scholar-verify]" } ]
  } } } }
}
```
`field` is a dotted path resolved against the spec (`risk.*`, `returns.<mech>.<key>`,
`returns.buyout.priceSource ∈ {oracle, self}`, `dispute.*`). A spec consistent under one authority
may be refused under another (e.g. DSN-MUI's extra `nisbah_explicit`) — pluggable jurisprudence.
`RULES-1` is raised when a module has no entry for the class.

## 9. Code generation

Reached only after a clean check. Targets (`codegen.rs`, selected by `fiqhc build --target`):
- **solidity** — a Solidity 0.8.24 contract per class (immutable parties, `BPS` basis points,
  `onlyX`/`live`/`nonReentrant` modifiers, one event per transition, the independent oracle as the
  trust boundary), plus a Hardhat property test and a JSON deploy descriptor. Every declared
  invariant appears as a `@dev INVARIANT` annotation; every external value-moving function carries
  `nonReentrant` (proved by `tests/codegen_safety.rs`).
- **manifest** — a portable JSON invariant manifest (`{code, field, op, value, citation}`) for
  enforcement on any ledger or traditional database via the invariant gateway.

## 10. Tooling surface

```
fiqhc parse <f.fiqh>             dump the AST
fiqhc check <f.fiqh> [--rules R] run the engine (built-in or a pluggable module)
fiqhc build <f.fiqh> [--target solidity|manifest|all] [--root DIR]
fiqhc nl    <f.txt>              draft a spec from natural language (experimental, LLM)
fiqhc lsp                        Language Server over stdio (editors)
fiqhc fuzz  [N]                  fuzz the front-end + engine
```
The engine is also exposed as a C-ABI / WebAssembly surface (`crates/fiqhc-ffi`:
`fiqh_alloc/fiqh_free/fiqh_check_json/fiqh_free_cstr`) for in-browser validation and for embedding
into legacy core-banking systems.

## 11. Diagnostic code reference

| Code | Severity | Meaning |
|---|---|---|
| META-1 | warning | no fiqh basis cited in `meta` |
| CLASS-1 | error | unknown instrument class |
| REGIME-1 | error | declared regime contradicts the class |
| ROLE-1 / ROLE-2 | error | no independent oracle / oracle not marked `independent` |
| CAP-1 / CAP-2 | error | capital ≠ 10000 bps / fewer than two contributors |
| INV-1 | error | a required invariant is not declared |
| RIBA-1 | error | capital is guaranteed (riba) |
| RIBA-2 | error/warn | return charged on principal, not the living share/usufruct |
| RIBA-3 | error | late-payment penalty to the lessor (interest on a debt) |
| RISK-1 / RISK-2 / RISK-3 | error | loss not proportional / not on rabb al-mal / not on lessor |
| GHARAR-1 / GHARAR-2 | error | price/realized value not independently attested |
| RENT-1 / RENT-2 | error | rent block missing a basis / missing entirely |
| BUYOUT-1 / BUYOUT-2 | error | buyout missing a price / missing entirely |
| MUD-1..4 | error | mudarib capital / missing roles / rabb not 100% |
| PROFIT-1 / PROFIT-2 | error | profit not by ratio / no profit mechanism |
| IJARAH-1 | warning | rent basis not `usufruct` |
| IJARAH-2 | error | no distinct `transferOwnership` step |
| PARTY-1 | error | a required commercial role is missing |
| CONSID-1 | error | consideration does not move between distinct parties |
| TERMS-1 | error | no `release` block |
| CERTAINTY-1 / CERTAINTY-2 | error | indefinite amount / indefinite condition |
| PENALTY-1 | error | a penalty clause (unenforceable; must be liquidated damages) |
| DISPUTE-1 | error | no arbiter-ruling remedy declared |
| ORACLE-1..5 | error | malformed consensus-oracle configuration |
| RULES-1 | error | pluggable module has no entry for this class |
| PARSE | error | lexical/syntactic error (LSP/FFI surface) |

## 12. Stability

This is version 0.1: the grammar and diagnostic codes may evolve. The epistemic boundary (§0) is
**stable and non-negotiable** — no version of this engine will issue a fatwa. Contributions to the
grammar, the engine, and especially the published rule modules are welcome; rule modules must be
authored or ratified by a qualified authority, never by the compiler.
