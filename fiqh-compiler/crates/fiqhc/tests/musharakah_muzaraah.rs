//! Musharakah (full partnership) and Muzara'ah (sharecropping).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_musharakah_compiles_and_lowers() {
    let src = include_str!("../../../specs/musharakah.fiqh");
    assert!(errors(src).is_empty(), "valid musharakah must compile: {:?}", errors(src).iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    let spec = compile_parse(src).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("loss_by_capital"), "must document loss-by-capital");
}

#[test]
fn guaranteed_partnership_is_refused() {
    let errs = errors(include_str!("../../../specs/musharakah_riba.fiqh"));
    assert!(errs.iter().any(|e| e.code == "RIBA-1"), "a capital guarantee => RIBA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "RISK-1"), "loss not by capital => RISK-1");
}

#[test]
fn valid_muzaraah_compiles_and_lowers() {
    let src = include_str!("../../../specs/muzaraah.fiqh");
    assert!(errors(src).is_empty(), "valid muzara'ah must compile: {:?}", errors(src).iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    let spec = compile_parse(src).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("function splitCrop"), "must share the actual harvest");
}

#[test]
fn fixed_rent_sharecropping_is_refused() {
    let errs = errors(include_str!("../../../specs/muzaraah_riba.fiqh"));
    assert!(errs.iter().any(|e| e.code == "MUZARA-1"), "fixed quantity => MUZARA-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "MUZARA-2"), "fixed rent => MUZARA-2");
}
