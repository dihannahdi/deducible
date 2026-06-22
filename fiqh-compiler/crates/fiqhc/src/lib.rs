//! `fiqhc` — a compliance-by-construction compiler for Islamic-finance contracts.
//!
//! Pipeline: source (`.fiqh`) → lex → parse → AST → semantic analysis (the *fiqh
//! invariant engine*) → Solidity / Hardhat-test / deploy-descriptor codegen.
//!
//! The engine does not issue a fatwa. It proves a specification is *consistent or
//! inconsistent with a declared, human-authored, citation-bearing rule-base*; the
//! fiqh validity of that rule-base remains a qualified scholar's domain. A spec
//! whose declared economics contradict its declared basis cannot be lowered to a
//! contract — compliance is a property of the language, not a runtime check.

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod lsp;
pub mod nl;
pub mod parser;
pub mod sema;

/// Lex + parse only. Returns the AST or a `(message, span)` error.
pub fn compile_parse(src: &str) -> Result<ast::Spec, (String, ast::Span)> {
    let toks = lexer::lex(src)?;
    parser::parse(toks).map_err(|e| (e.msg, e.span))
}

/// Lex + parse + run the fiqh invariant engine. Returns the AST and all diagnostics.
/// Callers gate codegen on the presence of any `Severity::Error`.
pub fn compile_check(src: &str) -> Result<(ast::Spec, Vec<sema::Diagnostic>), (String, ast::Span)> {
    let spec = compile_parse(src)?;
    let diags = sema::check(&spec);
    Ok((spec, diags))
}

/// Lex + parse + check against a PLUGGABLE rule-base module (Open Core pillar #2) — the same
/// engine, any authority's published rule module.
pub fn compile_check_with_rules(
    src: &str,
    ruleset_json: &str,
) -> Result<(ast::Spec, Vec<sema::Diagnostic>), (String, ast::Span)> {
    let spec = compile_parse(src)?;
    let rs = sema::RuleSet::from_json(ruleset_json).map_err(|e| (e, ast::Span::new(0, 0)))?;
    let diags = sema::check_with_ruleset(&spec, &rs);
    Ok((spec, diags))
}
