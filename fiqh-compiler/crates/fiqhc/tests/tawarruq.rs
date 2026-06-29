//! Tawarruq (individual) — the engine accepts the licit form (third-party onward sale, possession
//! first) and refuses the ring that sells back to / is arranged by the financier ('inah + munazzam).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/tawarruq.fiqh");
const NEG: &str = include_str!("../../../specs/tawarruq_inah.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_individual_tawarruq_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid individual tawarruq must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn inah_ring_tawarruq_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "TAWARRUQ-1"), "sale back to the financier must raise TAWARRUQ-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "TAWARRUQ-3"), "financier-arranged sale must raise TAWARRUQ-3");
    for e in errs.iter().filter(|e| e.code.starts_with("TAWARRUQ")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn tawarruq_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function sellSpot"), "must emit the onward sale");
    assert!(g.sol.contains("spot buyer must differ from the credit seller"), "must guard 'inah at construction");
    assert!(g.sol.contains("must take possession (qabd) before reselling"), "must enforce qabd");
}
