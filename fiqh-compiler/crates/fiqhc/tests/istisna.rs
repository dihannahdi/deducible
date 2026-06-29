//! Istisna' (manufacture-to-order) — the engine accepts a made-to-spec sale (maker furnishes
//! materials, price may be progressive) and refuses one that is really hire-of-labour with an
//! undescribed object.

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/istisna.fiqh");
const NEG: &str = include_str!("../../../specs/istisna_gharar.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_istisna_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid istisna must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn mislabelled_istisna_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "ISTISNA-1"), "undescribed object must raise ISTISNA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "ISTISNA-2"), "customer-supplied material must raise ISTISNA-2");
    for e in errs.iter().filter(|e| e.code.starts_with("ISTISNA")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn istisna_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function manufacture"), "must emit the manufacture step");
    assert!(g.sol.contains("would exceed the fixed price"), "must cap the price");
    assert!(g.test_js.contains("PROGRESS instalments"), "must generate the progress-payment test");
}
