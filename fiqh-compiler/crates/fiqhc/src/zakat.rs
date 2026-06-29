//! Zakat al-Tijarah — the algorithmic zakat layer (enterprise vector #5).
//!
//! Every Muslim-owned commercial entity owes zakat on its trade goods ('urud al-tijarah)
//! once two conditions are met: the wealth has been held for a full lunar year (haul) and
//! it reaches the threshold (nisab). The rate is fixed at rubʿ al-ʿushr — one fortieth,
//! 2.5%. Rather than leave this to a year-end accountant (and to evasion), `fiqhc` lifts it
//! into the contract itself: the generated instrument routes 2.5% of the qualifying base to
//! a maslahah / zakat fund, on-chain, before the partners take their share. Corporate zakat
//! becomes a property of the code, not an act of conscience.
//!
//! Fiqh basis (flagged for human takhrij):
//!   - Rate 1/40 = 2.5% on trade goods — agreed across the four madhahib (rubʿ al-ʿushr).
//!   - Nisab = the value of 85g of gold (20 mithqal) or 595g of silver (200 dirham); the
//!     silver nisab is the lower and is often preferred so more reaches the poor.
//!   - Haul = a full HIJRI (lunar ≈ 354-day) year — not a solar year; a solar haul would
//!     under-collect by ~11 days a year and is an error of method, not merely of rounding.
//!   - 'urud al-tijarah are zakatable: al-Tawbah 9:103; the athar of Samurah b. Jundub
//!     (Sunan Abi Dawud) that the Prophet ﷺ ordered sadaqah on what is prepared for sale;
//!     AAOIFI Shari'ah Standard No. 35 (Zakah). [scholar-verify]
//!
//! Epistemics unchanged: the engine computes a quantity from a declared, citation-bearing
//! rule. The fiqh of the zakatable BASE (what counts as liquid trade wealth, the treatment
//! of debts and fixed assets) is the scholar's to define; the engine only applies it.

/// Rubʿ al-ʿushr — one fortieth — expressed in basis points. The single rate for trade goods
/// (and, by the same measure, for gold, silver, currency, salaries and rented assets).
pub const RATE_BPS_TIJARAH: u64 = 250; // 2.5%

/// ʿushr — one tenth — on rain-fed / unirrigated produce.
pub const RATE_BPS_USHR: u64 = 1000; // 10%

/// Niṣf al-ʿushr — one twentieth — on produce watered by effort/irrigation (the cost halves the due).
pub const RATE_BPS_NISF_USHR: u64 = 500; // 5%

/// Basis-point denominator.
pub const BPS: u64 = 10_000;

/// The expected zakat rate (in bps) for a declared `kind` of zakatable wealth, or `None` if the
/// kind is not one the engine knows how to rate.
///
/// The rubʿ al-ʿushr genera — trade goods, gold, silver, currency, salaries (māl mustafād) and
/// rented/industrial assets (mustaghallāt) — all carry 1/40 (2.5%). Produce carries ʿushr
/// (10% if rain-fed) or niṣf al-ʿushr (5% if irrigated by effort/cost) — Sahih al-Bukhari, the
/// hadith of Ibn ʿUmar; due at HARVEST (al-Anʿam 6:141), not at a lunar haul. Livestock (per-head
/// tables) does not fit a percentage rate and is handled separately. [scholar-verify]
pub fn rate_for_kind(kind: &str) -> Option<u64> {
    match kind {
        "tijarah" | "gold" | "silver" | "currency" | "salary" | "mustaghallat" => Some(RATE_BPS_TIJARAH),
        "crops_rain" => Some(RATE_BPS_USHR),
        "crops_irrigated" => Some(RATE_BPS_NISF_USHR),
        _ => None,
    }
}

/// Is `kind` a zakatable genus the engine can rate? (Produce kinds are due at harvest, not a haul.)
pub fn is_known_kind(kind: &str) -> bool {
    rate_for_kind(kind).is_some()
}

/// Produce (ʿushr/niṣf al-ʿushr) is due at HARVEST, with no lunar haul.
pub fn is_produce_kind(kind: &str) -> bool {
    matches!(kind, "crops_rain" | "crops_irrigated")
}

/// Compute the zakat due on a zakatable `base`, given the `nisab` threshold and the rate in
/// basis points. Below nisab nothing is due; at or above it, `rate_bps`/10000 of the base.
///
/// Integer arithmetic mirrors the on-chain computation exactly (no floating point), so the
/// Rust result and the generated Solidity agree to the tinybar.
pub fn zakat_due(base: u64, nisab: u64, rate_bps: u64) -> u64 {
    if base < nisab {
        return 0;
    }
    // base * rate_bps / BPS, widened to u128 to avoid overflow on large bases.
    ((base as u128 * rate_bps as u128) / BPS as u128) as u64
}

/// Is this the agreed trade-goods rate (1/40)?
pub fn is_tijarah_rate(rate_bps: u64) -> bool {
    rate_bps == RATE_BPS_TIJARAH
}

/// A haul declaration is sound only if it is a lunar (hijri) year. The accepted spellings.
pub fn is_lunar_haul(haul: &str) -> bool {
    matches!(haul, "hijri_year" | "lunar_year" | "qamari_year")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_due_below_nisab() {
        assert_eq!(zakat_due(100, 850, RATE_BPS_TIJARAH), 0);
        assert_eq!(zakat_due(849, 850, RATE_BPS_TIJARAH), 0);
    }

    #[test]
    fn exactly_one_fortieth_at_and_above_nisab() {
        // at nisab
        assert_eq!(zakat_due(850, 850, RATE_BPS_TIJARAH), 850 * 250 / 10_000);
        // a round figure: 1,000,000 -> 25,000 (exactly 2.5% = 1/40)
        assert_eq!(zakat_due(1_000_000, 850, RATE_BPS_TIJARAH), 25_000);
        assert_eq!(zakat_due(1_000_000, 850, RATE_BPS_TIJARAH), 1_000_000 / 40);
    }

    #[test]
    fn no_overflow_on_large_base() {
        // a base near u64::MAX must not overflow the multiply (u128 widening).
        let big = 1_000_000_000_000_000_000u64; // 1e18
        assert_eq!(zakat_due(big, 1, RATE_BPS_TIJARAH), big / 40);
    }

    #[test]
    fn rate_and_haul_predicates() {
        assert!(is_tijarah_rate(250));
        assert!(!is_tijarah_rate(200));
        assert!(is_lunar_haul("hijri_year"));
        assert!(!is_lunar_haul("gregorian_year"));
        assert!(!is_lunar_haul("solar_year"));
    }

    #[test]
    fn rate_for_each_known_genus() {
        for g in ["tijarah", "gold", "silver", "currency", "salary", "mustaghallat"] {
            assert_eq!(rate_for_kind(g), Some(250), "{} is rub' al-'ushr", g);
        }
        assert_eq!(rate_for_kind("crops_rain"), Some(1000));
        assert_eq!(rate_for_kind("crops_irrigated"), Some(500));
        assert_eq!(rate_for_kind("livestock"), None);
        assert_eq!(rate_for_kind("nonsense"), None);
        assert!(is_produce_kind("crops_rain") && is_produce_kind("crops_irrigated"));
        assert!(!is_produce_kind("gold"));
    }
}
