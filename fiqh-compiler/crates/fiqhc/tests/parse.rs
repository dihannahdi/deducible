//! Iter A gate: the Musharakah spec parses into the expected AST shape.

use fiqhc::ast::Section;
use fiqhc::compile_parse;

const MMP: &str = include_str!("../../../specs/musharakah_mutanaqisah.fiqh");

#[test]
fn parses_musharakah() {
    let spec = compile_parse(MMP).expect("musharakah spec should parse");
    assert_eq!(spec.name, "MusharakahMutanaqisah");
    assert_eq!(spec.class, "musharakah_mutanaqisah");

    let parties = spec.parties();
    assert_eq!(parties.len(), 5, "expected 5 parties");
    assert!(parties.iter().any(|p| p.name == "valuer"
        && p.role == "oracle"
        && p.flags.iter().any(|f| f == "independent")));

    // capital split sums to 10000 bps
    let total: u64 = spec
        .capital()
        .iter()
        .filter_map(|c| match c {
            fiqhc::ast::CapItem::Assign { bps, .. } => Some(*bps),
            _ => None,
        })
        .sum();
    assert_eq!(total, 10000);

    // the four core invariants plus role separation
    assert!(spec.has_invariant("ownership_conserved"));
    assert!(spec.has_invariant("rent_on_living_share"));
    assert!(spec.has_invariant("loss_follows_capital"));
    assert!(spec.has_invariant("price_attested"));
    assert!(spec.invariants().len() >= 5);

    // lifecycle has a parametric buyShare step
    assert!(spec
        .lifecycle()
        .iter()
        .any(|s| s.name == "buyShare" && s.arg.as_deref() == Some("bps")));

    // returns + rescission present
    assert!(spec.returns().iter().any(|r| r.kind == "rent"));
    assert!(spec.returns().iter().any(|r| r.kind == "buyout"));
    assert!(spec.rescission().iter().any(|r| r.kind == "khiyar_al_shart"));

    // meta retained the (scholar-verify) basis string
    let basis = fiqhc::ast::kv_get(
        match &spec.sections[0] {
            Section::Meta(m) => m,
            _ => panic!("first section should be meta"),
        },
        "basis",
    );
    assert!(basis.and_then(|e| e.as_str()).unwrap_or("").contains("AAOIFI"));
}
