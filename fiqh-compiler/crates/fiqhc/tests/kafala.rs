//! Kafala (suretyship) — the engine accepts a gratuitous guarantee with recourse for what was
//! paid, and refuses a paid guarantee that recovers a surcharge.

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/kafala.fiqh");
const NEG: &str = include_str!("../../../specs/kafala_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_kafala_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid kafala must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn paid_guarantee_with_surcharge_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "KAFALA-1"), "a fee for the guarantee must raise KAFALA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "KAFALA-2"), "a surcharge on recourse must raise KAFALA-2");
    for e in errs.iter().filter(|e| e.code.starts_with("KAFALA")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn kafala_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function recover"), "must emit recover");
    assert!(g.sol.contains("no surcharge"), "must reject a surcharge on recourse on-chain");
}
