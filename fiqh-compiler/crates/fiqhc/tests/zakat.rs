//! Vector 5 — the built-in Zakat al-Tijarah layer.
//!
//! Proves: (a) the compute module is exact integer arithmetic; (b) a spec that declares a
//! zakat { } section compiles and the generated contract carries the non-bypassable 2.5%
//! routing; (c) a wrong rate or a solar haul is refused with a cited diagnostic.

use fiqhc::{codegen, compile_check, zakat};

fn error_codes(src: &str) -> Vec<String> {
    let (_s, diags) = compile_check(src).expect("spec should parse");
    diags.iter().filter(|d| d.is_error()).map(|d| d.code.clone()).collect()
}

#[test]
fn compute_module_is_exact() {
    assert_eq!(zakat::zakat_due(849, 850, 250), 0); // below nisab
    assert_eq!(zakat::zakat_due(1_000_000, 850, 250), 25_000); // exactly 1/40
    assert_eq!(zakat::zakat_due(1_000_000, 850, 250), 1_000_000 / 40);
}

#[test]
fn zakat_spec_compiles_and_emits_nonbypassable_routing() {
    let src = include_str!("../../../specs/musharakah_zakat.fiqh");
    let (spec, diags) = compile_check(src).expect("spec should parse");
    let errors: Vec<&String> = diags.iter().filter(|d| d.is_error()).map(|d| &d.code).collect();
    assert!(errors.is_empty(), "a correct zakat spec must compile; got {:?}", errors);

    let g = codegen::generate(&spec).expect("codegen");
    assert!(g.sol.contains("function payZakat"), "generated contract must route zakat");
    assert!(g.sol.contains("ZAKAT_NISAB = 8500000"), "nisab must be compiled in as a constant");
    assert!(g.sol.contains("ZAKAT_RATE_BPS = 250"), "rate must be compiled in as 1/40");
    assert!(g.sol.contains("maslahahFund.call"), "zakat must route to the maslahah/zakat fund");
    assert!(g.test_js.contains("ZakatRouted"), "generated test must exercise the zakat path");

    // The same routing surfaces in the portable manifest.
    let manifest = codegen::build_manifest(&spec);
    assert!(manifest.contains("\"zakat.rate_bps\""), "manifest must carry the zakat constraint");
}

#[test]
fn instrument_without_zakat_has_no_routing() {
    // The default musharakah spec declares no zakat { }; the generated contract must be
    // byte-for-byte free of zakat routing (opt-in semantics, no destabilization).
    let src = include_str!("../../../specs/musharakah_mutanaqisah.fiqh");
    let (spec, _d) = compile_check(src).expect("spec should parse");
    let g = codegen::generate(&spec).expect("codegen");
    assert!(!g.sol.contains("payZakat"), "an instrument without a zakat block must not emit zakat routing");
}

#[test]
fn solar_haul_is_refused() {
    let codes = error_codes(include_str!("../../../specs/musharakah_zakat_solar.fiqh"));
    assert!(codes.iter().any(|c| c == "ZAKAT-2"), "a solar haul must be refused; got {:?}", codes);
}

#[test]
fn wrong_rate_is_refused() {
    let src = r#"
instrument WrongRate : musharakah_mutanaqisah {
  meta { basis: "x"; currency: tinybar; }
  parties { bank: financier; client: acquirer; valuer: oracle independent; arbiter: adjudicator; maslahah: beneficiary; }
  capital { bank: 8000 bps; client: 2000 bps; require bank + client == 10000 bps; }
  returns { rent { basis: bank.share; rate: 1 per_bps_period; } buyout { price: oracle.fairValue * bps; transfers: bank.share -> client.share; } }
  risk { loss: proportional_to_ownership; capital_guarantee: none; }
  zakat { rate_bps: 200; nisab: 100 tinybar; haul: hijri_year; beneficiary: maslahah; }
  invariant ownership_conserved  { bank.share + client.share == 10000 }
  invariant rent_on_living_share { rent.basis == bank.share }
  invariant loss_follows_capital { loss == proportional_to_ownership }
  invariant price_attested       { buyout.price == oracle.fairValue }
  lifecycle { fund; payRent; buyShare(bps); settle; }
}
"#;
    let codes = error_codes(src);
    assert!(codes.iter().any(|c| c == "ZAKAT-1"), "a non-1/40 rate must be refused; got {:?}", codes);
}
