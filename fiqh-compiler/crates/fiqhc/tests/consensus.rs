//! V2-B gate: an instrument can declare a zero-trust consensus oracle; the engine validates
//! the parameters and the backend wires the ConsensusValuationOracle into the deploy descriptor.

use fiqhc::{codegen, compile_check};

const CONSENSUS: &str = include_str!("../../../specs/musharakah_consensus.fiqh");

fn errors(src: &str) -> Vec<String> {
    let (_, diags) = compile_check(src).expect("parses");
    diags.into_iter().filter(|d| d.is_error()).map(|d| d.code).collect()
}

#[test]
fn consensus_spec_is_accepted_and_wires_the_oracle() {
    assert!(errors(CONSENSUS).is_empty(), "consensus spec should pass: {:?}", errors(CONSENSUS));
    let (spec, _) = compile_check(CONSENSUS).unwrap();
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.descriptor.contains("ConsensusValuationOracle"), "descriptor must wire the consensus oracle");
    assert!(g.descriptor.contains("\"mode\": \"consensus\""));
    assert!(g.descriptor.contains("\"committee\": 5"));
    assert!(g.descriptor.contains("\"quorum\": 3"));
    assert!(g.descriptor.contains("\"ghararBoundBps\": 500"));
}

#[test]
fn quorum_exceeding_committee_is_refused() {
    let bad = r#"
instrument Bad : musharakah_mutanaqisah {
  parties { bank: financier; client: acquirer; valuer: oracle independent; arbiter: adjudicator; maslahah: beneficiary; }
  capital { bank: 8000 bps; client: 2000 bps; }
  returns { rent { basis: bank.share; rate: 1 per_bps_period; } buyout { price: oracle.fairValue * bps; transfers: bank.share -> client.share; } }
  risk { loss: proportional_to_ownership; capital_guarantee: none; }
  oracle { mode: consensus; committee: 3; quorum: 5; gharar_bound_bps: 500; }
  invariant ownership_conserved { bank.share + client.share == 10000 }
  invariant rent_on_living_share { rent.basis == bank.share }
  invariant loss_follows_capital { loss == proportional_to_ownership }
  invariant price_attested { buyout.price == oracle.fairValue }
  lifecycle { fund; payRent; buyShare(bps); settle; }
}
"#;
    let codes = errors(bad);
    assert!(codes.iter().any(|c| c == "ORACLE-3"), "quorum>committee must be refused: {:?}", codes);
}

#[test]
fn bad_gharar_bound_is_refused() {
    let bad = r#"
instrument Bad2 : musharakah_mutanaqisah {
  parties { bank: financier; client: acquirer; valuer: oracle independent; arbiter: adjudicator; maslahah: beneficiary; }
  capital { bank: 8000 bps; client: 2000 bps; }
  returns { rent { basis: bank.share; rate: 1 per_bps_period; } buyout { price: oracle.fairValue * bps; transfers: bank.share -> client.share; } }
  risk { loss: proportional_to_ownership; capital_guarantee: none; }
  oracle { mode: consensus; committee: 5; quorum: 3; gharar_bound_bps: 0; }
  invariant ownership_conserved { bank.share + client.share == 10000 }
  invariant rent_on_living_share { rent.basis == bank.share }
  invariant loss_follows_capital { loss == proportional_to_ownership }
  invariant price_attested { buyout.price == oracle.fairValue }
  lifecycle { fund; payRent; buyShare(bps); settle; }
}
"#;
    assert!(errors(bad).iter().any(|c| c == "ORACLE-4"), "gharar_bound_bps=0 must be refused");
}
