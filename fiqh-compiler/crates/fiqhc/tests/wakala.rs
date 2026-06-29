//! Wakala (agency) — the engine accepts an agency with a known fee and no guarantee, and refuses
//! one where the agent guarantees the return (a riba-bearing loan in disguise).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/wakala.fiqh");
const NEG: &str = include_str!("../../../specs/wakala_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_wakala_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid wakala must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn guaranteed_agency_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "WAKALA-1"), "a guaranteed return must raise WAKALA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    for e in errs.iter().filter(|e| e.code.starts_with("WAKALA")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn wakala_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function settle"), "must emit settle");
    assert!(g.sol.contains("no_agent_guarantee"), "must document the no-guarantee invariant");
}
