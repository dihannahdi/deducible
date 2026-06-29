//! Ijarah (plain operating lease) — the engine accepts a usufruct lease where the lessor bears the
//! risk, and refuses one where rent is on principal and the lessee bears the risk (a loan).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/ijarah.fiqh");
const NEG: &str = include_str!("../../../specs/ijarah_riba_plain.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_ijarah_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid ijarah must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn interest_lease_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "RIBA-2"), "rent on principal must raise RIBA-2; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "RISK-3"), "lessee bearing risk must raise RISK-3");
}

#[test]
fn ijarah_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function returnAsset"), "must emit returnAsset (no ownership transfer)");
    assert!(g.sol.contains("pay exactly one period's rent (no surcharge)"), "must reject a rent surcharge");
}
