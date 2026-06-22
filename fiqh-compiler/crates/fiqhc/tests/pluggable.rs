//! Open Core pillar #2 gate: the engine is separated from the rule-base. The SAME spec is
//! checked against different authorities' published rule modules — and jurisdictions may
//! diverge (a spec accepted under AAOIFI can be refused under DSN-MUI, and vice-versa).

use fiqhc::compile_check_with_rules;

const AAOIFI: &str = include_str!("../../../rules/aaoifi.rules.json");
const DSN: &str = include_str!("../../../rules/dsn-mui.rules.json");
const MMP: &str = include_str!("../../../specs/musharakah_mutanaqisah.fiqh");
const MMP_DSN: &str = include_str!("../../../specs/musharakah_dsn.fiqh");
const RIBA: &str = include_str!("../../../specs/riba_disguised.fiqh");

fn errs(src: &str, rules: &str) -> Vec<fiqhc::sema::Diagnostic> {
    let (_, d) = compile_check_with_rules(src, rules).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn aaoifi_accepts_base_musharakah() {
    assert!(errs(MMP, AAOIFI).is_empty(), "should pass under AAOIFI");
}

#[test]
fn dsn_diverges_on_the_same_spec() {
    // The base musharakah lacks `nisbah_explicit`, which the DSN-MUI module requires.
    let codes: Vec<String> = errs(MMP, DSN).into_iter().map(|d| d.code).collect();
    assert!(codes.iter().any(|c| c == "INV-1"), "DSN-MUI should require nisbah_explicit; got {:?}", codes);
    // The DSN-tailored spec passes under DSN-MUI.
    assert!(errs(MMP_DSN, DSN).is_empty(), "the DSN spec should pass under DSN-MUI");
}

#[test]
fn riba_refused_under_both_with_their_own_citations() {
    let a = errs(RIBA, AAOIFI);
    let d = errs(RIBA, DSN);
    assert!(!a.is_empty() && !d.is_empty(), "riba refused under both");
    assert!(a.iter().any(|x| x.code == "RIBA-1" && x.citation.contains("AAOIFI")), "AAOIFI citation");
    assert!(d.iter().any(|x| x.code == "RIBA-1" && x.citation.contains("DSN-MUI")), "DSN-MUI citation");
}
