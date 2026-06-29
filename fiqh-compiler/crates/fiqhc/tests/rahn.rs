//! Rahn (pledge) — the engine accepts a pledge that secures without forfeiting, and refuses one
//! the creditor benefits from or keeps on default.

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/rahn.fiqh");
const NEG: &str = include_str!("../../../specs/rahn_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_rahn_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid rahn must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn benefit_and_forfeiture_rahn_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "RAHN-1"), "creditor benefit must raise RAHN-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "RAHN-2"), "forfeiture must raise RAHN-2");
    for e in errs.iter().filter(|e| e.code.starts_with("RAHN")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn rahn_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function liquidate"), "must emit liquidate");
    assert!(g.sol.contains("surplus to pledgor"), "must return the surplus to the pledgor on default");
}
