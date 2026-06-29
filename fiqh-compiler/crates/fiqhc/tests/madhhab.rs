//! Madhhab-level rule modules — the same engine, four schools, a LIVE khilaf made executable.
//! The base musharakah does not declare `profit_tracks_capital`: it is consistent under the
//! Hanafi/Hanbali modules (profit by free stipulation) and refused under the Maliki/Shafi'i modules
//! (profit must track capital). Riba is refused under every school.

use fiqhc::compile_check_with_rules;

const MMP: &str = include_str!("../../../specs/musharakah_mutanaqisah.fiqh");
const HANAFI: &str = include_str!("../../../rules/hanafi.rules.json");
const HANBALI: &str = include_str!("../../../rules/hanbali.rules.json");
const MALIKI: &str = include_str!("../../../rules/maliki.rules.json");
const SHAFII: &str = include_str!("../../../rules/shafii.rules.json");

fn err_codes(src: &str, rules: &str) -> Vec<String> {
    let (_s, d) = compile_check_with_rules(src, rules).expect("parses");
    d.iter().filter(|x| x.is_error()).map(|x| x.code.clone()).collect()
}

#[test]
fn same_spec_diverges_across_madhahib() {
    assert!(err_codes(MMP, HANAFI).is_empty(), "the base musharakah is consistent under Hanafi; got {:?}", err_codes(MMP, HANAFI));
    assert!(err_codes(MMP, HANBALI).is_empty(), "consistent under Hanbali; got {:?}", err_codes(MMP, HANBALI));
    assert!(err_codes(MMP, MALIKI).iter().any(|c| c == "INV-1"), "refused under Maliki (missing profit_tracks_capital); got {:?}", err_codes(MMP, MALIKI));
    assert!(err_codes(MMP, SHAFII).iter().any(|c| c == "INV-1"), "refused under Shafi'i; got {:?}", err_codes(MMP, SHAFII));
}

#[test]
fn riba_refused_under_every_madhhab() {
    let riba = include_str!("../../../specs/riba_disguised.fiqh");
    for (name, rules) in [("Hanafi", HANAFI), ("Hanbali", HANBALI), ("Maliki", MALIKI), ("Shafi'i", SHAFII)] {
        assert!(!err_codes(riba, rules).is_empty(), "a disguised riba must be refused under {}", name);
    }
}
