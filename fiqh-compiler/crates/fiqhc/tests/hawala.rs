//! Hawala (debt transfer) — the engine accepts a like-for-like transfer that discharges the
//! original debtor, and refuses one carrying an increase over the debt.

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/hawala.fiqh");
const NEG: &str = include_str!("../../../specs/hawala_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_hawala_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid hawala must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn increasing_hawala_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "HAWALA-1"), "a transfer above the debt must raise HAWALA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    for e in errs.iter().filter(|e| e.code.starts_with("HAWALA")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn hawala_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function acceptHawala"), "must emit acceptHawala");
    assert!(g.sol.contains("muhilDischarged"), "must discharge the original debtor");
    assert!(g.sol.contains("settle exactly the debt, no increase"), "must reject an increase on-chain");
}
