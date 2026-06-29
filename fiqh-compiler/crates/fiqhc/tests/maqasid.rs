//! The maqasid / hiyal-risk surfacing layer raises WARNINGS (never errors): a form-compliant
//! murabaha still compiles, but the engine flags the maqsad question for a scholar.

use fiqhc::compile_check;

#[test]
fn murabaha_compiles_clean_but_carries_a_maqsad_warning() {
    let (_s, d) = compile_check(include_str!("../../../specs/murabahah.fiqh")).expect("parses");
    assert!(d.iter().all(|x| !x.is_error()), "murabaha must still compile (no errors): {:?}", d.iter().filter(|x| x.is_error()).map(|x| x.code.clone()).collect::<Vec<_>>());
    let maqsad: Vec<_> = d.iter().filter(|x| x.code == "MAQASID-1").collect();
    assert_eq!(maqsad.len(), 1, "must surface exactly one MAQASID-1 warning");
    assert!(!maqsad[0].is_error(), "MAQASID-1 must be a WARNING, never an error");
    assert!(!maqsad[0].citation.is_empty(), "the maqsad warning must carry its basis");
}

#[test]
fn a_lease_carries_no_maqsad_smell() {
    // No known circumvention pattern matches a plain ijarah — and that is NOT a ruling of soundness.
    let (_s, d) = compile_check(include_str!("../../../specs/ijarah.fiqh")).expect("parses");
    assert!(d.iter().all(|x| !x.code.starts_with("MAQASID")), "a plain lease triggers no maqsad warning");
}
