//! Takaful (cooperative) and Mudarabah pool — multilateral instruments over the participant pool.

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_takaful_compiles_and_lowers() {
    let src = include_str!("../../../specs/takaful.fiqh");
    assert!(errors(src).is_empty(), "valid takaful must compile: {:?}", errors(src).iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    let spec = compile_parse(src).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function distributeSurplus"), "surplus must return to participants");
}

#[test]
fn commercial_insurance_is_refused() {
    let errs = errors(include_str!("../../../specs/takaful_riba.fiqh"));
    assert!(errs.iter().any(|e| e.code == "TAKAFUL-1"), "a premium => TAKAFUL-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "TAKAFUL-2"), "surplus to operator => TAKAFUL-2");
}

#[test]
fn valid_mudarabah_pool_compiles_and_lowers() {
    let src = include_str!("../../../specs/mudarabah_pool.fiqh");
    assert!(errors(src).is_empty(), "valid mudarabah_pool must compile: {:?}", errors(src).iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    let spec = compile_parse(src).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("address[] public rabbs"), "must emit the rabb al-mal pool");
}

#[test]
fn guaranteed_mudarabah_pool_is_refused() {
    let errs = errors(include_str!("../../../specs/mudarabah_pool_riba.fiqh"));
    assert!(errs.iter().any(|e| e.code == "RIBA-1"), "a capital guarantee => RIBA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "RISK-2"), "loss on mudarib => RISK-2");
}
