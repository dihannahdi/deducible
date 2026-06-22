//! Open Core pillar #4 (robustness): the front-end + engine never panic on malformed input.

use fiqhc::fuzz;

const SEEDS: &[&str] = &[
    include_str!("../../../specs/musharakah_mutanaqisah.fiqh"),
    include_str!("../../../specs/mudarabah.fiqh"),
    include_str!("../../../specs/commercial_escrow.fiqh"),
    include_str!("../../../specs/riba_disguised.fiqh"),
];

#[test]
fn engine_never_panics_on_fuzzed_input() {
    let found = fuzz::run(30_000, SEEDS);
    assert!(found.is_none(), "engine panicked on input:\n{}", found.unwrap_or_default());
}
