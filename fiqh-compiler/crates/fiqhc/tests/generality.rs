//! Iter D gate: the SAME compiler accepts two further instruments (Mudarabah, Ijarah)
//! and refuses their riba/gharar negative controls. This is the "primitive" claim — the
//! fiqh engine is not hardcoded to one instrument.

use fiqhc::{codegen, compile_check};

const MUD: &str = include_str!("../../../specs/mudarabah.fiqh");
const MUD_RIBA: &str = include_str!("../../../specs/mudarabah_riba.fiqh");
const IJ: &str = include_str!("../../../specs/ijarah_imbt.fiqh");
const IJ_RIBA: &str = include_str!("../../../specs/ijarah_riba.fiqh");

fn errors(src: &str) -> Vec<String> {
    let (_, diags) = compile_check(src).expect("parses");
    diags.into_iter().filter(|d| d.is_error()).map(|d| d.code).collect()
}

#[test]
fn mudarabah_is_accepted_and_lowers() {
    assert!(errors(MUD).is_empty(), "mudarabah should pass: {:?}", errors(MUD));
    let (spec, _) = compile_check(MUD).unwrap();
    let g = codegen::generate(&spec).expect("mudarabah should lower");
    assert!(g.sol.contains("contract MudarabahGen"));
    assert!(g.sol.contains("loss falls on the rabb al-mal") || g.sol.contains("loss_on_rabb_al_mal"));
}

#[test]
fn ijarah_is_accepted_and_lowers() {
    assert!(errors(IJ).is_empty(), "ijarah should pass: {:?}", errors(IJ));
    let (spec, _) = compile_check(IJ).unwrap();
    let g = codegen::generate(&spec).expect("ijarah should lower");
    assert!(g.sol.contains("contract IjarahImbtGen"));
    assert!(g.sol.contains("transferOwnership"));
}

#[test]
fn mudarabah_negative_control_is_refused() {
    let codes = errors(MUD_RIBA);
    for expected in ["RIBA-1", "RISK-2", "PROFIT-1", "GHARAR-2"] {
        assert!(codes.iter().any(|c| c == expected), "expected {} among {:?}", expected, codes);
    }
}

#[test]
fn ijarah_negative_control_is_refused() {
    let codes = errors(IJ_RIBA);
    for expected in ["RIBA-2", "RISK-3", "IJARAH-2", "RIBA-3"] {
        assert!(codes.iter().any(|c| c == expected), "expected {} among {:?}", expected, codes);
    }
}
