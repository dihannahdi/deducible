//! Vector 4 — lifecycle off-ramps: jaa'ihah abatement + faraid dissolution.
//!
//! Proves the contingency section validates correctly, the faraid engine is reachable and
//! exact, and the generated contract carries the off-ramp functions.

use fiqhc::{codegen, compile_check};
use fiqhc::faraid::{distribute, Heirs, Spouse};

fn error_codes(src: &str) -> Vec<String> {
    let (_s, diags) = compile_check(src).expect("spec should parse");
    diags.iter().filter(|d| d.is_error()).map(|d| d.code.clone()).collect()
}

#[test]
fn contingency_spec_compiles_and_emits_offramps() {
    let src = include_str!("../../../specs/musharakah_jaaihah.fiqh");
    let (spec, diags) = compile_check(src).expect("spec should parse");
    let errors: Vec<&String> = diags.iter().filter(|d| d.is_error()).map(|d| &d.code).collect();
    assert!(errors.is_empty(), "a correct contingency spec must compile; got {:?}", errors);

    let g = codegen::generate(&spec).expect("codegen");
    assert!(g.sol.contains("function declareJaaihah"), "must emit the jaa'ihah off-ramp");
    assert!(g.sol.contains("function effectiveRentDue"), "rent must abate under jaa'ihah");
    assert!(g.sol.contains("function dissolveByFaraid"), "must emit the faraid dissolution");
    assert!(g.sol.contains("faraid shares must total 10000 bps"), "shares must be validated");
    assert!(g.test_js.contains("FaraidDissolution"), "generated test must exercise the dissolution");
}

#[test]
fn death_by_discretion_is_refused() {
    let codes = error_codes(include_str!("../../../specs/musharakah_death_discretion.fiqh"));
    assert!(codes.iter().any(|c| c == "CONT-2"), "distributing by discretion must be refused; got {:?}", codes);
}

#[test]
fn jaaihah_penalty_handler_is_refused() {
    let src = r#"
instrument J : musharakah_mutanaqisah {
  meta { basis: "x"; currency: tinybar; }
  parties { bank: financier; client: acquirer; valuer: oracle independent; arbiter: adjudicator; maslahah: beneficiary; }
  capital { bank: 8000 bps; client: 2000 bps; require bank + client == 10000 bps; }
  returns { rent { basis: bank.share; rate: 1 per_bps_period; } buyout { price: oracle.fairValue * bps; transfers: bank.share -> client.share; } }
  risk { loss: proportional_to_ownership; capital_guarantee: none; }
  contingency { jaaihah: penalty; }
  invariant ownership_conserved  { bank.share + client.share == 10000 }
  invariant rent_on_living_share { rent.basis == bank.share }
  invariant loss_follows_capital { loss == proportional_to_ownership }
  invariant price_attested       { buyout.price == oracle.fairValue }
  lifecycle { fund; payRent; buyShare(bps); settle; }
}
"#;
    let codes = error_codes(src);
    assert!(codes.iter().any(|c| c == "CONT-1"), "a jaa'ihah penalty must be refused; got {:?}", codes);
}

#[test]
fn faraid_engine_is_exact_and_reachable() {
    // daughter + mother, radd 3:1 -> 7500 / 2500 (the split the generated test uses).
    let h = Heirs { spouse: Spouse::None, father: false, mother: true, sons: 0, daughters: 1 };
    let s = distribute(&h).unwrap();
    let total: u64 = s.iter().map(|x| x.bps).sum();
    assert_eq!(total, 10_000);
    assert_eq!(s.iter().find(|x| x.heir == "daughters").unwrap().bps, 7500);
    assert_eq!(s.iter().find(|x| x.heir == "mother").unwrap().bps, 2500);
}
