//! Fase 6 — the social economy: waqf (endowment), hibah (gift), wasiyya (bequest).

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}
fn codes(src: &str) -> Vec<String> { errors(src).iter().map(|e| e.code.clone()).collect() }

#[test]
fn waqf_valid_and_refused() {
    let pos = include_str!("../../../specs/waqf.fiqh");
    assert!(errors(pos).is_empty(), "valid waqf must compile: {:?}", codes(pos));
    let g = codegen::generate(&compile_parse(pos).unwrap()).unwrap();
    assert!(g.sol.contains("function distributeIncome"), "must distribute income while the corpus stays locked");
    let neg = codes(include_str!("../../../specs/waqf_riba.fiqh"));
    assert!(neg.contains(&"WAQF-1".to_string()) && neg.contains(&"WAQF-2".to_string()), "waqf neg => WAQF-1+2; got {:?}", neg);
}

#[test]
fn hibah_valid_and_refused() {
    let pos = include_str!("../../../specs/hibah.fiqh");
    assert!(errors(pos).is_empty(), "valid hibah must compile: {:?}", codes(pos));
    let neg = codes(include_str!("../../../specs/hibah_riba.fiqh"));
    assert!(neg.contains(&"HIBAH-1".to_string()) && neg.contains(&"HIBAH-2".to_string()), "hibah neg => HIBAH-1+2; got {:?}", neg);
}

#[test]
fn wasiyya_valid_and_refused() {
    let pos = include_str!("../../../specs/wasiyya.fiqh");
    assert!(errors(pos).is_empty(), "valid wasiyya must compile: {:?}", codes(pos));
    let neg = codes(include_str!("../../../specs/wasiyya_riba.fiqh"));
    assert!(neg.contains(&"WASIYYA-1".to_string()) && neg.contains(&"WASIYYA-2".to_string()), "wasiyya neg => WASIYYA-1+2; got {:?}", neg);
}
