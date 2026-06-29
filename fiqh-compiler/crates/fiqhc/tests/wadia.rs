//! Wadia (safekeeping) — the engine accepts a deposit held as amana and refuses one that is
//! guaranteed and used by the custodian (an interest account in disguise).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/wadia.fiqh");
const NEG: &str = include_str!("../../../specs/wadia_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_wadia_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid wadia must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn guaranteed_used_deposit_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "WADIA-1"), "a guaranteed deposit must raise WADIA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "WADIA-2"), "a used deposit must raise WADIA-2");
    for e in errs.iter().filter(|e| e.code.starts_with("WADIA")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn wadia_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function withdraw"), "must emit withdraw");
    assert!(g.sol.contains("only depositor"), "the custodian must have no power to move the deposit");
}
