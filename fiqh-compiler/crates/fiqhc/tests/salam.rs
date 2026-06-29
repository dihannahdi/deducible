//! Salam (forward sale) — the engine accepts a full-prepayment forward sale with a known object
//! and term, and refuses a deferred-price ("debt for debt") salam with an unknown term.

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/salam.fiqh");
const NEG: &str = include_str!("../../../specs/salam_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_salam_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid salam must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn debt_for_debt_salam_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "SALAM-1"), "deferred price must raise SALAM-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "SALAM-3"), "unknown term must raise SALAM-3");
    for e in errs.iter().filter(|e| e.code.starts_with("SALAM")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn salam_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function payPriceInFull"), "must emit the full-prepayment entry");
    assert!(g.sol.contains("the full salam price must be paid at the session"), "must enforce full prepayment");
    assert!(g.test_js.contains("full_prepayment"), "must generate the prepayment test");
}
