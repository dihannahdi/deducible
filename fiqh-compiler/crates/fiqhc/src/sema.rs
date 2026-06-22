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

// --- citations (all flagged for human takhrij) ---
const C_RIBA: &str = "Qur'an al-Baqarah 2:275; AAOIFI Shari'ah Standard No. 12 [scholar-verify]";
const C_RISK: &str = "a partnership shares profit AND loss — AAOIFI SS No. 12 [scholar-verify]";
const C_GHARAR: &str = "prohibition of gharar (Sahih Muslim, Kitab al-Buyu') [scholar-verify]";
const C_MUDARABAH: &str = "AAOIFI Shari'ah Standard No. 13 (Mudarabah) [scholar-verify]";
const C_IJARAH: &str = "AAOIFI Shari'ah Standard No. 9 (Ijarah) [scholar-verify]";
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
    fn error(code: &str, span: Span, msg: impl Into<String>, citation: &str) -> Self {
        Diagnostic {
            code: code.to_string(),
            severity: Severity::Error,
            message: msg.into(),
            citation: citation.to_string(),
            span,
        }
    }
    fn warn(code: &str, span: Span, msg: impl Into<String>, citation: &str) -> Self {
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
    CommercialEscrow,
    Unknown(String),
}

impl Class {
    pub fn from_str(s: &str) -> Class {
        match s {
            "musharakah_mutanaqisah" => Class::MusharakahMutanaqisah,
            "mudarabah" => Class::Mudarabah,
            "ijarah_imbt" => Class::IjarahImbt,
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
        Class::CommercialEscrow => check_commercial(spec, &mut d),
        Class::Unknown(_) => {}
    }

    d
}

// --- shared helpers ---

fn meta_get<'a>(spec: &'a Spec, key: &str) -> Option<&'a Expr> {
    spec.meta().into_iter().find(|k| k.key == key).map(|k| &k.val)
}

fn risk_get<'a>(spec: &'a Spec, key: &str) -> Option<&'a Expr> {
    spec.risk().into_iter().find(|k| k.key == key).map(|k| &k.val)
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
    // RIBA-1: no guaranteed capital.
    match risk_get(spec, "capital_guarantee") {
        Some(e) if e.as_ident() == Some("none") => {}
        Some(e) => d.push(Diagnostic::error(
            "RIBA-1",
            spec.span,
            format!(
                "capital is guaranteed to '{}'; a guaranteed return of capital turns a partnership into an interest-bearing loan (riba)",
                e.render()
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

    // RISK-1: loss is shared proportional to ownership.
    match risk_get(spec, "loss") {
        Some(e) if e.as_ident() == Some("proportional_to_ownership") => {}
        Some(e) => d.push(Diagnostic::error(
            "RISK-1",
            spec.span,
            format!(
                "loss allocation is '{}'; a diminishing partnership must share loss proportional_to_ownership (no risk-sharing = no partnership)",
                e.render()
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
        Some(rent) => match kv_get(&rent.kvs, "basis") {
            None => d.push(Diagnostic::error("RENT-1", rent.span, "rent block has no 'basis'", "")),
            Some(b) if basis_is_principal(b) => d.push(Diagnostic::error(
                "RIBA-2",
                rent.span,
                "rent is charged on principal/capital — that is interest on a loan, not rent on a living share",
                C_RIBA,
            )),
            Some(b) => {
                let fin = role_party(spec, "financier").map(|p| p.name.clone());
                let ok = matches!(b.as_path(), Some(p) if p.len() == 2 && Some(p[0].clone()) == fin && p[1] == "share");
                if !ok {
                    d.push(Diagnostic::warn(
                        "RIBA-2",
                        rent.span,
                        format!("rent basis '{}' is not <financier>.share; rent should fall on the financier's living share", b.render()),
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
        Some(b) => match kv_get(&b.kvs, "price") {
            None => d.push(Diagnostic::error("BUYOUT-1", b.span, "buyout has no 'price'", "")),
            Some(price) => {
                let oracle = role_party(spec, "oracle").map(|p| p.name.clone());
                let attested = expr_mentions(price, "oracle")
                    || oracle.as_deref().map(|n| expr_mentions(price, n)).unwrap_or(false);
                if !attested {
                    d.push(Diagnostic::error(
                        "GHARAR-1",
                        b.span,
                        format!(
                            "buyout price '{}' is not derived from the independent oracle; a self-named or fixed price re-introduces gharar and can disguise a guaranteed return",
                            price.render()
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
