//! Sarf (currency/metal exchange) — the engine accepts a spot cross-genus exchange and refuses a
//! same-genus unequal + deferred one (riba al-fadl + riba al-nasi'a together).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

const POS: &str = include_str!("../../../specs/sarf.fiqh");
const NEG: &str = include_str!("../../../specs/sarf_riba.fiqh");

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_sarf_compiles_clean() {
    let errs = errors(POS);
    assert!(
        errs.is_empty(),
        "a valid spot cross-genus sarf must compile clean; got {:?}",
        errs.iter().map(|e| (e.code.clone(), e.message.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn unequal_deferred_sarf_is_refused() {
    let errs = errors(NEG);
    assert!(errs.iter().any(|e| e.code == "SARF-1"), "deferred settlement must raise SARF-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "SARF-2"), "same-genus unequal must raise SARF-2");
    for e in errs.iter().filter(|e| e.code.starts_with("SARF")) {
        assert!(!e.citation.is_empty(), "diagnostic {} must cite its basis", e.code);
    }
}

#[test]
fn sarf_lowers_to_solidity() {
    let spec = compile_parse(POS).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function settle"), "must emit the atomic settle");
    assert!(g.sol.contains("both legs must be present (yadan bi-yad)"), "must enforce spot atomicity");
    assert!(g.sol.contains("same-genus exchange must be equal"), "must guard riba al-fadl at construction");
}
