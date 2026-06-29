//! The maqasid / hiyal-risk surfacing layer.
//!
//! IMPORTANT (epistemics): this layer NEVER rules and NEVER emits an error. It raises only
//! WARNINGS that flag a form-compliant contract whose *maqsad* (purpose/intent) a qualified
//! scholar should examine — the substance-over-form question that a syntactic rule-base cannot
//! settle. The engine polices FORM; here it merely *surfaces* where form may diverge from purpose
//! (the classic site of hiyal, legal stratagems). This is the deliberate ceiling described in
//! Paper III: a riba spec won't compile, but a formally-valid murabaha that is in substance a
//! circumvention of riba (tahayul al-murabaha 'ala al-riba) is for the faqih to judge, not the
//! compiler. cf. al-Shatibi, al-Muwafaqat; Ibn al-Qayyim, I'lam al-Muwaqqi'in (on the hiyal).
//! [scholar-verify]

use crate::ast::Spec;

/// Surface maqsad-risk warnings for a spec. Returns `(code, message)` pairs; the caller wraps each
/// as a `Diagnostic::warn` (never an error). An empty vector means no maqsad smell was detected —
/// which is NOT a ruling that the maqsad is sound, only that no known pattern matched.
pub fn surface(spec: &Spec) -> Vec<(&'static str, String)> {
    let mut w = Vec::new();
    match spec.class.as_str() {
        // Sale-based financing is the historic home of circumvention. Each leg can be valid while
        // the maqsad is pure cash-financing dressed as trade.
        "tawarruq" => w.push((
            "MAQASID-1",
            "tawarruq is a recognised site of circumvention: even when each leg is valid and the \
             onward sale is to a genuine third party, the maqsad may be pure cash-financing dressed \
             as trade. A scholar should confirm a real need for the commodity and that this is not \
             organised tawarruq (munazzam) in substance.".to_string(),
        )),
        "murabahah" => w.push((
            "MAQASID-1",
            "murabaha can become tahayul al-murabaha 'ala al-riba (a circumvention of riba) where \
             the markup merely tracks a prevailing interest rate. A scholar should confirm the buyer \
             genuinely wanted the good and that the markup is a real trade profit, not interest by \
             another name.".to_string(),
        )),
        // A diminishing partnership that also charges rent can drift toward a disguised loan if the
        // rent is set to amortise the financier's capital rather than price the usufruct.
        "musharakah_mutanaqisah" => w.push((
            "MAQASID-2",
            "musharakah mutanaqisah is sound by construction here, but its maqsad should be checked: \
             if the rent is, in substance, calibrated to return the financier's capital with a fixed \
             yield (rather than to price the living share's usufruct), the partnership drifts toward \
             a disguised interest-bearing loan. A scholar should review the rent's economic basis.".to_string(),
        )),
        _ => {}
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Spec, Span};

    fn spec_of(class: &str) -> Spec {
        Spec { name: "T".to_string(), class: class.to_string(), sections: vec![], span: Span::new(0, 0) }
    }

    #[test]
    fn flags_known_hiyal_sites_only() {
        assert_eq!(surface(&spec_of("murabahah")).len(), 1);
        assert_eq!(surface(&spec_of("tawarruq")).len(), 1);
        assert_eq!(surface(&spec_of("musharakah_mutanaqisah")).len(), 1);
        // a plain lease has no known circumvention pattern -> no warning (NOT a ruling of soundness)
        assert_eq!(surface(&spec_of("ijarah")).len(), 0);
        assert_eq!(surface(&spec_of("qard_hasan")).len(), 0);
    }
}
