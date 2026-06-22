//! Faraid — the Islamic law of inheritance (mawarith), as a deterministic engine.
//!
//! When a contracting entity dissolves by the death of a partner, the deceased's estate is
//! not distributed by will or by the surviving partners' discretion — it passes by the fixed
//! shares (al-furud al-muqaddara) that Allah Himself apportioned in the Qur'an (al-Nisa'
//! 4:11-12 and 4:176). This module computes those shares exactly, so a contract can call it
//! as an off-ramp: on death, liquidate and distribute algorithmically to the rightful heirs.
//!
//! Scope (the core that covers the great majority of estates): spouse, father, mother, sons
//! and daughters — with the residuary ('asaba) mechanism, the 2:1 male-to-female apportion
//! among children, and the two corrections 'awl (when the fixed shares overflow unity) and
//! radd (when they fall short and there is no residuary). Collaterals (siblings, grandparents)
//! and the hajb (occlusion) rules among them are deliberately OUT of this core; an estate with
//! such heirs must go to a faradi (specialist) — the engine says so rather than guessing.
//!
//! Epistemics: 'ilm al-fara'id is among the most precise and agreed-upon sciences of the
//! Shari'ah; the furud here are the muhkam of the Book. Where a question is khilafi (notably
//! radd to a lone spouse) the code follows the majority and the comment flags it. This is
//! computation over a settled rule-base, not a fatwa on a specific estate. Allahu a'lam.

/// Which spouse survives (if any). Multiple wives share the single spousal fraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spouse {
    Husband,
    Wife,
    None,
}

/// The surviving heirs from the supported core.
#[derive(Debug, Clone, Copy)]
pub struct Heirs {
    pub spouse: Spouse,
    pub father: bool,
    pub mother: bool,
    pub sons: u32,
    pub daughters: u32,
}

/// One heir category and its share, in basis points of the estate. Categories with multiple
/// members (sons/daughters) carry the category total; the per-head split is the total divided
/// by the count (the 2:1 ratio is already applied between sons and daughters).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    pub heir: String,
    pub bps: u64,
}

/// A non-negative fraction over u128, kept reduced.
#[derive(Debug, Clone, Copy)]
struct Frac {
    num: u128,
    den: u128,
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

impl Frac {
    fn new(num: u128, den: u128) -> Frac {
        let g = gcd(num.max(1), den).max(1);
        Frac { num: num / g, den: den / g }
    }
    fn zero() -> Frac {
        Frac { num: 0, den: 1 }
    }
    fn add(self, o: Frac) -> Frac {
        Frac::new(self.num * o.den + o.num * self.den, self.den * o.den)
    }
    fn sub(self, o: Frac) -> Frac {
        // assumes self >= o
        Frac::new(self.num * o.den - o.num * self.den, self.den * o.den)
    }
    fn mul(self, o: Frac) -> Frac {
        Frac::new(self.num * o.num, self.den * o.den)
    }
    /// self <=> 1
    fn cmp_one(self) -> std::cmp::Ordering {
        self.num.cmp(&self.den)
    }
    fn to_bps(self) -> u128 {
        // round to nearest
        (self.num * 10_000 + self.den / 2) / self.den
    }
}

/// Compute the faraid distribution. Returns the per-category shares (summing to 10000 bps), or
/// an error string if the estate contains heirs this core does not model.
pub fn distribute(h: &Heirs) -> Result<Vec<Share>, String> {
    let has_descendant = h.sons > 0 || h.daughters > 0;

    // Collect fixed-share (fard) entries and, separately, the residuary ('asaba) weights.
    // Each entry is (label, Frac). Residuary entries are (label, weight) and split the residue.
    let mut fard: Vec<(String, Frac)> = Vec::new();
    let mut asaba: Vec<(String, u128)> = Vec::new();

    // --- spouse ---
    match h.spouse {
        Spouse::Husband => {
            fard.push(("husband".into(), if has_descendant { Frac::new(1, 4) } else { Frac::new(1, 2) }));
        }
        Spouse::Wife => {
            fard.push(("wife".into(), if has_descendant { Frac::new(1, 8) } else { Frac::new(1, 4) }));
        }
        Spouse::None => {}
    }

    // --- mother ---
    // 1/6 with a descendant (this core has no sibling count, which would also trigger 1/6);
    // otherwise 1/3. (The 1/3-of-remainder 'Umariyyatan cases involve a spouse+both parents and
    // no descendant; flagged below.)
    if h.mother {
        fard.push(("mother".into(), if has_descendant { Frac::new(1, 6) } else { Frac::new(1, 3) }));
    }

    // --- father ---
    // With a descendant: 1/6 (and the residue too, if there are no sons — father is then both a
    // sharer and the residuary). Without a descendant: pure residuary.
    if h.father {
        if has_descendant {
            fard.push(("father".into(), Frac::new(1, 6)));
            if h.sons == 0 {
                asaba.push(("father".into(), 0)); // marker; father takes whatever residue remains
            }
        } else {
            asaba.push(("father".into(), 1));
        }
    }

    // --- children ---
    if h.sons > 0 {
        // sons (and daughters with them) are 'asaba bil-ghayr at 2:1.
        asaba.push(("sons".into(), 2 * h.sons as u128));
        if h.daughters > 0 {
            asaba.push(("daughters".into(), h.daughters as u128));
        }
    } else if h.daughters > 0 {
        // daughters alone take a fixed share: one => 1/2, two or more => 2/3.
        let f = if h.daughters == 1 { Frac::new(1, 2) } else { Frac::new(2, 3) };
        fard.push(("daughters".into(), f));
    }

    if fard.is_empty() && asaba.is_empty() {
        return Err("no surviving heirs from the supported core; refer to a faradi".into());
    }

    // Sum the fixed shares.
    let sum_fard = fard.iter().fold(Frac::zero(), |acc, (_, f)| acc.add(*f));

    // 'Umariyyatan guard: spouse + both parents + no descendant gives the mother 1/3 of the
    // REMAINDER (not of the whole), a special case this core does not encode. Refuse rather
    // than mis-distribute.
    if !has_descendant
        && h.father
        && h.mother
        && matches!(h.spouse, Spouse::Husband | Spouse::Wife)
    {
        return Err("'Umariyyatayn case (spouse + both parents, no descendant): mother takes 1/3 of the remainder — refer to a faradi".into());
    }

    let mut out: Vec<(String, Frac)> = Vec::new();

    // Does a true residuary exist? (father-with-only-daughters marker has weight 0 but is still
    // a residuary that mops up any positive remainder.)
    let has_asaba = !asaba.is_empty();

    match sum_fard.cmp_one() {
        std::cmp::Ordering::Greater => {
            // 'AWL: the fixed shares overflow unity. Scale every share by 1/sum_fard so they
            // sum to one; the residuary (if any) is occluded to nothing.
            for (label, f) in &fard {
                out.push((label.clone(), f.mul(Frac::new(sum_fard.den, sum_fard.num))));
            }
        }
        std::cmp::Ordering::Equal => {
            for (label, f) in &fard {
                out.push((label.clone(), *f));
            }
            // residue is zero; asaba get nothing.
        }
        std::cmp::Ordering::Less => {
            let residue = Frac::new(1, 1).sub(sum_fard);
            if has_asaba {
                // Give the residue to the residuaries by weight. The father-only-daughters
                // marker (weight 0) absorbs the residue when it is the sole residuary.
                let total_weight: u128 = asaba.iter().map(|(_, w)| *w).sum();
                for (label, f) in &fard {
                    out.push((label.clone(), *f));
                }
                if total_weight == 0 {
                    // sole residuary is the father (with only daughters): he takes all residue,
                    // folded into his existing fard line.
                    if let Some(slot) = out.iter_mut().find(|(l, _)| l == "father") {
                        slot.1 = slot.1.add(residue);
                    }
                } else {
                    for (label, w) in &asaba {
                        if *w == 0 {
                            continue;
                        }
                        let part = residue.mul(Frac::new(*w, total_weight));
                        if let Some(slot) = out.iter_mut().find(|(l, _)| l == label) {
                            slot.1 = slot.1.add(part);
                        } else {
                            out.push((label.clone(), part));
                        }
                    }
                }
            } else {
                // RADD: no residuary. Return the surplus proportionally to the fixed-share
                // heirs OTHER than the spouse (majority view). If the spouse is the only heir,
                // the spouse takes the surplus (khilaf — many contemporary bodies, e.g. via
                // bayt al-mal's cession, allow radd to a lone spouse).
                let spouse_label = match h.spouse {
                    Spouse::Husband => Some("husband"),
                    Spouse::Wife => Some("wife"),
                    Spouse::None => None,
                };
                let radd_base: Frac = fard
                    .iter()
                    .filter(|(l, _)| Some(l.as_str()) != spouse_label)
                    .fold(Frac::zero(), |acc, (_, f)| acc.add(*f));
                let non_spouse_exists = radd_base.num > 0;
                for (label, f) in &fard {
                    if Some(label.as_str()) == spouse_label && non_spouse_exists {
                        out.push((label.clone(), *f)); // spouse: fard only, no radd
                    } else if non_spouse_exists {
                        // proportional radd: f + residue * (f / radd_base)
                        let extra = residue.mul(f.mul(Frac::new(radd_base.den, radd_base.num)));
                        out.push((label.clone(), f.add(extra)));
                    } else {
                        // lone spouse (or only-spouse fard set): takes everything.
                        out.push((label.clone(), f.add(residue)));
                    }
                }
            }
        }
    }

    // Convert to bps with a largest-remainder fix so the total is exactly 10000.
    let mut shares: Vec<Share> = out
        .iter()
        .map(|(l, f)| Share { heir: l.clone(), bps: f.to_bps() as u64 })
        .collect();
    let total: u64 = shares.iter().map(|s| s.bps).sum();
    if total != 10_000 && !shares.is_empty() {
        // adjust the largest share by the (small, rounding) difference.
        let diff = 10_000i64 - total as i64;
        let idx = shares
            .iter()
            .enumerate()
            .max_by_key(|(_, s)| s.bps)
            .map(|(i, _)| i)
            .unwrap();
        shares[idx].bps = (shares[idx].bps as i64 + diff).max(0) as u64;
    }
    Ok(shares)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bps_of(shares: &[Share], heir: &str) -> u64 {
        shares.iter().find(|s| s.heir == heir).map(|s| s.bps).unwrap_or(0)
    }
    fn total(shares: &[Share]) -> u64 {
        shares.iter().map(|s| s.bps).sum()
    }

    #[test]
    fn son_and_daughter_two_to_one() {
        let h = Heirs { spouse: Spouse::None, father: false, mother: false, sons: 1, daughters: 1 };
        let s = distribute(&h).unwrap();
        assert_eq!(bps_of(&s, "sons"), 6667); // 2/3, with rounding fix
        assert_eq!(bps_of(&s, "daughters"), 3333);
        assert_eq!(total(&s), 10_000);
    }

    #[test]
    fn husband_and_one_son() {
        let h = Heirs { spouse: Spouse::Husband, father: false, mother: false, sons: 1, daughters: 0 };
        let s = distribute(&h).unwrap();
        assert_eq!(bps_of(&s, "husband"), 2500); // 1/4 with descendant
        assert_eq!(bps_of(&s, "sons"), 7500); // residue
        assert_eq!(total(&s), 10_000);
    }

    #[test]
    fn wife_and_one_son() {
        let h = Heirs { spouse: Spouse::Wife, father: false, mother: false, sons: 1, daughters: 0 };
        let s = distribute(&h).unwrap();
        assert_eq!(bps_of(&s, "wife"), 1250); // 1/8 with descendant
        assert_eq!(bps_of(&s, "sons"), 8750);
        assert_eq!(total(&s), 10_000);
    }

    #[test]
    fn father_mother_and_a_son() {
        let h = Heirs { spouse: Spouse::None, father: true, mother: true, sons: 1, daughters: 0 };
        let s = distribute(&h).unwrap();
        assert_eq!(bps_of(&s, "father"), 1667); // 1/6
        assert_eq!(bps_of(&s, "mother"), 1667); // 1/6
        assert_eq!(bps_of(&s, "sons"), 6666); // residue (rounding-fixed)
        assert_eq!(total(&s), 10_000);
    }

    #[test]
    fn awl_minbariyyah_like() {
        // wife 1/8, two daughters 2/3, father 1/6, mother 1/6 -> sum 27/24 -> 'awl to 27.
        let h = Heirs { spouse: Spouse::Wife, father: true, mother: true, sons: 0, daughters: 2 };
        let s = distribute(&h).unwrap();
        // base 'awls from 24 to 27: wife 3/27, daughters 16/27, father 4/27, mother 4/27.
        // In bps (largest-remainder fix lands the spare basis point on the daughters):
        assert_eq!(bps_of(&s, "wife"), 1111); // 3/27
        assert_eq!(bps_of(&s, "mother"), 1481); // 4/27
        assert_eq!(bps_of(&s, "father"), 1481); // 4/27
        assert_eq!(bps_of(&s, "daughters"), 5927); // 16/27, rounding-fixed
        assert_eq!(total(&s), 10_000);
    }

    #[test]
    fn radd_daughter_and_mother() {
        // daughter 1/2, mother 1/6, no residuary -> radd in ratio 3:1 -> daughter 3/4, mother 1/4.
        let h = Heirs { spouse: Spouse::None, father: false, mother: true, sons: 0, daughters: 1 };
        let s = distribute(&h).unwrap();
        assert_eq!(bps_of(&s, "daughters"), 7500);
        assert_eq!(bps_of(&s, "mother"), 2500);
        assert_eq!(total(&s), 10_000);
    }

    #[test]
    fn radd_excludes_spouse() {
        // wife 1/8, daughter 1/2 (+ radd of the rest to the daughter, not the wife).
        // fard: wife 1/8, daughter 1/2; sum = 5/8; residue 3/8 -> all to daughter (only non-spouse).
        let h = Heirs { spouse: Spouse::Wife, father: false, mother: false, sons: 0, daughters: 1 };
        let s = distribute(&h).unwrap();
        assert_eq!(bps_of(&s, "wife"), 1250); // 1/8, no radd
        assert_eq!(bps_of(&s, "daughters"), 8750); // 1/2 + 3/8
        assert_eq!(total(&s), 10_000);
    }

    #[test]
    fn umariyyatayn_is_refused() {
        // husband + father + mother, no descendant: the special 1/3-of-remainder case.
        let h = Heirs { spouse: Spouse::Husband, father: true, mother: true, sons: 0, daughters: 0 };
        assert!(distribute(&h).is_err());
    }

    #[test]
    fn empty_estate_is_refused() {
        let h = Heirs { spouse: Spouse::None, father: false, mother: false, sons: 0, daughters: 0 };
        assert!(distribute(&h).is_err());
    }
}
