//! Vector 2 — the graph-based invariant checker for composite contracts.
//!
//! Each leg here is, in isolation, a valid sale. The tests prove the compiler refuses the
//! ILLICIT COMPOSITIONS (bay' al-'inah, organized tawarruq, monetization flips) while
//! letting a genuine acyclic composite through — compliance as a property of the graph.

use fiqhc::compile_check_bundle;

fn error_codes(src: &str) -> Vec<String> {
    let (_b, diags) = compile_check_bundle(src).expect("bundle should parse");
    diags
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.code.clone())
        .collect()
}

#[test]
fn inah_buyback_is_refused() {
    let codes = error_codes(include_str!("../../../specs/inah_disguised.fiqh"));
    assert!(
        codes.iter().any(|c| c == "INAH-1"),
        "a sale-then-buyback of the same asset must be refused as bay' al-'inah; got {:?}",
        codes
    );
}

#[test]
fn organized_tawarruq_ring_is_refused() {
    let codes = error_codes(include_str!("../../../specs/tawarruq_munazzam.fiqh"));
    assert!(
        codes.iter().any(|c| c == "INAH-2"),
        "a commodity ring returning to the financier must be refused as organized tawarruq; got {:?}",
        codes
    );
}

#[test]
fn halal_murabahah_composite_compiles() {
    let (_b, diags) = compile_check_bundle(include_str!("../../../specs/murabahah_composite.fiqh"))
        .expect("bundle should parse");
    let errors: Vec<&String> = diags
        .iter()
        .filter(|d| d.is_error())
        .map(|d| &d.code)
        .collect();
    assert!(
        errors.is_empty(),
        "an acyclic wakalah+murabahah composite must compile; got errors {:?}",
        errors
    );
}

#[test]
fn monetization_flip_to_third_party_is_refused() {
    // The customer buys deferred and immediately sells the same asset spot to a broker who
    // does NOT return it to the bank. Cash now, larger debt later = tawarruq monetization,
    // even though there is no closed cycle back to the financier.
    let src = r#"
bundle Flip {
  meta { basis: "monetization flip"; regime: islamic; }
  parties { bank: financier; customer: client; broker: intermediary; }
  legs {
    buy:  murabahah { from: bank;     to: customer; asset: gold; payment: deferred; price: 11000; }
    sell: bay       { from: customer; to: broker;   asset: gold; payment: spot;     price: 10000; }
  }
}
"#;
    let codes = error_codes(src);
    assert!(
        codes.iter().any(|c| c == "INAH-2"),
        "deferred-in + spot-out of the same asset must be refused as monetization; got {:?}",
        codes
    );
}

#[test]
fn distinct_assets_do_not_false_positive() {
    // Two unrelated deferred sales of DIFFERENT assets, no buy-back: must compile clean.
    let src = r#"
bundle TwoSales {
  meta { basis: "two independent murabahah sales"; regime: islamic; }
  parties { bank: financier; customer: client; supplier: vendor; }
  legs {
    a: wakalah   { from: supplier; to: bank;     asset: car;   payment: spot;     price: 9000; }
    b: murabahah { from: bank;     to: customer; asset: car;   payment: deferred; price: 9900; }
    c: wakalah   { from: supplier; to: bank;     asset: house; payment: spot;     price: 50000; }
    d: murabahah { from: bank;     to: customer; asset: house; payment: deferred; price: 55000; }
  }
}
"#;
    let codes = error_codes(src);
    assert!(
        codes.is_empty(),
        "independent acyclic sales of different assets must compile; got {:?}",
        codes
    );
}
