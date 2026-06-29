//! Ju'ala (reward-for-result) and 'Ariyya (gratuitous usufruct loan).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_juala_compiles_and_lowers() {
    let src = include_str!("../../../specs/juala.fiqh");
    assert!(errors(src).is_empty(), "valid ju'ala must compile: {:?}", errors(src).iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    let spec = compile_parse(src).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("reward is due only on completion"), "must gate the reward on completion");
}

#[test]
fn upfront_unknown_juala_is_refused() {
    let errs = errors(include_str!("../../../specs/juala_gharar.fiqh"));
    assert!(errs.iter().any(|e| e.code == "JUALA-1"), "unknown reward => JUALA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "JUALA-2"), "not-on-completion => JUALA-2");
}

#[test]
fn valid_ariyah_compiles_and_lowers() {
    let src = include_str!("../../../specs/ariyah.fiqh");
    assert!(errors(src).is_empty(), "valid 'ariyya must compile: {:?}", errors(src).iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    let spec = compile_parse(src).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function returnAsset"), "must return the same asset");
}

#[test]
fn charged_ariyah_is_refused() {
    let errs = errors(include_str!("../../../specs/ariyah_riba.fiqh"));
    assert!(errs.iter().any(|e| e.code == "ARIYAH-1"), "a fee => ARIYAH-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "ARIYAH-2"), "returning the like => ARIYAH-2");
}
