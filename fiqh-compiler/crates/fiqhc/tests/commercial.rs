//! V3 gate: compliance-by-construction is universal across legal regimes. The SAME engine
//! accepts a common-law commercial escrow and refuses a penalty/indefinite negative control;
//! a regime/class mismatch is itself an inconsistency.

use fiqhc::{codegen, compile_check};

const ESCROW: &str = include_str!("../../../specs/commercial_escrow.fiqh");
const PENALTY: &str = include_str!("../../../specs/commercial_escrow_penalty.fiqh");

fn errors(src: &str) -> Vec<String> {
    let (_, diags) = compile_check(src).expect("parses");
    diags.into_iter().filter(|d| d.is_error()).map(|d| d.code).collect()
}

#[test]
fn commercial_escrow_is_accepted_and_lowers_with_a_judiciary_engine() {
    assert!(errors(ESCROW).is_empty(), "commercial escrow should pass: {:?}", errors(ESCROW));
    let (spec, _) = compile_check(ESCROW).unwrap();
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("contract CommercialEscrowGen"));
    assert!(g.sol.contains("arbiterRuling"), "must emit the code-based judiciary engine");
}

#[test]
fn penalty_and_indefinite_terms_are_refused() {
    let codes = errors(PENALTY);
    for expected in ["PENALTY-1", "CERTAINTY-1", "DISPUTE-1"] {
        assert!(codes.iter().any(|c| c == expected), "expected {} among {:?}", expected, codes);
    }
}

#[test]
fn regime_class_mismatch_is_refused() {
    let bad = r#"
instrument Mismatch : musharakah_mutanaqisah {
  meta { regime: common_law; basis: "x"; }
  parties { bank: financier; client: acquirer; valuer: oracle independent; arbiter: adjudicator; maslahah: beneficiary; }
  capital { bank: 8000 bps; client: 2000 bps; }
  returns { rent { basis: bank.share; rate: 1 per_bps_period; } buyout { price: oracle.fairValue * bps; transfers: bank.share -> client.share; } }
  risk { loss: proportional_to_ownership; capital_guarantee: none; }
  invariant ownership_conserved { bank.share + client.share == 10000 }
  invariant rent_on_living_share { rent.basis == bank.share }
  invariant loss_follows_capital { loss == proportional_to_ownership }
  invariant price_attested { buyout.price == oracle.fairValue }
  lifecycle { fund; payRent; buyShare(bps); settle; }
}
"#;
    assert!(errors(bad).iter().any(|c| c == "REGIME-1"), "declared regime contradicting the class must be refused");
}
