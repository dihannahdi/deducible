//! Rule-module ratification: tamper-evidence for "ratification becomes a module, not a fork"
//! (rule-modules.md). A module's `ratification` block pins a content hash to a named
//! authority and date; the engine cannot verify the encoding is FAITHFUL to the cited fatwa
//! (that remains a scholar's judgment [scholar-verify]), only that the content actually
//! loaded is byte-for-byte what was signed off on.

use fiqhc::ast::Span;
use fiqhc::sema::RuleSet;

const AAOIFI: &str = include_str!("../../../rules/aaoifi.rules.json");

fn with_ratification(patch: impl FnOnce(&mut serde_json::Value)) -> String {
    let mut j: serde_json::Value = serde_json::from_str(AAOIFI).expect("valid json");
    patch(&mut j);
    j.to_string()
}

#[test]
fn shipped_modules_are_honestly_marked_draft() {
    // None of the six shipped rule modules has actually been ratified by a real board yet —
    // the ratification block must say so, not claim a status that hasn't happened.
    for src in [
        include_str!("../../../rules/aaoifi.rules.json"),
        include_str!("../../../rules/dsn-mui.rules.json"),
        include_str!("../../../rules/hanafi.rules.json"),
        include_str!("../../../rules/hanbali.rules.json"),
        include_str!("../../../rules/maliki.rules.json"),
        include_str!("../../../rules/shafii.rules.json"),
    ] {
        let rs = RuleSet::from_json(src).expect("valid module");
        let d = rs.verify_ratification(Span::new(0, 0));
        assert_eq!(d.len(), 1, "expected exactly one RULES-3 draft warning for {}", rs.label());
        assert_eq!(d[0].code, "RULES-3");
        assert!(!d[0].is_error(), "draft status must warn, not block compilation");
    }
}

#[test]
fn missing_ratification_block_warns() {
    let src = with_ratification(|j| {
        j.as_object_mut().unwrap().remove("ratification");
    });
    let rs = RuleSet::from_json(&src).unwrap();
    let d = rs.verify_ratification(Span::new(0, 0));
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, "RULES-3");
    assert!(!d[0].is_error());
}

#[test]
fn ratified_with_matching_hash_passes_clean() {
    let src = with_ratification(|j| {
        let hash = RuleSet::from_json(&j.to_string()).unwrap().compute_content_hash();
        let r = j["ratification"].as_object_mut().unwrap();
        r.insert("status".into(), "ratified".into());
        r.insert("ratified_by".into(), "Test Shari'ah Board".into());
        r.insert("ratification_date".into(), "2026-01-01".into());
        r.insert("sha256_of_module".into(), hash.into());
    });
    let rs = RuleSet::from_json(&src).unwrap();
    let d = rs.verify_ratification(Span::new(0, 0));
    assert!(d.is_empty(), "a ratified module whose hash matches must pass with no diagnostics, got {:?}", d);
}

#[test]
fn ratified_with_tampered_content_is_refused() {
    // The content hash is computed BEFORE the constraints are tampered with, simulating a
    // module edited after a board signed off on the original content.
    let original_hash = RuleSet::from_json(AAOIFI).unwrap().compute_content_hash();
    let src = with_ratification(|j| {
        let r = j["ratification"].as_object_mut().unwrap();
        r.insert("status".into(), "ratified".into());
        r.insert("ratified_by".into(), "Test Shari'ah Board".into());
        r.insert("ratification_date".into(), "2026-01-01".into());
        r.insert("sha256_of_module".into(), original_hash.clone().into());
        // Tamper: loosen the capital-guarantee constraint after "ratification".
        j["regimes"]["islamic"]["classes"]["musharakah_mutanaqisah"]["constraints"][0]["value"] =
            "bank".into();
    });
    let rs = RuleSet::from_json(&src).unwrap();
    let d = rs.verify_ratification(Span::new(0, 0));
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, "RULES-2");
    assert!(d[0].is_error(), "hash mismatch after ratification must be a hard error, not a warning");
    assert!(d[0].message.contains(&original_hash));
}

#[test]
fn ratified_without_a_hash_is_refused() {
    let src = with_ratification(|j| {
        let r = j["ratification"].as_object_mut().unwrap();
        r.insert("status".into(), "ratified".into());
        r.remove("sha256_of_module");
    });
    let rs = RuleSet::from_json(&src).unwrap();
    let d = rs.verify_ratification(Span::new(0, 0));
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, "RULES-2");
    assert!(d[0].is_error());
}

#[test]
fn adding_or_reordering_ratification_metadata_does_not_change_the_content_hash() {
    // The hash is over `regimes` only, so a board can add its ratification record without
    // invalidating the very hash it is trying to pin.
    let bare_hash = RuleSet::from_json(AAOIFI).unwrap().compute_content_hash();
    let src = with_ratification(|j| {
        let r = j["ratification"].as_object_mut().unwrap();
        r.insert("status".into(), "ratified".into());
        r.insert("ratified_by".into(), "Another Board".into());
        r.insert("extra_metadata_field".into(), "irrelevant".into());
    });
    let rs = RuleSet::from_json(&src).unwrap();
    assert_eq!(rs.compute_content_hash(), bare_hash);
}
