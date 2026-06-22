//! Vector 1 — Zero-Knowledge Fiqh: prove loss-proportionality without revealing the amounts.
//!
//! Two layers: a self-contained sigma-protocol PoC (the `zk` module) and the production Circom
//! circuit emitted by codegen. These integration tests cover the roundtrip + the codegen output.

use fiqhc::{codegen, compile_parse, zk};

#[test]
fn sigma_proof_roundtrip() {
    // honest: 8000:2000 ownership, 800:200 loss is proportional.
    let ok = zk::prove_proportional_loss(8000, 2000, 800, 200);
    assert!(zk::verify_proportional_loss(&ok), "an honest proportional loss must verify");
    // dishonest: 800:300 is not proportional to 8000:2000.
    let bad = zk::prove_proportional_loss(8000, 2000, 800, 300);
    assert!(!zk::verify_proportional_loss(&bad), "a disproportionate loss must NOT verify");
}

#[test]
fn proof_carries_no_amounts() {
    // The proof struct exposes only commitments and the Schnorr transcript — never the losses.
    let p = zk::prove_proportional_loss(8000, 2000, 800, 200);
    // sanity: the public fields are the bps and opaque group elements.
    assert_eq!(p.bank_bps, 8000);
    assert_eq!(p.client_bps, 2000);
    assert!(p.cb != 0 && p.cc != 0 && p.t != 0);
}

#[test]
fn codegen_emits_circuit_and_verifier_gate() {
    let src = include_str!("../../../specs/musharakah_mutanaqisah.fiqh");
    let spec = compile_parse(src).expect("spec should parse");
    let zkc = codegen::build_zk(&spec);

    assert!(zkc.circom.contains("lossBank * clientBps"), "circuit must encode proportional loss");
    assert!(zkc.circom.contains("bankBps + clientBps === 10000"), "circuit must conserve ownership");
    assert!(zkc.circom.contains("private witness"), "loss amounts must be private signals");
    assert!(zkc.manifest.contains("\"scheme\": \"groth16\""), "manifest must declare the scheme");
    assert!(zkc.manifest.contains("RISK-1"), "manifest must cite the loss-sharing invariant");
    assert!(zkc.verifier_consumer.contains("settleWithProof"), "must emit a verifier gate");
    assert!(zkc.verifier_consumer.contains("[RISK-1] zk proof invalid"), "gate must cite RISK-1");
}
