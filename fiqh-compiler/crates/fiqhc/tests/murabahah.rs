//! Murabaha (cost-plus trust sale) — the engine accepts a compliant sale and refuses one bent
//! back into a loan (time-based markup + selling before possession). Consistency, not fatwa.

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/murabahah.fiqh");
const NEG: &str = include_str!("../../../specs/murabahah_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_murabaha_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid murabaha must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn riba_murabaha_is_refused_with_citations() {
    let errs = errors(NEG);
    assert!(
        errs.iter().any(|e| e.code == "RIBA-2"),
        "a time-based markup must raise RIBA-2; got {:?}",
        errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>()
    );
    assert!(
        errs.iter().any(|e| e.code == "MUR-2"),
        "selling before possession (qabd) must raise MUR-2; got {:?}",
        errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>()
    );
    // every fiqh diagnostic must carry a daleel for human takhrij.
    for e in errs.iter().filter(|e| e.code.starts_with("RIBA") || e.code.starts_with("MUR")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn murabaha_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function acquireAsset"), "must emit the qabd step");
    assert!(g.sol.contains("cannot sell before possession (qabd)"), "must guard sell on prior possession");
    assert!(g.sol.contains("would exceed the fixed total price"), "must cap the debt at the fixed total");
    assert!(g.test_js.contains("price_certain"), "must generate the price-certainty test");
}
