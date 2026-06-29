//! Sukuk — the first multilateral instrument. The engine accepts an asset-backed, pro-rata sukuk
//! over a holder pool, and refuses a bond (fixed coupon on principal) or a pool not summing to 100%.

use fiqhc::sema::Diagnostic;
use fiqhc::{codegen, compile_check, compile_parse};

fn errors(src: &str) -> Vec<Diagnostic> {
    let (_spec, d) = compile_check(src).expect("parses");
    d.into_iter().filter(|x| x.is_error()).collect()
}

#[test]
fn valid_sukuk_compiles_and_lowers_with_pool() {
    let src = include_str!("../../../specs/sukuk.fiqh");
    assert!(errors(src).is_empty(), "valid sukuk must compile: {:?}", errors(src).iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    let spec = compile_parse(src).expect("parses");
    let g = codegen::generate(&spec).expect("lowers");
    assert!(g.sol.contains("address[] public holders"), "must emit the holder pool (multilateral array)");
    assert!(g.sol.contains("ownership shares must total 10000 bps"), "must guard the pool sum on-chain");
}

#[test]
fn bond_sukuk_is_refused() {
    let errs = errors(include_str!("../../../specs/sukuk_riba.fiqh"));
    assert!(errs.iter().any(|e| e.code == "SUKUK-1"), "a bond return => SUKUK-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
    assert!(errs.iter().any(|e| e.code == "SUKUK-2"), "a fixed coupon => SUKUK-2");
}

#[test]
fn pool_not_totalling_is_refused() {
    let errs = errors(include_str!("../../../specs/sukuk_badpool.fiqh"));
    assert!(errs.iter().any(|e| e.code == "POOL-1"), "a pool not summing to 10000 => POOL-1; got {:?}", errs.iter().map(|e| e.code.clone()).collect::<Vec<_>>());
}
