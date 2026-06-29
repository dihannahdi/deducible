//! The fiqh invariant engine — semantic analysis of a `.fiqh` specification.
//!
//! IMPORTANT (epistemics): this engine issues no fatwa. It proves a specification
//! is *consistent or inconsistent with a declared, human-authored, citation-bearing
//! rule-base*. The fiqh validity of the rule-base itself — and of the instrument,
//! which may carry khilaf — remains a qualified scholar's domain. Every citation is
//! flagged `[scholar-verify]`. The contribution is the separation of the rule-base
//! (fiqh, human-ratified) from the enforcement engine (mechanical, sound):
//! a contract whose declared economics contradict its declared basis cannot be
//! lowered to Solidity. Compliance becomes a property of the language.

use crate::ast::*;
use serde_json::Value;

// --- citations (all flagged for human takhrij) ---
const C_RIBA: &str = "Qur'an al-Baqarah 2:275; AAOIFI Shari'ah Standard No. 12 [scholar-verify]";
const C_RISK: &str = "a partnership shares profit AND loss — AAOIFI SS No. 12 [scholar-verify]";
const C_GHARAR: &str = "prohibition of gharar (Sahih Muslim, Kitab al-Buyu') [scholar-verify]";
const C_MUDARABAH: &str = "AAOIFI Shari'ah Standard No. 13 (Mudarabah) [scholar-verify]";
const C_IJARAH: &str = "AAOIFI Shari'ah Standard No. 9 (Ijarah) [scholar-verify]";
const C_MURABAHA: &str = "murabaha (cost-plus trust sale, bay' al-amana): AAOIFI Shari'ah Standard No. 8; the buyer must know the true cost; the seller must possess the good first — 'do not sell what you do not have' (Hakim b. Hizam — Sunan Abi Dawud, al-Tirmidhi, al-Nasa'i); al-Baqarah 2:275 (Allah permitted trade and forbade riba) [scholar-verify]";
const C_SALAM: &str = "salam (forward sale): AAOIFI Shari'ah Standard No. 10; the Prophet ﷺ — of one who pays in advance — said let it be for a known measure and known weight to a known term (Ibn ʿAbbas — Sahih al-Bukhari, Sahih Muslim); the full price (ra's al-mal) must be paid at the session, else it is bayʿ al-kaliʾ bi-l-kaliʾ (debt for debt) [scholar-verify]";
const C_ISTISNA: &str = "istisnaʿ (manufacture-to-order): AAOIFI Shari'ah Standard No. 11; held permissible by istihsan (the Hanafis) and adopted broadly — the masnuʿ must be described (maʿlūm) and the saniʿ supplies the materials (else it is ijarat al-ʿamal, hire of labour); unlike salam, the price MAY be deferred or paid in instalments [scholar-verify]";
const C_SARF: &str = "ṣarf (currency/precious-metal exchange): AAOIFI Shari'ah Standard No. 1; the Prophet ﷺ — gold for gold, silver for silver... like for like, equal for equal, hand to hand; if the genera differ, sell as you wish so long as it is hand to hand (ʿUbada b. al-Ṣamit — Sahih Muslim). Same genus ⇒ equal (else riba al-faḍl); any ṣarf ⇒ spot/yadan bi-yad (else riba al-nasiʾa) [scholar-verify]";
const C_WAKALA: &str = "wakala (agency): AAOIFI Shari'ah Standard No. 23; the agent (wakil) acts on the principal's account and is a trustee (amīn) — he does NOT guarantee the capital or the profit (a guarantee by the agent converts the agency into a guaranteed loan, i.e. riba); his compensation is a KNOWN fee (ujra), free of gharar. In wakala bi-l-istithmar the realized return belongs to the principal [scholar-verify]";
const C_WADIA: &str = "wadiʿa (safekeeping deposit): al-Nisaʾ 4:58 ('render the trusts to their owners') and al-Baqarah 2:283; the deposit is a trust (yad amāna) — the custodian is NOT liable for its loss absent taʿaddī (transgression) or taqṣīr (negligence), and may not use it. If the custodian guarantees its return regardless, or uses it, the contract becomes a loan (qarḍ) and any conditioned benefit is riba. A fee for the safekeeping SERVICE itself is permitted (wadiʿa bi-l-ujra) [scholar-verify]";
const C_HAWALA: &str = "hawala (assignment/transfer of debt): the Prophet ﷺ — 'procrastination by a rich man is injustice; and when one of you is referred to a solvent man, let him follow/accept' (Sahih al-Bukhari, Sahih Muslim). The transfer is for the LIKE of the debt — no increase (an increase would be riba / sale of debt) — and a valid hawala DISCHARGES the original debtor (al-muhil) toward the creditor [scholar-verify]";
const C_KAFALA: &str = "kafala / ḍamān (suretyship): AAOIFI Shari'ah Standard No. 5; the guarantee is a gratuitous undertaking (tabarruʿ) — the majority forbid taking a FEE for a guarantee, since it is a benefit on lending one's credit and a paid guarantee can mask riba; 'al-zaʿīm ghārim' — the guarantor is liable (Sunan Abi Dawud, al-Tirmidhi, ḥasan). On paying, the guarantor has recourse to the debtor for exactly what he paid, no surcharge [scholar-verify]";
const C_RAHN: &str = "rahn (pledge / collateral): al-Baqarah 2:283 (a pledge taken in hand); the Prophet ﷺ pledged his armour to a Jew of Madina for barley (Sahih al-Bukhari, Sahih Muslim). The pledge secures the debt but the creditor takes NO benefit from it (the majority: a benefit to the creditor is riba on the loan), and on default the surplus over the debt returns to the pledgor — the pledge is not forfeit (lā yaghlaqu al-rahn). Held as amāna in the creditor's hand (majority; the Ḥanafīs hold it guaranteed up to the debt) [scholar-verify]";
const C_QARD: &str = "qarḍ ḥasan (benevolent loan): al-Ḥadid 57:11 and al-Baqarah 2:245 (lending Allah a goodly loan); the loan is repaid in like WITHOUT any stipulated increase or benefit to the lender — 'every loan that draws a benefit is riba' (a well-known maxim; the marfūʿ wording is weak, but the prohibition of a stipulated increase is by ijmaʿ). An unstipulated gift (hadiyya) at repayment is permitted [scholar-verify]";
const C_TAWARRUQ: &str = "tawarruq: the individual (fardī/classical) form is permitted by the majority — buy a commodity on credit, take possession, then sell it to a THIRD party for spot cash; but selling back to the original seller is bayʿ al-ʿīnah, and ORGANIZED tawarruq (munaẓẓam, where the financier arranges the onward sale into a ring) was forbidden by the OIC International Islamic Fiqh Academy Resolution 179 (19/5), 2009 [scholar-verify]";
const C_ROLE: &str = "valuation must be independently attested, not self-reported [scholar-verify]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub citation: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn error(code: &str, span: Span, msg: impl Into<String>, citation: &str) -> Self {
        Diagnostic {
            code: code.to_string(),
            severity: Severity::Error,
            message: msg.into(),
            citation: citation.to_string(),
            span,
        }
    }
    pub fn warn(code: &str, span: Span, msg: impl Into<String>, citation: &str) -> Self {
        Diagnostic {
            code: code.to_string(),
            severity: Severity::Warning,
            message: msg.into(),
            citation: citation.to_string(),
            span,
        }
    }
    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Class {
    MusharakahMutanaqisah,
    Mudarabah,
    IjarahImbt,
    Murabahah,
    Salam,
    Istisna,
    Sarf,
    Tawarruq,
    QardHasan,
    Rahn,
    Kafala,
    Hawala,
    Wadia,
    Wakala,
    CommercialEscrow,
    Unknown(String),
}

impl Class {
    pub fn from_str(s: &str) -> Class {
        match s {
            "musharakah_mutanaqisah" => Class::MusharakahMutanaqisah,
            "mudarabah" => Class::Mudarabah,
            "ijarah_imbt" => Class::IjarahImbt,
            "murabahah" => Class::Murabahah,
            "salam" => Class::Salam,
            "istisna" => Class::Istisna,
            "sarf" => Class::Sarf,
            "tawarruq" => Class::Tawarruq,
            "qard_hasan" => Class::QardHasan,
            "rahn" => Class::Rahn,
            "kafala" => Class::Kafala,
            "hawala" => Class::Hawala,
            "wadia" => Class::Wadia,
            "wakala" => Class::Wakala,
            "commercial_escrow" => Class::CommercialEscrow,
            other => Class::Unknown(other.to_string()),
        }
    }

    /// The legal regime this class belongs to.
    pub fn regime(&self) -> &'static str {
        match self {
            Class::CommercialEscrow => "common_law",
            _ => "islamic",
        }
    }
}

// Common-law doctrine citations (real leading authorities; flagged for human verification).
const C_PENALTY: &str =
    "penalty doctrine: Dunlop v New Garage [1915] AC 79; Cavendish Square v Makdessi [2015] UKSC 67 [verify]";
const C_CERTAINTY: &str = "certainty of terms: Scammell & Nephew v Ouston [1941] AC 251 [verify]";
const C_CONSIDERATION: &str = "consideration must move from the promisee: Currie v Misa (1875) LR 10 Ex 153 [verify]";
const C_GOODFAITH: &str = "duty of good faith (UCC sec. 1-304; Yam Seng v ITC [2013] EWHC 111) [verify]";
const C_ZAKAT: &str = "zakat al-tijarah: rub' al-'ushr (1/40 = 2.5%) on trade goods at haul (a lunar year) + nisab — al-Tawbah 9:103; Sunan Abi Dawud (athar of Samurah b. Jundub on goods prepared for sale); AAOIFI SS No. 35 [scholar-verify]";
const C_ASNAF: &str = "the eight categories of zakat recipients (aṣnāf): al-Tawba 9:60 — al-fuqarāʾ, al-masākīn, al-ʿāmilīn ʿalayhā, al-muʾallafa qulūbuhum, fī al-riqāb, al-ghārimīn, fī sabīlillāh, ibn al-sabīl [scholar-verify]";
const C_JAAIHAH: &str = "wad' al-jawa'ih (remission for blight): the Prophet ﷺ ordered the abatement of a buyer's obligation when goods are struck by a calamity — Sahih Muslim, hadith of Jabir; the loss falls on the owner, never added as interest [scholar-verify]";
const C_FARAID: &str = "al-fara'id: the estate of a deceased passes by the fixed shares apportioned in al-Nisa' 4:11-12, 4:176 — distribution is by the furud, not by discretion [scholar-verify]";

/// Run the engine. Returns all diagnostics; callers gate codegen on the presence
/// of any `Severity::Error`.
pub fn check(spec: &Spec) -> Vec<Diagnostic> {
    let mut d = Vec::new();

    if meta_get(spec, "basis").is_none() {
        d.push(Diagnostic::warn(
            "META-1",
            spec.span,
            "no fiqh basis cited; declare meta { basis: \"...\"; } and have a scholar ratify it",
            "",
        ));
    }

    let class = Class::from_str(&spec.class);

    // The same machinery — declare a rule-base R, refuse specs inconsistent with R, generate the
    // enforcing contract — applies across legal regimes. Only R differs (this is the universality
    // claim). A declared regime that contradicts the class is itself an inconsistency.
    if let Class::Unknown(s) = &class {
        d.push(Diagnostic::error(
            "CLASS-1",
            spec.span,
            format!("unknown instrument class '{}' — the engine has no rule-base for it", s),
            "",
        ));
        return d;
    }
    if let Some(dr) = meta_get(spec, "regime").and_then(|e| e.as_ident()) {
        if dr != class.regime() {
            d.push(Diagnostic::error(
                "REGIME-1",
                spec.span,
                format!("declared regime '{}' does not match the '{}' regime of class '{}'", dr, class.regime(), spec.class),
                "",
            ));
        }
    }

    match class {
        Class::MusharakahMutanaqisah | Class::Mudarabah | Class::IjarahImbt => {
            check_role_separation(spec, &mut d);
            check_oracle_cfg(spec, &mut d);
            match Class::from_str(&spec.class) {
                Class::MusharakahMutanaqisah => check_musharakah(spec, &mut d),
                Class::Mudarabah => check_mudarabah(spec, &mut d),
                Class::IjarahImbt => check_ijarah(spec, &mut d),
                _ => {}
            }
        }
        Class::Murabahah => check_murabahah(spec, &mut d),
        Class::Salam => check_salam(spec, &mut d),
        Class::Istisna => check_istisna(spec, &mut d),
        Class::Sarf => check_sarf(spec, &mut d),
        Class::Tawarruq => check_tawarruq(spec, &mut d),
        Class::QardHasan => check_qard(spec, &mut d),
        Class::Rahn => check_rahn(spec, &mut d),
        Class::Kafala => check_kafala(spec, &mut d),
        Class::Hawala => check_hawala(spec, &mut d),
        Class::Wadia => check_wadia(spec, &mut d),
        Class::Wakala => check_wakala(spec, &mut d),
        Class::CommercialEscrow => check_commercial(spec, &mut d),
        Class::Unknown(_) => {}
    }

    // Zakat al-Tijarah (enterprise vector #5): optional, but when declared it must be the
    // agreed rate (1/40), a lunar haul, a positive nisab, and a real beneficiary.
    check_zakat(spec, &mut d);

    // Contingency off-ramps (enterprise vector #4): jaa'ihah reschedule + faraid dissolution.
    check_contingency(spec, &mut d);

    d
}

fn contingency_get<'a>(spec: &'a Spec, key: &str) -> Option<&'a Expr> {
    spec.contingency_cfg().into_iter().find(|k| k.key == key).map(|k| &k.val)
}

/// Validate an optional `contingency { ... }` section — the emergency exits a real contract
/// needs when struck by force majeure or the death of a party. The handlers are constrained:
/// a jaa'ihah may only RESCHEDULE (never add a penalty/interest), and a death may only invoke
/// FARAID (never discretionary distribution).
fn check_contingency(spec: &Spec, d: &mut Vec<Diagnostic>) {
    if spec.contingency_cfg().is_empty() {
        return;
    }
    let mut recognized = false;

    if let Some(h) = contingency_get(spec, "jaaihah").and_then(|e| e.as_ident()) {
        recognized = true;
        if h != "reschedule" && h != "abate" {
            d.push(Diagnostic::error(
                "CONT-1",
                spec.span,
                format!("jaa'ihah handler '{}' is not allowed; a calamity may only 'reschedule' or 'abate' the obligation — never add a penalty or interest", h),
                C_JAAIHAH,
            ));
        }
    }

    if let Some(h) = contingency_get(spec, "death").and_then(|e| e.as_ident()) {
        recognized = true;
        if h != "faraid" {
            d.push(Diagnostic::error(
                "CONT-2",
                spec.span,
                format!("death handler '{}' is not allowed; an estate must be distributed by faraid (the fixed Qur'anic shares), not by discretion", h),
                C_FARAID,
            ));
        } else if role_party(spec, "arbiter").is_none() && role_party(spec, "adjudicator").is_none() {
            d.push(Diagnostic::error(
                "CONT-3",
                spec.span,
                "a faraid dissolution requires an arbiter/adjudicator to invoke and attest the distribution",
                C_FARAID,
            ));
        }
    }

    if !recognized {
        d.push(Diagnostic::warn(
            "CONT-4",
            spec.span,
            "contingency block declares no recognized off-ramp (expected jaaihah and/or death)",
            "",
        ));
    }
}

fn zakat_get<'a>(spec: &'a Spec, key: &str) -> Option<&'a Expr> {
    spec.zakat_cfg().into_iter().find(|k| k.key == key).map(|k| &k.val)
}

/// Validate an optional `zakat { ... }` section. Absent => no built-in zakat routing. When
/// present, the engine compiles the 2.5% routing into the contract, so corporate zakat
/// becomes non-bypassable rather than a year-end act of conscience.
fn check_zakat(spec: &Spec, d: &mut Vec<Diagnostic>) {
    if spec.zakat_cfg().is_empty() {
        return;
    }
    // ZAKAT-5: the genus must be one the engine can rate. An absent `kind` defaults to tijarah —
    // back-compatible with the original zakat al-tijarah layer.
    let kind = zakat_get(spec, "kind").and_then(|e| e.as_ident()).unwrap_or("tijarah");
    let expected_rate = crate::zakat::rate_for_kind(kind);
    if expected_rate.is_none() {
        d.push(Diagnostic::error(
            "ZAKAT-5",
            spec.span,
            format!("unknown zakat genus '{}'; known: tijarah/gold/silver/currency/salary/mustaghallat (2.5%), crops_rain (10%), crops_irrigated (5%)", kind),
            C_ZAKAT,
        ));
    }
    // ZAKAT-1: the rate must match the genus — rubʿ al-ʿushr (2.5%) for wealth, ʿushr (10%) for
    // rain-fed produce, niṣf al-ʿushr (5%) for irrigated produce.
    if let Some(exp) = expected_rate {
        match zakat_get(spec, "rate_bps").and_then(|e| e.as_num()) {
            Some(r) if r == exp => {}
            Some(r) => d.push(Diagnostic::error(
                "ZAKAT-1",
                spec.span,
                format!("zakat rate is {} bps; zakat on '{}' is {} bps", r, kind, exp),
                C_ZAKAT,
            )),
            None => d.push(Diagnostic::error(
                "ZAKAT-1",
                spec.span,
                format!("zakat block must declare rate_bps: {} for genus '{}'", exp, kind),
                C_ZAKAT,
            )),
        }
    }
    // ZAKAT-2: wealth is due at a HIJRI (lunar) haul; produce is due at HARVEST (no haul).
    if crate::zakat::is_produce_kind(kind) {
        match zakat_get(spec, "haul").and_then(|e| e.as_ident()) {
            Some("harvest") => {}
            _ => d.push(Diagnostic::error(
                "ZAKAT-2",
                spec.span,
                "produce zakat is due at HARVEST (al-Anʿam 6:141), not at a lunar haul; declare haul: harvest",
                C_ZAKAT,
            )),
        }
    } else {
        match zakat_get(spec, "haul").and_then(|e| e.as_ident()) {
            Some(h) if crate::zakat::is_lunar_haul(h) => {}
            Some(h) => d.push(Diagnostic::error(
                "ZAKAT-2",
                spec.span,
                format!("haul '{}' is not a lunar year; zakat's haul is the HIJRI (lunar ~354-day) year — a solar year systematically under-collects", h),
                C_ZAKAT,
            )),
            None => d.push(Diagnostic::error(
                "ZAKAT-2",
                spec.span,
                "zakat block must declare haul: hijri_year",
                C_ZAKAT,
            )),
        }
    }
    // ZAKAT-3: a positive nisab threshold (the 85g-gold / 595g-silver equivalent).
    match zakat_get(spec, "nisab").and_then(|e| e.as_num()) {
        Some(n) if n > 0 => {}
        _ => d.push(Diagnostic::error(
            "ZAKAT-3",
            spec.span,
            "zakat block must declare a positive nisab (the 85g-gold / 595g-silver equivalent in the ledger's unit)",
            C_ZAKAT,
        )),
    }
    // ZAKAT-4: a beneficiary that resolves to a declared party or role — the 2.5% needs a home.
    match zakat_get(spec, "beneficiary").and_then(|e| e.as_ident()) {
        Some(b) => {
            let known = spec.parties().iter().any(|p| p.name == b || p.role == b);
            if !known {
                d.push(Diagnostic::error(
                    "ZAKAT-4",
                    spec.span,
                    format!("zakat beneficiary '{}' is not a declared party or role (the maslahah / zakat fund must receive the 2.5%)", b),
                    C_ZAKAT,
                ));
            }
        }
        None => d.push(Diagnostic::error(
            "ZAKAT-4",
            spec.span,
            "zakat block must declare a beneficiary (e.g. beneficiary: maslahah)",
            C_ZAKAT,
        )),
    }

    // ZAKAT-6: if a disbursement policy by the eight aṣnāf (al-Tawba 9:60) is declared at all,
    // every one of the eight categories must be present and the shares must sum to 10000 bps.
    // (The fund remains the on-chain collection point; this is the validated, recorded policy by
    // which a trustee disburses to eligible recipients.)
    let asnaf_keys = [
        "asnaf_fuqara", "asnaf_masakin", "asnaf_amilin", "asnaf_muallafa",
        "asnaf_riqab", "asnaf_gharimin", "asnaf_fi_sabilillah", "asnaf_ibn_sabil",
    ];
    if spec.zakat_cfg().iter().any(|kv| kv.key.starts_with("asnaf_")) {
        let mut sum: u64 = 0;
        for cat in asnaf_keys {
            match zakat_get(spec, cat).and_then(|e| e.as_num()) {
                Some(n) => sum += n,
                None => d.push(Diagnostic::error(
                    "ZAKAT-6",
                    spec.span,
                    format!("the zakat aṣnāf policy is missing category '{}'; all eight of al-Tawba 9:60 must be present", cat),
                    C_ASNAF,
                )),
            }
        }
        for kv in spec.zakat_cfg() {
            if kv.key.starts_with("asnaf_") && !asnaf_keys.contains(&kv.key.as_str()) {
                d.push(Diagnostic::error(
                    "ZAKAT-6",
                    kv.span,
                    format!("'{}' is not one of the eight aṣnāf of al-Tawba 9:60", kv.key),
                    C_ASNAF,
                ));
            }
        }
        if sum != 10_000 {
            d.push(Diagnostic::error(
                "ZAKAT-6",
                spec.span,
                format!("the aṣnāf shares sum to {} bps; the eight categories must total exactly 10000 bps", sum),
                C_ASNAF,
            ));
        }
    }
}

// --- shared helpers ---

fn meta_get<'a>(spec: &'a Spec, key: &str) -> Option<&'a Expr> {
    spec.meta().into_iter().find(|k| k.key == key).map(|k| &k.val)
}

fn risk_get<'a>(spec: &'a Spec, key: &str) -> Option<&'a Expr> {
    spec.risk().into_iter().find(|k| k.key == key).map(|k| &k.val)
}

/// Span-aware lookup, so a diagnostic can point at the OFFENDING token (precise line/col)
/// rather than the instrument header — the foundation of the LSP's red squiggle.
fn risk_kv<'a>(spec: &'a Spec, key: &str) -> Option<&'a Kv> {
    spec.risk().into_iter().find(|k| k.key == key)
}

fn ret_kv<'a>(rb: &'a RetBlock, key: &str) -> Option<&'a Kv> {
    rb.kvs.iter().find(|k| k.key == key)
}

fn role_party<'a>(spec: &'a Spec, role: &str) -> Option<&'a Party> {
    spec.parties().into_iter().find(|p| p.role == role)
}

fn find_return<'a>(spec: &'a Spec, kind: &str) -> Option<&'a RetBlock> {
    spec.returns().into_iter().find(|r| r.kind == kind)
}

fn capital_assigns(spec: &Spec) -> Vec<(String, u64)> {
    spec.capital()
        .into_iter()
        .filter_map(|c| match c {
            CapItem::Assign { party, bps, .. } => Some((party.clone(), *bps)),
            _ => None,
        })
        .collect()
}

fn party_bps(spec: &Spec, party: &str) -> Option<u64> {
    capital_assigns(spec)
        .into_iter()
        .find(|(p, _)| p == party)
        .map(|(_, b)| b)
}

/// Collect the leading segment of every path mentioned in an expression.
fn collect_heads(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Path(p) => {
            if let Some(f) = p.first() {
                out.push(f.clone());
            }
        }
        Expr::Bin(l, _, r) => {
            collect_heads(l, out);
            collect_heads(r, out);
        }
        Expr::Paren(x) => collect_heads(x, out),
        _ => {}
    }
}

fn expr_mentions(e: &Expr, name: &str) -> bool {
    let mut v = Vec::new();
    collect_heads(e, &mut v);
    v.iter().any(|s| s == name)
}

/// Does this return-basis refer to principal/capital (interest on money) rather
/// than to a living share / usufruct / realized profit?
fn basis_is_principal(e: &Expr) -> bool {
    if let Some(id) = e.as_ident() {
        if id == "principal" || id == "capital" {
            return true;
        }
    }
    if let Some(p) = e.as_path() {
        if let Some(last) = p.last() {
            if last == "principal" || last == "capital" {
                return true;
            }
        }
    }
    false
}

fn require_invariants(spec: &Spec, names: &[&str], d: &mut Vec<Diagnostic>) {
    for name in names {
        if !spec.has_invariant(name) {
            d.push(Diagnostic::error(
                "INV-1",
                spec.span,
                format!("required invariant '{}' is not declared for this instrument class", name),
                "",
            ));
        }
    }
}

fn check_capital_sum(spec: &Spec, d: &mut Vec<Diagnostic>, require_multi: bool) {
    let assigns = capital_assigns(spec);
    let sum: u64 = assigns.iter().map(|(_, b)| *b).sum();
    if sum != 10_000 {
        d.push(Diagnostic::error(
            "CAP-1",
            spec.span,
            format!("capital shares sum to {} bps; they must total exactly 10000 bps", sum),
            "",
        ));
    }
    if require_multi && assigns.iter().filter(|(_, b)| *b > 0).count() < 2 {
        d.push(Diagnostic::error(
            "CAP-2",
            spec.span,
            "a partnership requires at least two capital contributors",
            "",
        ));
    }
}

fn oracle_get<'a>(spec: &'a Spec, key: &str) -> Option<&'a Expr> {
    spec.oracle_cfg().into_iter().find(|k| k.key == key).map(|k| &k.val)
}

/// Validate an optional `oracle { ... }` configuration. Absent => single-valuer (mock) mode.
/// `mode: consensus` declares a zero-trust committee oracle; the gharar boundary becomes the
/// declared dispersion bound, enforced on-chain.
fn check_oracle_cfg(spec: &Spec, d: &mut Vec<Diagnostic>) {
    if spec.oracle_cfg().is_empty() {
        return;
    }
    let mode = oracle_get(spec, "mode").and_then(|e| e.as_ident()).unwrap_or("");
    if mode != "consensus" {
        d.push(Diagnostic::error(
            "ORACLE-1",
            spec.span,
            format!("unknown oracle mode '{}'; only 'consensus' is supported", mode),
            C_GHARAR,
        ));
        return;
    }
    let committee = oracle_get(spec, "committee").and_then(|e| e.as_num());
    let quorum = oracle_get(spec, "quorum").and_then(|e| e.as_num());
    let bound = oracle_get(spec, "gharar_bound_bps").and_then(|e| e.as_num());
    match (committee, quorum, bound) {
        (Some(c), Some(q), Some(b)) => {
            if q < 1 {
                d.push(Diagnostic::error("ORACLE-2", spec.span, "quorum must be >= 1", C_GHARAR));
            }
            if c < q {
                d.push(Diagnostic::error(
                    "ORACLE-3",
                    spec.span,
                    format!("committee ({}) must be >= quorum ({})", c, q),
                    C_GHARAR,
                ));
            }
            if b == 0 || b >= 10_000 {
                d.push(Diagnostic::error(
                    "ORACLE-4",
                    spec.span,
                    "gharar_bound_bps must be in (0, 10000) — the dispersion above which a value is majhul",
                    C_GHARAR,
                ));
            }
        }
        _ => d.push(Diagnostic::error(
            "ORACLE-5",
            spec.span,
            "a consensus oracle requires committee, quorum, and gharar_bound_bps",
            C_GHARAR,
        )),
    }
}

fn check_role_separation(spec: &Spec, d: &mut Vec<Diagnostic>) {
    match role_party(spec, "oracle") {
        None => d.push(Diagnostic::error(
            "ROLE-1",
            spec.span,
            "no independent valuation oracle is declared; a real-world value would be self-reported by a contracting party (gharar / self-dealing)",
            C_ROLE,
        )),
        Some(o) => {
            if !o.flags.iter().any(|f| f == "independent") {
                d.push(Diagnostic::error(
                    "ROLE-2",
                    o.span,
                    format!("party '{}' acts as the valuation oracle but is not marked 'independent'", o.name),
                    C_ROLE,
                ));
            }
        }
    }
}

// --- Musharakah Mutanaqisah ---

fn check_musharakah(spec: &Spec, d: &mut Vec<Diagnostic>) {
    // RIBA-1: no guaranteed capital. (precise span: the offending kv)
    match risk_kv(spec, "capital_guarantee") {
        Some(kv) if kv.val.as_ident() == Some("none") => {}
        Some(kv) => d.push(Diagnostic::error(
            "RIBA-1",
            kv.span,
            format!(
                "capital is guaranteed to '{}'; a guaranteed return of capital turns a partnership into an interest-bearing loan (riba)",
                kv.val.render()
            ),
            C_RIBA,
        )),
        None => d.push(Diagnostic::error(
            "RIBA-1",
            spec.span,
            "a musharakah must explicitly declare risk { capital_guarantee: none }",
            C_RIBA,
        )),
    }

    // RISK-1: loss is shared proportional to ownership. (precise span)
    match risk_kv(spec, "loss") {
        Some(kv) if kv.val.as_ident() == Some("proportional_to_ownership") => {}
        Some(kv) => d.push(Diagnostic::error(
            "RISK-1",
            kv.span,
            format!(
                "loss allocation is '{}'; a diminishing partnership must share loss proportional_to_ownership (no risk-sharing = no partnership)",
                kv.val.render()
            ),
            C_RISK,
        )),
        None => d.push(Diagnostic::error(
            "RISK-1",
            spec.span,
            "a musharakah must declare risk { loss: proportional_to_ownership }",
            C_RISK,
        )),
    }

    // RIBA-2: rent falls on the living share, never on principal.
    match find_return(spec, "rent") {
        None => d.push(Diagnostic::error(
            "RENT-2",
            spec.span,
            "musharakah mutanaqisah requires a rent (ijarah) return on the financier's living share",
            C_RIBA,
        )),
        Some(rent) => match ret_kv(rent, "basis") {
            None => d.push(Diagnostic::error("RENT-1", rent.span, "rent block has no 'basis'", "")),
            Some(kv) if basis_is_principal(&kv.val) => d.push(Diagnostic::error(
                "RIBA-2",
                kv.span,
                "rent is charged on principal/capital — that is interest on a loan, not rent on a living share",
                C_RIBA,
            )),
            Some(kv) => {
                let fin = role_party(spec, "financier").map(|p| p.name.clone());
                let ok = matches!(kv.val.as_path(), Some(p) if p.len() == 2 && Some(p[0].clone()) == fin && p[1] == "share");
                if !ok {
                    d.push(Diagnostic::warn(
                        "RIBA-2",
                        kv.span,
                        format!("rent basis '{}' is not <financier>.share; rent should fall on the financier's living share", kv.val.render()),
                        C_RIBA,
                    ));
                }
            }
        },
    }

    // GHARAR-1: the buyout price is derived from the independent oracle.
    match find_return(spec, "buyout") {
        None => d.push(Diagnostic::error(
            "BUYOUT-2",
            spec.span,
            "musharakah mutanaqisah requires a buyout mechanism (the diminishing leg)",
            "",
        )),
        Some(b) => match ret_kv(b, "price") {
            None => d.push(Diagnostic::error("BUYOUT-1", b.span, "buyout has no 'price'", "")),
            Some(kv) => {
                let oracle = role_party(spec, "oracle").map(|p| p.name.clone());
                let attested = expr_mentions(&kv.val, "oracle")
                    || oracle.as_deref().map(|n| expr_mentions(&kv.val, n)).unwrap_or(false);
                if !attested {
                    d.push(Diagnostic::error(
                        "GHARAR-1",
                        kv.span,
                        format!(
                            "buyout price '{}' is not derived from the independent oracle; a self-named or fixed price re-introduces gharar and can disguise a guaranteed return",
                            kv.val.render()
                        ),
                        C_GHARAR,
                    ));
                }
            }
        },
    }

    require_invariants(
        spec,
        &["ownership_conserved", "rent_on_living_share", "loss_follows_capital", "price_attested"],
        d,
    );
    check_capital_sum(spec, d, true);
}

// --- Mudarabah ---

fn check_mudarabah(spec: &Spec, d: &mut Vec<Diagnostic>) {
    // RIBA-1: no guaranteed capital and no guaranteed profit.
    match risk_get(spec, "capital_guarantee") {
        Some(e) if e.as_ident() == Some("none") => {}
        Some(e) => d.push(Diagnostic::error(
            "RIBA-1",
            spec.span,
            format!("capital is guaranteed to '{}'; the mudarib may not guarantee the rabb al-mal's capital (riba)", e.render()),
            C_MUDARABAH,
        )),
        None => d.push(Diagnostic::error(
            "RIBA-1",
            spec.span,
            "a mudarabah must declare risk { capital_guarantee: none }",
            C_MUDARABAH,
        )),
    }

    // RISK-2: financial loss falls on the rabb al-mal alone (absent the mudarib's ta'addi/taqsir).
    match risk_get(spec, "loss") {
        Some(e) if e.as_ident() == Some("on_rabb_al_mal") => {}
        Some(e) => d.push(Diagnostic::error(
            "RISK-2",
            spec.span,
            format!("loss allocation is '{}'; in mudarabah, financial loss is borne by the rabb al-mal alone (the mudarib loses only effort), absent proven misconduct", e.render()),
            C_MUDARABAH,
        )),
        None => d.push(Diagnostic::error(
            "RISK-2",
            spec.span,
            "a mudarabah must declare risk { loss: on_rabb_al_mal }",
            C_MUDARABAH,
        )),
    }

    // MUD-1: capital comes from the rabb al-mal only; the mudarib contributes labor, not capital.
    let rabb = role_party(spec, "rabb_al_mal").map(|p| p.name.clone());
    let mudarib = role_party(spec, "mudarib").map(|p| p.name.clone());
    if rabb.is_none() {
        d.push(Diagnostic::error("MUD-2", spec.span, "mudarabah requires a party with role 'rabb_al_mal' (capital provider)", C_MUDARABAH));
    }
    if mudarib.is_none() {
        d.push(Diagnostic::error("MUD-3", spec.span, "mudarabah requires a party with role 'mudarib' (entrepreneur)", C_MUDARABAH));
    }
    if let Some(m) = &mudarib {
        if party_bps(spec, m).unwrap_or(0) > 0 {
            d.push(Diagnostic::error(
                "MUD-1",
                spec.span,
                "the mudarib contributes labor, not capital; mudarib capital > 0 makes this a musharakah, not a mudarabah",
                C_MUDARABAH,
            ));
        }
    }
    if let Some(r) = &rabb {
        if party_bps(spec, r).unwrap_or(0) != 10_000 {
            d.push(Diagnostic::error(
                "MUD-4",
                spec.span,
                "in mudarabah the rabb al-mal provides 100% of the capital (10000 bps)",
                C_MUDARABAH,
            ));
        }
    }

    // profit: shared by a pre-agreed RATIO (not a fixed sum), attested independently.
    match find_return(spec, "profit") {
        None => d.push(Diagnostic::error("PROFIT-2", spec.span, "mudarabah requires a profit return shared by ratio", C_MUDARABAH)),
        Some(p) => {
            match kv_get(&p.kvs, "split") {
                Some(e) if e.as_ident() == Some("ratio") => {}
                _ => d.push(Diagnostic::error(
                    "PROFIT-1",
                    p.span,
                    "profit must be split by a pre-agreed ratio (split: ratio); a fixed guaranteed sum to either party is riba",
                    C_MUDARABAH,
                )),
            }
            let oracle = role_party(spec, "oracle").map(|q| q.name.clone());
            let attested = kv_get(&p.kvs, "source")
                .map(|src| expr_mentions(src, "oracle") || oracle.as_deref().map(|n| expr_mentions(src, n)).unwrap_or(false))
                .unwrap_or(false);
            if !attested {
                d.push(Diagnostic::error(
                    "GHARAR-2",
                    p.span,
                    "realized profit must be attested by the independent oracle (profit { source: oracle.realizedProfit; }), not self-reported by the mudarib",
                    C_GHARAR,
                ));
            }
        }
    }

    require_invariants(
        spec,
        &["capital_from_rabb_al_mal_only", "profit_by_ratio", "loss_on_rabb_al_mal", "no_guaranteed_profit"],
        d,
    );
    check_capital_sum(spec, d, false);
}

// --- Murabaha (cost-plus trust sale, bay' al-amana) ---
//
// Murabaha is a SALE, not a loan and not a partnership: the bank buys a real good, takes
// possession of it, discloses its true cost, and sells it on for a fixed, disclosed markup —
// payable deferred. The riba creeps in three ways, and the engine refuses each: (1) a markup
// that grows with the deferral period is interest, not trade profit; (2) selling before
// possession (qabd) turns the "sale" into a financing of money for money; (3) a late-payment
// penalty that accrues to the seller is interest on the resulting debt. Consistency, not fatwa.
fn check_murabahah(spec: &Spec, d: &mut Vec<Diagnostic>) {
    // parties: a distinct seller (the bank) and buyer (the customer).
    let seller = role_party(spec, "seller").map(|p| p.name.clone());
    let buyer = role_party(spec, "buyer").map(|p| p.name.clone());
    if seller.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "a murabaha requires a party with role 'seller' (the bank/financier who buys then resells)", C_MURABAHA));
    }
    if buyer.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "a murabaha requires a party with role 'buyer' (the customer)", C_MURABAHA));
    }
    if let (Some(s), Some(b)) = (&seller, &buyer) {
        if s == b {
            d.push(Diagnostic::error("PARTY-1", spec.span, "seller and buyer must be distinct parties — a sale needs two sides", C_MURABAHA));
        }
    }

    // the `sale` block: disclosed cost, fixed markup, definite total, no penalty-riba.
    match find_return(spec, "sale") {
        None => d.push(Diagnostic::error(
            "MUR-1",
            spec.span,
            "a murabaha requires returns { sale { cost; markup; total; markup_basis; payment } } — it is bay' al-amana, a trust sale on a disclosed cost",
            C_MURABAHA,
        )),
        Some(sale) => {
            // MUR-1: the true acquisition cost must be DISCLOSED (a definite, non-zero sum). This is
            // what makes murabaha a trust sale (bay' al-amana) rather than an ordinary bargained sale.
            match ret_kv(sale, "cost") {
                Some(kv) if kv.val.as_num().map(|n| n > 0).unwrap_or(false) => {}
                Some(kv) => d.push(Diagnostic::error(
                    "MUR-1",
                    kv.span,
                    "the murabaha cost must be a definite, disclosed sum (bay' al-amana: the buyer must know the true cost)",
                    C_MURABAHA,
                )),
                None => d.push(Diagnostic::error(
                    "MUR-1",
                    sale.span,
                    "murabaha must disclose the acquisition 'cost' — the buyer must know it (bay' al-amana)",
                    C_MURABAHA,
                )),
            }

            // RIBA-2: the markup must be a FIXED, disclosed sum agreed at contract — never a function
            // of time or of the principal. A markup that grows with the deferral period is interest.
            match ret_kv(sale, "markup_basis") {
                Some(kv) if kv.val.as_ident() == Some("fixed") => {}
                Some(kv) if kv.val.as_ident() == Some("time") || basis_is_principal(&kv.val) => d.push(Diagnostic::error(
                    "RIBA-2",
                    kv.span,
                    "the markup is tied to time/principal — a profit that grows with the deferral period is interest (riba), not a fixed trade markup",
                    C_RIBA,
                )),
                Some(kv) => d.push(Diagnostic::warn(
                    "RIBA-2",
                    kv.span,
                    format!("markup_basis '{}' is not 'fixed'; a murabaha markup must be a fixed disclosed sum agreed at contract", kv.val.render()),
                    C_RIBA,
                )),
                None => d.push(Diagnostic::error(
                    "RIBA-2",
                    sale.span,
                    "murabaha must declare markup_basis: fixed — the profit is a fixed sum agreed at contract, never interest accruing over time",
                    C_RIBA,
                )),
            }

            // GHARAR-1: the total price must be a definite, known sum at contract (price certainty).
            match ret_kv(sale, "total") {
                Some(kv) if kv.val.as_num().map(|n| n > 0).unwrap_or(false) => {}
                _ => d.push(Diagnostic::error(
                    "GHARAR-1",
                    sale.span,
                    "murabaha must declare a definite total price (cost + markup), known at contract — an unknown price is gharar",
                    C_GHARAR,
                )),
            }

            // RIBA-3: no late-payment penalty accruing to the seller (interest on the resulting debt).
            if let Some(kv) = sale.kvs.iter().find(|k| k.key == "late_penalty") {
                if kv.val.as_ident() != Some("none") {
                    d.push(Diagnostic::error(
                        "RIBA-3",
                        kv.span,
                        "a late-payment penalty accruing to the seller is interest on a debt (riba); any charge must go to charity, not the seller",
                        C_RIBA,
                    ));
                }
            }
        }
    }

    // MUR-2 (qabd / prior ownership): the bank must take ownership and possession BEFORE selling.
    // 'Do not sell what you do not have' (Hakim b. Hizam). Enforced by a distinct lifecycle step
    // 'acquireAsset' that must PRECEDE 'sell' — the substance that separates murabaha from a loan.
    let lc = spec.lifecycle();
    let acq = lc.iter().position(|s| s.name == "acquireAsset");
    let sell = lc.iter().position(|s| s.name == "sell");
    match (acq, sell) {
        (Some(a), Some(s)) if a < s => {}
        (None, _) => d.push(Diagnostic::error(
            "MUR-2",
            spec.span,
            "murabaha requires a lifecycle step 'acquireAsset' — the bank must take ownership and possession (qabd) of the good BEFORE selling it ('do not sell what you do not have')",
            C_MURABAHA,
        )),
        (Some(_), None) => d.push(Diagnostic::error(
            "MUR-2",
            spec.span,
            "murabaha requires a 'sell' lifecycle step following 'acquireAsset'",
            C_MURABAHA,
        )),
        (Some(_), Some(_)) => d.push(Diagnostic::error(
            "MUR-2",
            spec.span,
            "in murabaha 'acquireAsset' (taking possession) must PRECEDE 'sell' — selling before possession (qabd) is invalid",
            C_MURABAHA,
        )),
    }

    require_invariants(
        spec,
        &["cost_disclosed", "markup_fixed", "price_certain", "prior_ownership", "no_penalty_interest"],
        d,
    );
}

// --- Salam (forward sale: full prepayment now, described good delivered later) ---
//
// Salam is the one sale where the object need not yet exist — precisely because the price is
// paid IN FULL at the session. The fiqh holds it tight against gharar: (1) defer the price and
// you have bayʿ al-kaliʾ bi-l-kaliʾ (debt for debt); (2) leave the object un-described and you
// have gharar. So the engine demands full prepayment, a known quantity, and a known term.
fn check_salam(spec: &Spec, d: &mut Vec<Diagnostic>) {
    let buyer = role_party(spec, "buyer").map(|p| p.name.clone());
    let seller = role_party(spec, "seller").map(|p| p.name.clone());
    if buyer.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "a salam requires a party with role 'buyer' (rabb al-salam, who pays the price now)", C_SALAM));
    }
    if seller.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "a salam requires a party with role 'seller' (al-muslam ilayh, who delivers later)", C_SALAM));
    }
    if let (Some(b), Some(s)) = (&buyer, &seller) {
        if b == s {
            d.push(Diagnostic::error("PARTY-1", spec.span, "buyer and seller must be distinct parties", C_SALAM));
        }
    }

    match find_return(spec, "salam") {
        None => d.push(Diagnostic::error(
            "SALAM-1",
            spec.span,
            "a salam requires returns { salam { price; payment; quantity; measure; quality; delivery_date } }",
            C_SALAM,
        )),
        Some(s) => {
            // SALAM-1: the whole price (ra's al-mal) is paid at the session — never deferred.
            match ret_kv(s, "payment") {
                Some(kv) if kv.val.as_ident() == Some("spot_full") => {}
                Some(kv) => d.push(Diagnostic::error(
                    "SALAM-1",
                    kv.span,
                    format!("salam payment is '{}'; the full ra's al-mal must be paid at the session — deferring both price and good is bayʿ al-kaliʾ bi-l-kaliʾ (debt for debt)", kv.val.render()),
                    C_SALAM,
                )),
                None => d.push(Diagnostic::error(
                    "SALAM-1",
                    s.span,
                    "salam must declare payment: spot_full (the full price paid at the session)",
                    C_SALAM,
                )),
            }
            // SALAM-2: the muslam fih is a known quantity (maʿlūm) — and a measure + quality.
            match ret_kv(s, "quantity") {
                Some(kv) if kv.val.as_num().map(|n| n > 0).unwrap_or(false) => {}
                _ => d.push(Diagnostic::error(
                    "SALAM-2",
                    s.span,
                    "salam must declare a known, positive quantity of the muslam fih (maʿlūm; an unknown quantity is gharar)",
                    C_SALAM,
                )),
            }
            if ret_kv(s, "measure").and_then(|kv| kv.val.as_ident()).is_none() {
                d.push(Diagnostic::error("SALAM-2", s.span, "salam must declare the measure (kayl/wazn) of the muslam fih", C_SALAM));
            }
            if ret_kv(s, "quality").and_then(|kv| kv.val.as_ident()).is_none() {
                d.push(Diagnostic::error("SALAM-2", s.span, "salam must declare the quality/grade of the muslam fih (so it is fully maʿlūm)", C_SALAM));
            }
            // SALAM-3: a known future delivery date (ajal maʿlūm).
            match ret_kv(s, "delivery_date") {
                Some(kv) if kv.val.as_num().map(|n| n > 0).unwrap_or(false) => {}
                _ => d.push(Diagnostic::error(
                    "SALAM-3",
                    s.span,
                    "salam must declare a known future delivery_date (ajal maʿlūm)",
                    C_SALAM,
                )),
            }
        }
    }

    require_invariants(spec, &["full_prepayment", "object_known", "delivery_known"], d);
}

// --- Istisna' (manufacture-to-order) ---
//
// Istisnaʿ commissions a good that does not yet exist — to be MADE to a known specification.
// It is distinguished from salam (the price may be deferred) and from ijarat al-ʿamal (the
// maker, al-saniʿ, supplies the materials, not just labour). The engine guards the gharar:
// the masnuʿ must be described, the maker must furnish the materials, the price must be known.
fn check_istisna(spec: &Spec, d: &mut Vec<Diagnostic>) {
    let buyer = role_party(spec, "buyer").map(|p| p.name.clone());
    let maker = role_party(spec, "manufacturer").map(|p| p.name.clone());
    if buyer.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "an istisnaʿ requires a party with role 'buyer' (al-mustasniʿ, who commissions the good)", C_ISTISNA));
    }
    if maker.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "an istisnaʿ requires a party with role 'manufacturer' (al-saniʿ, who builds to spec)", C_ISTISNA));
    }
    if let (Some(b), Some(m)) = (&buyer, &maker) {
        if b == m {
            d.push(Diagnostic::error("PARTY-1", spec.span, "buyer and manufacturer must be distinct parties", C_ISTISNA));
        }
    }

    match find_return(spec, "istisna") {
        None => d.push(Diagnostic::error(
            "ISTISNA-1",
            spec.span,
            "an istisnaʿ requires returns { istisna { price; spec; quantity; material_by; delivery_date } }",
            C_ISTISNA,
        )),
        Some(s) => {
            // ISTISNA-1: the masnuʿ must be DESCRIBED (spec) and of a known quantity — anti-gharar.
            if ret_kv(s, "spec").and_then(|kv| kv.val.as_ident()).is_none() {
                d.push(Diagnostic::error("ISTISNA-1", s.span, "istisnaʿ must describe the masnuʿ (spec: described) — an undescribed made-to-order good is gharar", C_ISTISNA));
            }
            match ret_kv(s, "quantity") {
                Some(kv) if kv.val.as_num().map(|n| n > 0).unwrap_or(false) => {}
                _ => d.push(Diagnostic::error("ISTISNA-1", s.span, "istisnaʿ must declare a known, positive quantity of the masnuʿ", C_ISTISNA)),
            }
            // ISTISNA-2: the maker supplies the materials — else it is ijarat al-ʿamal (hire of labour).
            match ret_kv(s, "material_by") {
                Some(kv) if kv.val.as_ident() == Some("manufacturer") => {}
                Some(kv) => d.push(Diagnostic::error(
                    "ISTISNA-2",
                    kv.span,
                    format!("material_by is '{}'; in istisnaʿ the maker (al-saniʿ) supplies the materials — if the customer supplies them this is ijarat al-ʿamal (hire of labour), a different contract", kv.val.render()),
                    C_ISTISNA,
                )),
                None => d.push(Diagnostic::error("ISTISNA-2", s.span, "istisnaʿ must declare material_by: manufacturer (the maker supplies the materials)", C_ISTISNA)),
            }
            // ISTISNA-3: a known, fixed price (it may be deferred — that is allowed, unlike salam).
            match ret_kv(s, "price") {
                Some(kv) if kv.val.as_num().map(|n| n > 0).unwrap_or(false) => {}
                _ => d.push(Diagnostic::error("ISTISNA-3", s.span, "istisnaʿ must declare a known, positive total price", C_ISTISNA)),
            }
            // delivery term should be known (ajal maʿlūm) — a contemporary requirement (AAOIFI).
            match ret_kv(s, "delivery_date") {
                Some(kv) if kv.val.as_num().map(|n| n > 0).unwrap_or(false) => {}
                _ => d.push(Diagnostic::warn("ISTISNA-4", s.span, "istisnaʿ should declare a known delivery_date (a defined term — AAOIFI SS 11)", C_ISTISNA)),
            }
        }
    }

    require_invariants(spec, &["object_specified", "material_by_maker", "price_known"], d);
}

// --- Sarf (currency / precious-metal exchange) ---
//
// The six-commodities hadith reduces to two pure invariants: SAME genus ⇒ equal amounts (else
// riba al-faḍl); ANY exchange ⇒ settled spot, hand-to-hand (else riba al-nasiʾa). Cross-genus
// amounts may differ (the rate), but the spot requirement never relaxes.
fn check_sarf(spec: &Spec, d: &mut Vec<Diagnostic>) {
    let a = role_party(spec, "exchanger_a").map(|p| p.name.clone());
    let b = role_party(spec, "exchanger_b").map(|p| p.name.clone());
    if a.is_none() || b.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "a ṣarf requires two parties with roles 'exchanger_a' and 'exchanger_b'", C_SARF));
    }
    if let (Some(x), Some(y)) = (&a, &b) {
        if x == y {
            d.push(Diagnostic::error("PARTY-1", spec.span, "the two counterparties to a ṣarf must be distinct", C_SARF));
        }
    }

    match find_return(spec, "exchange") {
        None => d.push(Diagnostic::error(
            "SARF-1",
            spec.span,
            "a ṣarf requires returns { exchange { give_asset; give_amount; take_asset; take_amount; same_genus; settlement } }",
            C_SARF,
        )),
        Some(ex) => {
            // SARF-1: settled spot (yadan bi-yad) — deferring either leg is riba al-nasiʾa.
            match ret_kv(ex, "settlement") {
                Some(kv) if kv.val.as_ident() == Some("spot") => {}
                Some(kv) => d.push(Diagnostic::error(
                    "SARF-1",
                    kv.span,
                    format!("settlement is '{}'; a ṣarf must be spot/yadan bi-yad — deferring either leg is riba al-nasiʾa", kv.val.render()),
                    C_SARF,
                )),
                None => d.push(Diagnostic::error("SARF-1", ex.span, "ṣarf must declare settlement: spot (yadan bi-yad)", C_SARF)),
            }
            let give = ret_kv(ex, "give_amount").and_then(|kv| kv.val.as_num());
            let take = ret_kv(ex, "take_amount").and_then(|kv| kv.val.as_num());
            if give.map(|n| n == 0).unwrap_or(true) || take.map(|n| n == 0).unwrap_or(true) {
                d.push(Diagnostic::error("SARF-3", ex.span, "ṣarf must declare positive give_amount and take_amount", C_SARF));
            }
            // SARF-2: same genus ⇒ equal for equal (else riba al-faḍl). Cross-genus may differ.
            match ret_kv(ex, "same_genus").and_then(|kv| kv.val.as_ident()) {
                Some("yes") => {
                    if let (Some(g), Some(t)) = (give, take) {
                        if g != t {
                            d.push(Diagnostic::error(
                                "SARF-2",
                                ex.span,
                                format!("a same-genus exchange is {} for {}; like must be EQUAL for like — any excess is riba al-faḍl", g, t),
                                C_SARF,
                            ));
                        }
                    }
                }
                Some("no") => {}
                _ => d.push(Diagnostic::error("SARF-2", ex.span, "ṣarf must declare same_genus: yes|no (same genus forces equal amounts)", C_SARF)),
            }
        }
    }

    require_invariants(spec, &["spot_settlement", "riba_fadl_guarded"], d);
}

// --- Tawarruq (individual / classical) ---
//
// Individual tawarruq is permitted: buy a commodity on credit, TAKE POSSESSION, then sell it to
// an INDEPENDENT third party for spot cash. Two collapses are forbidden and the engine guards
// both: selling back to the original seller is bayʿ al-ʿīnah (TAWARRUQ-1), and a sale the
// FINANCIER arranges is organized tawarruq munaẓẓam (TAWARRUQ-3, OIC Res. 179). The cyclic forms
// are also caught topologically by the composite checker; here we keep the licit form licit.
fn check_tawarruq(spec: &Spec, d: &mut Vec<Diagnostic>) {
    let cust = role_party(spec, "mustawriq").map(|p| p.name.clone());
    let fin = role_party(spec, "financier").map(|p| p.name.clone());
    let third = role_party(spec, "third_party").map(|p| p.name.clone());
    if cust.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "tawarruq requires a party with role 'mustawriq' (the one seeking cash)", C_TAWARRUQ));
    }
    if fin.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "tawarruq requires a party with role 'financier' (the credit seller of the commodity)", C_TAWARRUQ));
    }
    if third.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "tawarruq requires a party with role 'third_party' (the independent spot buyer)", C_TAWARRUQ));
    }
    // the spot buyer must differ from the financier — else it is a ring back to the seller ('inah).
    if let (Some(f), Some(t)) = (&fin, &third) {
        if f == t {
            d.push(Diagnostic::error("TAWARRUQ-1", spec.span, "the third-party spot buyer is the same as the credit seller — selling back to the seller is bayʿ al-ʿīnah", C_TAWARRUQ));
        }
    }

    match find_return(spec, "spot_sale") {
        None => d.push(Diagnostic::error(
            "TAWARRUQ-1",
            spec.span,
            "tawarruq requires returns { spot_sale { buyer; arranged_by; price } } — the onward sale to a third party",
            C_TAWARRUQ,
        )),
        Some(ss) => {
            // TAWARRUQ-1: the onward buyer must be the third party, never the financier.
            match ret_kv(ss, "buyer").and_then(|kv| kv.val.as_ident()) {
                Some(b) => {
                    let is_fin = Some(b) == fin.as_deref() || b == "financier";
                    let is_third = Some(b) == third.as_deref() || b == "third_party";
                    if is_fin {
                        d.push(Diagnostic::error("TAWARRUQ-1", ss.span, "the onward sale names the financier as buyer — selling back to the seller is bayʿ al-ʿīnah", C_TAWARRUQ));
                    } else if !is_third {
                        d.push(Diagnostic::error("TAWARRUQ-1", ss.span, format!("the onward sale buyer '{}' is not the declared independent third party", b), C_TAWARRUQ));
                    }
                }
                None => d.push(Diagnostic::error("TAWARRUQ-1", ss.span, "spot_sale must name the buyer (the independent third party)", C_TAWARRUQ)),
            }
            // TAWARRUQ-3: the onward sale must NOT be arranged by the financier (tawarruq munaẓẓam).
            match ret_kv(ss, "arranged_by").and_then(|kv| kv.val.as_ident()) {
                Some(a) if Some(a) == fin.as_deref() || a == "financier" => d.push(Diagnostic::error(
                    "TAWARRUQ-3",
                    ss.span,
                    "the onward sale is arranged by the financier — that is organized tawarruq (munaẓẓam), forbidden by OIC Fiqh Academy Res. 179 (19/5)",
                    C_TAWARRUQ,
                )),
                _ => {}
            }
        }
    }

    // TAWARRUQ-2 (qabd): possession must precede the onward sale.
    let lc = spec.lifecycle();
    let poss = lc.iter().position(|s| s.name == "takePossession");
    let sell = lc.iter().position(|s| s.name == "sellSpot");
    match (poss, sell) {
        (Some(p), Some(s)) if p < s => {}
        (None, _) => d.push(Diagnostic::error("TAWARRUQ-2", spec.span, "tawarruq requires a 'takePossession' step — the mutawarriq must possess (qabd) the commodity before reselling it", C_TAWARRUQ)),
        _ => d.push(Diagnostic::error("TAWARRUQ-2", spec.span, "in tawarruq 'takePossession' (qabd) must precede 'sellSpot'", C_TAWARRUQ)),
    }

    require_invariants(spec, &["onward_to_third_party", "possession_before_resale", "not_organized"], d);
}

// --- Qard Hasan (benevolent loan) ---
//
// The one pure-credit contract: the lender disburses, the borrower repays IN LIKE, and that is
// all. Any stipulated increase, or any fee/benefit conditioned on the loan, is riba — "kullu
// qarḍin jarra nafʿan fa-huwa riba". (An unstipulated gift at repayment is licit; the engine only
// forbids what is conditioned.) The borrower owns and is liable for the sum (it is ḍamān, not amana).
fn check_qard(spec: &Spec, d: &mut Vec<Diagnostic>) {
    let lender = role_party(spec, "lender").map(|p| p.name.clone());
    let borrower = role_party(spec, "borrower").map(|p| p.name.clone());
    if lender.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "qarḍ ḥasan requires a party with role 'lender'", C_QARD));
    }
    if borrower.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "qarḍ ḥasan requires a party with role 'borrower'", C_QARD));
    }
    if let (Some(l), Some(b)) = (&lender, &borrower) {
        if l == b {
            d.push(Diagnostic::error("PARTY-1", spec.span, "lender and borrower must be distinct parties", C_QARD));
        }
    }

    match find_return(spec, "loan") {
        None => d.push(Diagnostic::error(
            "QARD-1",
            spec.span,
            "qarḍ ḥasan requires returns { loan { principal; repayment; stipulated_increase; fee } }",
            C_QARD,
        )),
        Some(loan) => {
            // QARD-1: no stipulated increase — repayment is the principal, in like, no more.
            let repay_ok = ret_kv(loan, "repayment").and_then(|kv| kv.val.as_ident()) == Some("principal");
            let inc = ret_kv(loan, "stipulated_increase").and_then(|kv| kv.val.as_ident());
            if !repay_ok {
                d.push(Diagnostic::error("QARD-1", loan.span, "qarḍ ḥasan must declare repayment: principal — the loan is repaid in like, with no increase (any stipulated increase is riba)", C_QARD));
            }
            match inc {
                Some("none") => {}
                Some(other) => d.push(Diagnostic::error("QARD-1", loan.span, format!("stipulated_increase is '{}'; any increase conditioned on the loan is riba", other), C_QARD)),
                None => d.push(Diagnostic::error("QARD-1", loan.span, "qarḍ ḥasan must declare stipulated_increase: none", C_QARD)),
            }
            // QARD-2: no fee/benefit conditioned on the loan (a service fee that exceeds real cost is riba).
            match ret_kv(loan, "fee").and_then(|kv| kv.val.as_ident()) {
                Some("none") => {}
                Some(other) => d.push(Diagnostic::error("QARD-2", loan.span, format!("a fee '{}' is tied to the loan; 'every loan that draws a benefit is riba'", other), C_QARD)),
                None => d.push(Diagnostic::error("QARD-2", loan.span, "qarḍ ḥasan must declare fee: none (no benefit conditioned on the loan)", C_QARD)),
            }
            // principal must be a definite positive sum.
            match ret_kv(loan, "principal").and_then(|kv| kv.val.as_num()) {
                Some(n) if n > 0 => {}
                _ => d.push(Diagnostic::error("QARD-1", loan.span, "qarḍ ḥasan must declare a positive principal", C_QARD)),
            }
        }
    }

    require_invariants(spec, &["no_increase", "no_fee"], d);
}

// --- Rahn (pledge / collateral) ---
//
// The pledge (marhūn) secures a debt but is not a profit centre: the creditor takes no benefit
// from it (RAHN-1 — else it is riba on the loan), and on default it is sold to satisfy the debt
// with any surplus returning to the pledgor (RAHN-2 — the pledge is not forfeit). Held as amāna.
fn check_rahn(spec: &Spec, d: &mut Vec<Diagnostic>) {
    let pledgor = role_party(spec, "pledgor").map(|p| p.name.clone());
    let pledgee = role_party(spec, "pledgee").map(|p| p.name.clone());
    if pledgor.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "rahn requires a party with role 'pledgor' (al-rahin, the debtor who pledges)", C_RAHN));
    }
    if pledgee.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "rahn requires a party with role 'pledgee' (al-murtahin, the creditor who holds the pledge)", C_RAHN));
    }
    if let (Some(p), Some(q)) = (&pledgor, &pledgee) {
        if p == q {
            d.push(Diagnostic::error("PARTY-1", spec.span, "pledgor and pledgee must be distinct parties", C_RAHN));
        }
    }

    match find_return(spec, "pledge") {
        None => d.push(Diagnostic::error(
            "RAHN-1",
            spec.span,
            "rahn requires returns { pledge { debt; pledge_value; creditor_use; surplus } }",
            C_RAHN,
        )),
        Some(pl) => {
            // RAHN-1: the creditor derives NO benefit from the pledge (else riba on the loan).
            match ret_kv(pl, "creditor_use").and_then(|kv| kv.val.as_ident()) {
                Some("none") => {}
                Some(other) => d.push(Diagnostic::error("RAHN-1", pl.span, format!("creditor_use is '{}'; the creditor takes no benefit from the pledge — a benefit to the creditor is riba on the loan", other), C_RAHN)),
                None => d.push(Diagnostic::error("RAHN-1", pl.span, "rahn must declare creditor_use: none (the creditor takes no benefit from the pledge)", C_RAHN)),
            }
            // RAHN-2: on default the surplus over the debt returns to the pledgor (no forfeiture).
            match ret_kv(pl, "surplus").and_then(|kv| kv.val.as_ident()) {
                Some("to_pledgor") => {}
                Some(other) => d.push(Diagnostic::error("RAHN-2", pl.span, format!("surplus is '{}'; the pledge is not forfeit — the surplus over the debt returns to the pledgor", other), C_RAHN)),
                None => d.push(Diagnostic::error("RAHN-2", pl.span, "rahn must declare surplus: to_pledgor (the pledge is not forfeit)", C_RAHN)),
            }
            // definite debt and pledge value.
            let debt_ok = ret_kv(pl, "debt").and_then(|kv| kv.val.as_num()).map(|n| n > 0).unwrap_or(false);
            let val_ok = ret_kv(pl, "pledge_value").and_then(|kv| kv.val.as_num()).map(|n| n > 0).unwrap_or(false);
            if !debt_ok || !val_ok {
                d.push(Diagnostic::error("RAHN-3", pl.span, "rahn must declare a positive debt and pledge_value", C_RAHN));
            }
        }
    }

    require_invariants(spec, &["no_creditor_benefit", "surplus_to_pledgor"], d);
}

// --- Kafala (suretyship / guarantee) ---
//
// A gratuitous undertaking: the guarantor (kafil) assumes the debtor's obligation to the creditor.
// The majority forbid taking a FEE for the guarantee (KAFALA-1) — it is a benefit on lent credit and
// can mask riba. On paying, the guarantor recovers from the debtor exactly what he paid (KAFALA-2),
// never a surcharge.
fn check_kafala(spec: &Spec, d: &mut Vec<Diagnostic>) {
    for (role, who) in [("kafil", "guarantor"), ("principal_debtor", "debtor"), ("creditor", "creditor")] {
        if role_party(spec, role).is_none() {
            d.push(Diagnostic::error("PARTY-1", spec.span, format!("kafala requires a party with role '{}' ({})", role, who), C_KAFALA));
        }
    }
    let kafil = role_party(spec, "kafil").map(|p| p.name.clone());
    let debtor = role_party(spec, "principal_debtor").map(|p| p.name.clone());
    if let (Some(k), Some(db)) = (&kafil, &debtor) {
        if k == db {
            d.push(Diagnostic::error("PARTY-1", spec.span, "the guarantor (kafil) and the principal debtor must be distinct parties", C_KAFALA));
        }
    }

    match find_return(spec, "guarantee") {
        None => d.push(Diagnostic::error(
            "KAFALA-1",
            spec.span,
            "kafala requires returns { guarantee { amount; fee; recourse } }",
            C_KAFALA,
        )),
        Some(g) => {
            // KAFALA-1: no fee for the guarantee itself.
            match ret_kv(g, "fee").and_then(|kv| kv.val.as_ident()) {
                Some("none") => {}
                Some(other) => d.push(Diagnostic::error("KAFALA-1", g.span, format!("a fee '{}' is taken for the guarantee; the majority forbid a fee for kafala (a benefit on lent credit that can mask riba)", other), C_KAFALA)),
                None => d.push(Diagnostic::error("KAFALA-1", g.span, "kafala must declare fee: none (no fee for the guarantee)", C_KAFALA)),
            }
            // KAFALA-2: recourse to the debtor for exactly what was paid, no surcharge.
            match ret_kv(g, "recourse").and_then(|kv| kv.val.as_ident()) {
                Some("actual_paid") => {}
                Some(other) => d.push(Diagnostic::error("KAFALA-2", g.span, format!("recourse is '{}'; the guarantor recovers from the debtor exactly what he paid — a surcharge would be riba", other), C_KAFALA)),
                None => d.push(Diagnostic::error("KAFALA-2", g.span, "kafala must declare recourse: actual_paid (recover exactly what was paid)", C_KAFALA)),
            }
            match ret_kv(g, "amount").and_then(|kv| kv.val.as_num()) {
                Some(n) if n > 0 => {}
                _ => d.push(Diagnostic::error("KAFALA-1", g.span, "kafala must declare a positive guaranteed amount", C_KAFALA)),
            }
        }
    }

    require_invariants(spec, &["no_guarantee_fee", "recourse_actual"], d);
}

// --- Hawala (assignment / transfer of debt) ---
//
// The original debtor (muhil) refers his creditor (muhal) to a third party (muhal ʿalayh) who
// owes the muhil. On acceptance the original debtor is DISCHARGED (HAWALA-2). The transfer is for
// the LIKE of the debt — no increase (HAWALA-1), else it is a riba-bearing sale of debt.
fn check_hawala(spec: &Spec, d: &mut Vec<Diagnostic>) {
    for (role, who) in [("muhil", "original debtor"), ("muhal", "creditor"), ("muhal_alayh", "the new payer")] {
        if role_party(spec, role).is_none() {
            d.push(Diagnostic::error("PARTY-1", spec.span, format!("hawala requires a party with role '{}' ({})", role, who), C_HAWALA));
        }
    }

    match find_return(spec, "transfer") {
        None => d.push(Diagnostic::error(
            "HAWALA-1",
            spec.span,
            "hawala requires returns { transfer { debt; amount; discharge } }",
            C_HAWALA,
        )),
        Some(t) => {
            // HAWALA-1: the transferred amount equals the debt — no increase.
            let debt = ret_kv(t, "debt").and_then(|kv| kv.val.as_num());
            let amount = ret_kv(t, "amount").and_then(|kv| kv.val.as_num());
            match (debt, amount) {
                (Some(dv), Some(av)) if dv == av && dv > 0 => {}
                (Some(dv), Some(av)) => d.push(Diagnostic::error("HAWALA-1", t.span, format!("the transferred amount ({}) does not equal the debt ({}); a hawala transfers the LIKE of the debt — any increase is riba / sale of debt", av, dv), C_HAWALA)),
                _ => d.push(Diagnostic::error("HAWALA-1", t.span, "hawala must declare a positive debt and an equal transfer amount", C_HAWALA)),
            }
            // HAWALA-2: a valid hawala discharges the original debtor.
            match ret_kv(t, "discharge").and_then(|kv| kv.val.as_ident()) {
                Some("original_debtor") => {}
                Some(other) => d.push(Diagnostic::error("HAWALA-2", t.span, format!("discharge is '{}'; a valid hawala discharges the ORIGINAL debtor (al-muhil) toward the creditor", other), C_HAWALA)),
                None => d.push(Diagnostic::error("HAWALA-2", t.span, "hawala must declare discharge: original_debtor", C_HAWALA)),
            }
        }
    }

    require_invariants(spec, &["equal_transfer", "debtor_discharged"], d);
}

// --- Wadia (safekeeping deposit) ---
//
// The deposit is a trust (yad amāna): the custodian is not liable for its loss absent taʿaddī or
// taqṣīr (WADIA-1 — declaring it 'guaranteed' would turn it into a loan), and may not use it
// (WADIA-2). A fee for the safekeeping service is permitted and is not policed here.
fn check_wadia(spec: &Spec, d: &mut Vec<Diagnostic>) {
    let depositor = role_party(spec, "depositor").map(|p| p.name.clone());
    let custodian = role_party(spec, "custodian").map(|p| p.name.clone());
    if depositor.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "wadiʿa requires a party with role 'depositor' (al-mudiʿ)", C_WADIA));
    }
    if custodian.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "wadiʿa requires a party with role 'custodian' (al-mustawdaʿ)", C_WADIA));
    }
    if let (Some(a), Some(b)) = (&depositor, &custodian) {
        if a == b {
            d.push(Diagnostic::error("PARTY-1", spec.span, "depositor and custodian must be distinct parties", C_WADIA));
        }
    }

    match find_return(spec, "deposit") {
        None => d.push(Diagnostic::error(
            "WADIA-1",
            spec.span,
            "wadiʿa requires returns { deposit { amount; liability; custodian_use } }",
            C_WADIA,
        )),
        Some(dep) => {
            // WADIA-1: held as amāna (a trust), not guaranteed — else it becomes a loan (qarḍ).
            match ret_kv(dep, "liability").and_then(|kv| kv.val.as_ident()) {
                Some("amanah") => {}
                Some(other) => d.push(Diagnostic::error("WADIA-1", dep.span, format!("liability is '{}'; a deposit is held as amāna (trust) — guaranteeing its return turns it into a loan (qarḍ), and a conditioned benefit then becomes riba", other), C_WADIA)),
                None => d.push(Diagnostic::error("WADIA-1", dep.span, "wadiʿa must declare liability: amanah (a trust, not guaranteed)", C_WADIA)),
            }
            // WADIA-2: the custodian must not use the deposit.
            match ret_kv(dep, "custodian_use").and_then(|kv| kv.val.as_ident()) {
                Some("none") => {}
                Some(other) => d.push(Diagnostic::error("WADIA-2", dep.span, format!("custodian_use is '{}'; the custodian must not use the deposit (using it is taʿaddī and, if arranged, makes it a loan)", other), C_WADIA)),
                None => d.push(Diagnostic::error("WADIA-2", dep.span, "wadiʿa must declare custodian_use: none", C_WADIA)),
            }
            match ret_kv(dep, "amount").and_then(|kv| kv.val.as_num()) {
                Some(n) if n > 0 => {}
                _ => d.push(Diagnostic::error("WADIA-1", dep.span, "wadiʿa must declare a positive deposit amount", C_WADIA)),
            }
        }
    }

    require_invariants(spec, &["held_as_amanah", "no_custodian_use"], d);
}

// --- Wakala (agency, incl. investment agency) ---
//
// The agent (wakil) acts on the principal's account for a KNOWN fee and is a trustee, not a
// guarantor: he does not guarantee the capital or the profit (WAKALA-1 — a guarantee converts the
// agency into a riba-bearing loan). The fee must be disclosed (WAKALA-2 — gharar-free).
fn check_wakala(spec: &Spec, d: &mut Vec<Diagnostic>) {
    let principal = role_party(spec, "muwakkil").map(|p| p.name.clone());
    let agent = role_party(spec, "wakil").map(|p| p.name.clone());
    if principal.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "wakala requires a party with role 'muwakkil' (the principal)", C_WAKALA));
    }
    if agent.is_none() {
        d.push(Diagnostic::error("PARTY-1", spec.span, "wakala requires a party with role 'wakil' (the agent)", C_WAKALA));
    }
    if let (Some(p), Some(a)) = (&principal, &agent) {
        if p == a {
            d.push(Diagnostic::error("PARTY-1", spec.span, "principal and agent must be distinct parties", C_WAKALA));
        }
    }

    match find_return(spec, "agency") {
        None => d.push(Diagnostic::error(
            "WAKALA-1",
            spec.span,
            "wakala requires returns { agency { capital; fee; agent_guarantee } }",
            C_WAKALA,
        )),
        Some(ag) => {
            // WAKALA-1: the agent guarantees neither capital nor profit.
            match ret_kv(ag, "agent_guarantee").and_then(|kv| kv.val.as_ident()) {
                Some("none") => {}
                Some(other) => d.push(Diagnostic::error("WAKALA-1", ag.span, format!("agent_guarantee is '{}'; the agent (wakil) does not guarantee the capital or profit — a guarantee converts the agency into a riba-bearing loan", other), C_WAKALA)),
                None => d.push(Diagnostic::error("WAKALA-1", ag.span, "wakala must declare agent_guarantee: none", C_WAKALA)),
            }
            // WAKALA-2: the fee (ujra) must be DISCLOSED (a definite sum, possibly 0 for a gratuitous agency).
            if ret_kv(ag, "fee").and_then(|kv| kv.val.as_num()).is_none() {
                d.push(Diagnostic::error("WAKALA-2", ag.span, "wakala must declare a definite fee (ujra) — an undisclosed agency fee is gharar (use fee: 0 for a gratuitous agency)", C_WAKALA));
            }
            match ret_kv(ag, "capital").and_then(|kv| kv.val.as_num()) {
                Some(n) if n > 0 => {}
                _ => d.push(Diagnostic::error("WAKALA-1", ag.span, "wakala must declare a positive capital under management", C_WAKALA)),
            }
        }
    }

    require_invariants(spec, &["no_agent_guarantee", "fee_disclosed"], d);
}

// --- Ijarah Muntahia Bittamleek ---

fn check_ijarah(spec: &Spec, d: &mut Vec<Diagnostic>) {
    // rent is for usufruct, never on principal.
    match find_return(spec, "rent") {
        None => d.push(Diagnostic::error("RENT-2", spec.span, "ijarah requires a rent return for the usufruct of the asset", C_IJARAH)),
        Some(rent) => match kv_get(&rent.kvs, "basis") {
            None => d.push(Diagnostic::error("RENT-1", rent.span, "rent block has no 'basis'", "")),
            Some(b) if basis_is_principal(b) => d.push(Diagnostic::error(
                "RIBA-2",
                rent.span,
                "rent is charged on principal/capital — ijarah rent is the price of usufruct, not interest on money",
                C_RIBA,
            )),
            Some(b) if b.as_ident() != Some("usufruct") => d.push(Diagnostic::warn(
                "IJARAH-1",
                rent.span,
                format!("rent basis '{}' is not 'usufruct'; lease rent should price the usufruct of the asset", b.render()),
                C_IJARAH,
            )),
            Some(_) => {}
        },
    }

    // ownership risk rests on the lessor for the duration of the lease.
    match risk_get(spec, "loss") {
        Some(e) if e.as_ident() == Some("on_lessor") => {}
        Some(e) => d.push(Diagnostic::error(
            "RISK-3",
            spec.span,
            format!("ownership risk is '{}'; in ijarah the lessor (owner) bears the risk of the asset, not the lessee", e.render()),
            C_IJARAH,
        )),
        None => d.push(Diagnostic::error("RISK-3", spec.span, "ijarah must declare risk { loss: on_lessor }", C_IJARAH)),
    }

    // the transfer of ownership must be a SEPARATE contract/step, not bundled into the lease
    // (two contracts in one — bay' wa salaf — is prohibited).
    let has_transfer_step = spec.lifecycle().iter().any(|s| s.name == "transferOwnership");
    if !has_transfer_step {
        d.push(Diagnostic::error(
            "IJARAH-2",
            spec.span,
            "IMBT requires a distinct lifecycle step 'transferOwnership'; bundling sale into the lease combines two contracts in one (prohibited)",
            C_IJARAH,
        ));
    }

    // no late-payment penalty that accrues to the lessor as interest.
    if let Some(rent) = find_return(spec, "rent") {
        if let Some(p) = kv_get(&rent.kvs, "late_penalty") {
            if p.as_ident() != Some("none") {
                d.push(Diagnostic::error(
                    "RIBA-3",
                    rent.span,
                    "a late-payment penalty accruing to the lessor is interest on a debt (riba); any charge must go to charity, not the lessor",
                    C_RIBA,
                ));
            }
        }
    }

    require_invariants(
        spec,
        &["rent_for_usufruct", "lessor_bears_ownership_risk", "transfer_separate_from_lease", "no_late_penalty_interest"],
        d,
    );
}

// --- Commercial escrow (common law) — the universality claim + the judiciary engine ---

fn dispute_get<'a>(spec: &'a Spec, key: &str) -> Option<&'a Expr> {
    spec.dispute_cfg().into_iter().find(|k| k.key == key).map(|k| &k.val)
}

fn check_commercial(spec: &Spec, d: &mut Vec<Diagnostic>) {
    // parties: a depositor (payer), a beneficiary (payee), and an arbiter (the tribunal)
    for (role, desc) in [
        ("depositor", "depositor / payer"),
        ("beneficiary", "beneficiary / payee"),
        ("arbiter", "arbiter / tribunal"),
    ] {
        if role_party(spec, role).is_none() {
            d.push(Diagnostic::error(
                "PARTY-1",
                spec.span,
                format!("a commercial escrow requires a party with role '{}' ({})", role, desc),
                "",
            ));
        }
    }

    // consideration must move between DISTINCT parties
    let dep = role_party(spec, "depositor").map(|p| p.name.clone());
    let ben = role_party(spec, "beneficiary").map(|p| p.name.clone());
    if let (Some(dn), Some(bn)) = (&dep, &ben) {
        if dn == bn {
            d.push(Diagnostic::error(
                "CONSID-1",
                spec.span,
                "consideration must move between distinct parties; depositor and beneficiary are the same",
                C_CONSIDERATION,
            ));
        }
    }

    // certainty of terms + the penalty doctrine, from the `release` block
    match find_return(spec, "release") {
        None => d.push(Diagnostic::error(
            "TERMS-1",
            spec.span,
            "a commercial escrow requires returns { release { amount; condition; damages } }",
            C_CERTAINTY,
        )),
        Some(r) => {
            match kv_get(&r.kvs, "amount") {
                Some(e) if e.as_num().map(|n| n > 0).unwrap_or(false) => {}
                _ => d.push(Diagnostic::error(
                    "CERTAINTY-1",
                    r.span,
                    "the escrow amount must be a definite, non-zero sum (certainty of terms)",
                    C_CERTAINTY,
                )),
            }
            if kv_get(&r.kvs, "condition").and_then(|e| e.as_ident()).is_none() {
                d.push(Diagnostic::error(
                    "CERTAINTY-2",
                    r.span,
                    "the release condition must be definite",
                    C_CERTAINTY,
                ));
            }
            match kv_get(&r.kvs, "damages") {
                Some(e) if e.as_ident() == Some("liquidated") => {}
                Some(e) if e.as_ident() == Some("penalty") => d.push(Diagnostic::error(
                    "PENALTY-1",
                    r.span,
                    "a penalty clause is unenforceable; damages must be a genuine pre-estimate (liquidated), not a penalty in terrorem",
                    C_PENALTY,
                )),
                Some(_) => d.push(Diagnostic::error(
                    "PENALTY-1",
                    r.span,
                    "damages must be 'liquidated' (a genuine pre-estimate of loss)",
                    C_PENALTY,
                )),
                None => {}
            }
        }
    }

    // the dispute-resolution / judiciary engine must be present
    match dispute_get(spec, "remedy") {
        Some(e) if e.as_ident() == Some("arbiter_ruling") => {}
        _ => d.push(Diagnostic::error(
            "DISPUTE-1",
            spec.span,
            "a commercial contract must declare dispute { remedy: arbiter_ruling } — the arbitration / judiciary engine",
            C_GOODFAITH,
        )),
    }

    require_invariants(
        spec,
        &["certainty_of_terms", "no_penalty_clause", "consideration_present", "dispute_resolution_present"],
        d,
    );
}

// =====================================================================================
// Pluggable jurisprudence (Open Core pillar #2): the computation engine is separated from
// the rule-base. An authority (AAOIFI, DSN-MUI, a Common-Law panel) publishes a rule MODULE —
// pure data — and the same engine checks any spec against it. Ratification becomes a module,
// not a fork; the engine is regime-neutral and jurisdiction-agnostic.
// =====================================================================================

/// A rule-base published by some authority, loaded from a `*.rules.json` module.
pub struct RuleSet {
    pub authority: String,
    pub version: String,
    json: Value,
}

impl RuleSet {
    pub fn from_json(s: &str) -> Result<RuleSet, String> {
        let json: Value = serde_json::from_str(s).map_err(|e| format!("invalid rule module: {}", e))?;
        Ok(RuleSet {
            authority: json["authority"].as_str().unwrap_or("?").to_string(),
            version: json["version"].as_str().unwrap_or("?").to_string(),
            json,
        })
    }
    pub fn label(&self) -> String {
        format!("{} {}", self.authority, self.version)
    }
}

/// Resolve a dotted field path to the value the spec actually declares (as a string), so a
/// data-driven constraint `{field, op, value}` can be evaluated against it.
fn resolve_field(spec: &Spec, field: &str) -> Option<String> {
    let p: Vec<&str> = field.split('.').collect();
    match p.as_slice() {
        ["risk", k] => risk_get(spec, k).map(expr_to_string),
        ["dispute", k] => dispute_get(spec, k).map(expr_to_string),
        ["zakat", k] => zakat_get(spec, k).map(expr_to_string),
        ["returns", "buyout", "priceSource"] => {
            let b = find_return(spec, "buyout")?;
            let price = kv_get(&b.kvs, "price")?;
            let oracle = role_party(spec, "oracle").map(|x| x.name.clone());
            let attested = expr_mentions(price, "oracle")
                || oracle.as_deref().map(|n| expr_mentions(price, n)).unwrap_or(false);
            Some(if attested { "oracle".to_string() } else { "self".to_string() })
        }
        ["returns", mech, k] => {
            let r = find_return(spec, mech)?;
            kv_get(&r.kvs, k).map(expr_to_string)
        }
        _ => None,
    }
}

fn expr_to_string(e: &Expr) -> String {
    if let Some(id) = e.as_ident() {
        id.to_string()
    } else if let Some(n) = e.as_num() {
        n.to_string()
    } else if let Some(path) = e.as_path() {
        path.join(".")
    } else {
        e.render()
    }
}

fn want_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn eval_op(op: &str, got: Option<&str>, want: &Value) -> bool {
    let w = want_string(want);
    match op {
        "eq" => got.map(|g| g == w).unwrap_or(false),
        "ne" => got.map(|g| g != w).unwrap_or(true),
        "gt" => match (got.and_then(|g| g.parse::<i128>().ok()), w.parse::<i128>().ok()) {
            (Some(g), Some(t)) => g > t,
            _ => false,
        },
        _ => false,
    }
}

/// Check a spec against a pluggable rule-base. Same engine, any authority's module.
pub fn check_with_ruleset(spec: &Spec, rs: &RuleSet) -> Vec<Diagnostic> {
    let mut d = Vec::new();
    let class = Class::from_str(&spec.class);
    if let Class::Unknown(s) = &class {
        d.push(Diagnostic::error("CLASS-1", spec.span, format!("unknown instrument class '{}'", s), ""));
        return d;
    }
    let regime = class.regime();
    let entry = &rs.json["regimes"][regime]["classes"][&spec.class];
    if entry.is_null() {
        d.push(Diagnostic::error(
            "RULES-1",
            spec.span,
            format!("rule-base '{}' has no module for class '{}' (regime '{}')", rs.authority, spec.class, regime),
            "",
        ));
        return d;
    }

    if let Some(arr) = entry["required_invariants"].as_array() {
        for inv in arr {
            if let Some(name) = inv.as_str() {
                if !spec.has_invariant(name) {
                    d.push(Diagnostic::error(
                        "INV-1",
                        spec.span,
                        format!("[{}] required invariant '{}' is not declared", rs.authority, name),
                        "",
                    ));
                }
            }
        }
    }

    if let Some(arr) = entry["constraints"].as_array() {
        for c in arr {
            let code = c["code"].as_str().unwrap_or("RULE");
            let field = c["field"].as_str().unwrap_or("");
            let op = c["op"].as_str().unwrap_or("eq");
            let cite = c["citation"].as_str().unwrap_or("");
            let want = &c["value"];
            let got = resolve_field(spec, field);
            if !eval_op(op, got.as_deref(), want) {
                let gots = got.unwrap_or_else(|| "<undeclared>".to_string());
                d.push(Diagnostic::error(
                    code,
                    spec.span,
                    format!("[{}] expected {} {} {}, found '{}'", rs.authority, field, op, want_string(want), gots),
                    cite,
                ));
            }
        }
    }

    d
}
