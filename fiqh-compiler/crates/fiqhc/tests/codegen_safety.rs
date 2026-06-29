//! Open Core pillar #4 (codegen proofs): every generated contract guards value movement against
//! reentrancy, defines role modifiers, and compiles in its declared invariants. A static safety
//! property over the emitted Solidity — a step toward the mathematical guarantees a global
//! standard demands (full formal verification is future work).

use fiqhc::{codegen, compile_check};

const SPECS: &[&str] = &[
    include_str!("../../../specs/musharakah_mutanaqisah.fiqh"),
    include_str!("../../../specs/mudarabah.fiqh"),
    include_str!("../../../specs/ijarah_imbt.fiqh"),
    include_str!("../../../specs/murabahah.fiqh"),
    include_str!("../../../specs/salam.fiqh"),
    include_str!("../../../specs/istisna.fiqh"),
    include_str!("../../../specs/sarf.fiqh"),
    include_str!("../../../specs/tawarruq.fiqh"),
    include_str!("../../../specs/qard_hasan.fiqh"),
    include_str!("../../../specs/rahn.fiqh"),
    include_str!("../../../specs/kafala.fiqh"),
    include_str!("../../../specs/hawala.fiqh"),
    include_str!("../../../specs/wadia.fiqh"),
    include_str!("../../../specs/wakala.fiqh"),
    include_str!("../../../specs/commercial_escrow.fiqh"),
];

/// Every EXTERNAL/PUBLIC function that moves value (`.call{value:`) must carry `nonReentrant`.
/// Internal helpers are exempt — they are only reachable through guarded external entry points.
fn external_value_calls_are_guarded(sol: &str) -> Result<(), String> {
    let mut sig = String::new();
    for line in sol.lines() {
        let t = line.trim();
        if t.starts_with("function ") {
            sig = t.to_string();
        }
        if t.contains(".call{value:") {
            let external = sig.contains("external") || sig.contains("public");
            if external && !sig.contains("nonReentrant") {
                return Err(format!("unguarded external value-call in: {}", sig));
            }
        }
    }
    Ok(())
}

#[test]
fn generated_contracts_are_structurally_safe() {
    for src in SPECS {
        let (spec, diags) = compile_check(src).expect("parses");
        assert!(diags.iter().all(|d| !d.is_error()), "{} should be consistent", spec.name);
        let g = codegen::generate(&spec).expect("lowers");

        external_value_calls_are_guarded(&g.sol)
            .unwrap_or_else(|e| panic!("{}: {}", g.contract_name, e));
        assert!(g.sol.contains("modifier nonReentrant"), "{}: must define a reentrancy guard", g.contract_name);
        assert!(g.sol.contains("modifier only"), "{}: must define role modifiers", g.contract_name);
        assert!(g.sol.contains("@dev INVARIANT"), "{}: must compile in declared invariants", g.contract_name);
        assert!(g.sol.contains("pragma solidity ^0.8.24"), "{}: pinned pragma", g.contract_name);
    }
}
