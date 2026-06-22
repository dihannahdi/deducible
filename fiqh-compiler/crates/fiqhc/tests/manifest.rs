//! V4-A gate: the portable invariant manifest carries machine-checkable constraints derived
//! from the same facts the engine checks — so invariants can be injected into any backend.

use fiqhc::{codegen, compile_check};

const MMP: &str = include_str!("../../../specs/musharakah_mutanaqisah.fiqh");
const ESCROW: &str = include_str!("../../../specs/commercial_escrow.fiqh");

#[test]
fn manifest_carries_machine_checkable_constraints() {
    let (spec, _) = compile_check(MMP).unwrap();
    let m = codegen::build_manifest(&spec);
    assert!(m.contains("RIBA-1"), "manifest must encode the riba constraint");
    assert!(m.contains("risk.capital_guarantee"));
    assert!(m.contains("proportional_to_ownership"));
    assert!(m.contains("\"op\""), "constraints must carry an operator");

    let (e, _) = compile_check(ESCROW).unwrap();
    let me = codegen::build_manifest(&e);
    assert!(me.contains("PENALTY-1"), "common-law penalty doctrine in the manifest");
    assert!(me.contains("returns.release.damages"));
    assert!(me.contains("common_law"), "manifest records the regime");
}
