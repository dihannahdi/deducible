//! Iter B gate: the fiqh invariant engine accepts the compliant Musharakah spec
//! with zero errors, and REFUSES the riba-disguised spec with cited diagnostics.

use fiqhc::compile_check;

const MMP: &str = include_str!("../../../specs/musharakah_mutanaqisah.fiqh");
const RIBA: &str = include_str!("../../../specs/riba_disguised.fiqh");

fn error_codes(src: &str) -> Vec<String> {
    let (_, diags) = compile_check(src).expect("should parse");
    diags
        .into_iter()
        .filter(|d| d.is_error())
        .map(|d| d.code)
        .collect()
}

#[test]
fn compliant_musharakah_passes() {
    let codes = error_codes(MMP);
    assert!(codes.is_empty(), "expected zero errors, got {:?}", codes);
}

#[test]
fn riba_disguised_is_refused() {
    let codes = error_codes(RIBA);
    assert!(!codes.is_empty(), "the disguised loan must be refused");

    // The chorus of cited refusals we expect:
    for expected in ["RIBA-1", "RIBA-2", "RISK-1", "GHARAR-1"] {
        assert!(
            codes.iter().any(|c| c == expected),
            "expected diagnostic {} among {:?}",
            expected,
            codes
        );
    }
    // Two required invariants are missing (loss_follows_capital, price_attested).
    let missing = codes.iter().filter(|c| *c == "INV-1").count();
    assert!(missing >= 2, "expected >=2 missing-invariant errors, got {} in {:?}", missing, codes);
}

#[test]
fn every_refusal_carries_a_citation() {
    // Each fiqh-grounded refusal must cite a source (so a human can verify it).
    let (_, diags) = compile_check(RIBA).expect("parses");
    for d in diags.iter().filter(|d| d.is_error()) {
        // structural errors (INV/CAP/CLASS) may have empty citations; fiqh ones must not.
        if ["RIBA-1", "RIBA-2", "RISK-1", "GHARAR-1"].contains(&d.code.as_str()) {
            assert!(!d.citation.is_empty(), "{} must carry a citation", d.code);
            assert!(d.citation.contains("[scholar-verify]"), "{} citation must be flagged for takhrij", d.code);
        }
    }
}

#[test]
fn diagnostics_point_at_the_offending_token() {
    // Open Core pillar #1: precise spans. RIBA-1 must point at the capital_guarantee line,
    // not the instrument header — so the editor underlines the right place.
    let (_, diags) = compile_check(RIBA).expect("parses");
    let riba1 = diags.iter().find(|d| d.code == "RIBA-1" && d.is_error()).expect("RIBA-1");
    let line = RIBA.lines().nth(riba1.span.line.saturating_sub(1)).unwrap_or("");
    assert!(
        line.contains("capital_guarantee"),
        "RIBA-1 should point at the capital_guarantee line; got line {}: {:?}",
        riba1.span.line,
        line
    );
}
