//! Qard Hasan (benevolent loan) — the engine accepts a loan repaid in like and refuses one
//! carrying a stipulated increase or a fee (riba).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/qard_hasan.fiqh");
const NEG: &str = include_str!("../../../specs/qard_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_qard_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid qard hasan must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn interest_bearing_qard_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "QARD-1"), "a stipulated increase must raise QARD-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "QARD-2"), "a fee conditioned on the loan must raise QARD-2");
    for e in errs.iter().filter(|e| e.code.starts_with("QARD")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn qard_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function repay"), "must emit repay");
    assert!(g.sol.contains("repay exactly the principal, no increase"), "must reject any increase on-chain");
}
