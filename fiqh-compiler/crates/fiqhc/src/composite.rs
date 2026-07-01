//! Graph-based invariant checker for composite contracts (al-'uqud al-murakkabah).
//!
//! A single leg may be impeccable in isolation; their *composition* can still encode a
//! ruse. The canonical case is **bay' al-'inah**: A sells an asset to B on a deferred,
//! marked-up price, then B sells the *same* asset back to A spot for less. No goods are
//! ever truly used — the round-trip exists only so A receives cash now and repays more
//! later. That is riba wearing the dress of two sales. **Organized tawarruq**
//! (tawarruq munazzam) is its cousin: the buyer immediately monetizes the same asset for
//! spot cash through a ring arranged by the financier.
//!
//! Neither is visible to a per-contract check — each sale, alone, is valid. They are
//! visible only in the *flow graph*. This module builds the directed asset-flow graph
//! across the legs of a bundle, enumerates its simple cycles, and refuses the structures
//! that constitute riba by composition. Detection is by graph topology + the time/price
//! asymmetry that carries the interest — not by the labels the legs wear.
//!
//! Epistemics unchanged: the engine proves a *structure* is inconsistent with a declared,
//! citation-bearing rule-base. It issues no fatwa. The classification of 'inah and
//! organized tawarruq as impermissible is the scholars' (cited below, flagged
//! `[scholar-verify]`); the engine only detects the structure they named.
//!
//! A boundary the graph analysis cannot lift itself over: it proves a ring absent *from the
//! legs it was given*. A financier who structures an organized tawarruq across several
//! institutions so that no single submitter sees the full ring defeats this by construction,
//! not by any flaw in the cycle search. Two mitigations follow, both of which raise the cost
//! of concealment rather than claim to remove it: (1) `BUNDLE-2` requires an explicit,
//! attributed completeness attestation before a bundle may be certified free of cycles, so
//! "no ring found" always carries a falsifiable claim of "and I attest I showed you
//! everything I know of" rather than silent, unattributed omission; (2) the dangling-leg
//! pass (`MAQASID-3`) flags a party who takes on a deferred debt for an asset with no
//! matching disposal leg anywhere in the bundle — the topological signature of a ring whose
//! remainder lives outside what was submitted.

use crate::ast::*;
use crate::sema::Diagnostic;
use std::collections::HashSet;

const C_INAH: &str =
    "bay' al-'inah (a sale followed by a buy-back of the same asset) is a recognised ruse for riba — \
     Sunan Abi Dawud, Kitab al-Buyu' (the hadith of 'inah); the athar of 'Aishah and Zayd b. Arqam (al-Daraqutni); \
     prohibited by the majority (Hanafi, Maliki, Hanbali) via sadd al-dhara'i' [scholar-verify]";
const C_TAWARRUQ: &str =
    "organized tawarruq (tawarruq munazzam) — buying on credit then monetizing the same asset for spot cash \
     through an arranged ring — is impermissible: OIC International Islamic Fiqh Academy, Resolution 179 (19/5), \
     Sharjah 2009; AAOIFI Shari'ah Standard No. 30 (Tawarruq) [scholar-verify]";

/// A normalized, validated leg: a directed transfer of `asset` from one party to another,
/// at `price`, settled spot or deferred.
struct FlowLeg {
    id: String,
    from: String,
    to: String,
    asset: String,
    deferred: bool,
    price: u64,
    span: Span,
}

fn leg_ident(leg: &Leg, key: &str) -> Option<String> {
    kv_get(&leg.kvs, key).and_then(|e| e.as_ident().map(|s| s.to_string()))
}

fn leg_num(leg: &Leg, key: &str) -> Option<u64> {
    kv_get(&leg.kvs, key).and_then(|e| e.as_num())
}

/// Parse and validate every leg into a `FlowLeg`, emitting LEG-* diagnostics for
/// missing/invalid fields and dangling party references.
fn build_flows(b: &Bundle, d: &mut Vec<Diagnostic>) -> Vec<FlowLeg> {
    let party_names: Vec<String> = b.parties().iter().map(|p| p.name.clone()).collect();
    let mut flows = Vec::new();
    for leg in b.legs() {
        let from = leg_ident(leg, "from");
        let to = leg_ident(leg, "to");
        let asset = leg_ident(leg, "asset");
        let payment = leg_ident(leg, "payment").unwrap_or_else(|| "spot".to_string());
        let price = leg_num(leg, "price");

        if from.is_none() || to.is_none() || asset.is_none() {
            d.push(Diagnostic::error(
                "LEG-1",
                leg.span,
                format!("leg '{}' must declare from, to, and asset", leg.id),
                "",
            ));
            continue;
        }
        if payment != "spot" && payment != "deferred" {
            d.push(Diagnostic::error(
                "LEG-2",
                leg.span,
                format!("leg '{}' payment must be 'spot' or 'deferred', found '{}'", leg.id, payment),
                "",
            ));
        }
        let (from, to, asset) = (from.unwrap(), to.unwrap(), asset.unwrap());
        if !party_names.is_empty() {
            for (who, role) in [(&from, "from"), (&to, "to")] {
                if !party_names.contains(who) {
                    d.push(Diagnostic::error(
                        "LEG-3",
                        leg.span,
                        format!("leg '{}' references undeclared party '{}' in '{}'", leg.id, who, role),
                        "",
                    ));
                }
            }
        }
        if from == to {
            d.push(Diagnostic::error(
                "LEG-4",
                leg.span,
                format!("leg '{}' transfers an asset from a party to itself", leg.id),
                "",
            ));
        }
        flows.push(FlowLeg {
            id: leg.id.clone(),
            from,
            to,
            asset,
            deferred: payment == "deferred",
            price: price.unwrap_or(0),
            span: leg.span,
        });
    }
    flows
}

/// Enumerate the simple cycles among `edges` (each `(leg_index, from, to)`), deduplicated
/// by their edge set. Bundles are small, so a depth-first walk is more than adequate.
fn simple_cycles(nodes: &[String], edges: &[(usize, String, String)]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    let mut seen: HashSet<Vec<usize>> = HashSet::new();
    for start in nodes {
        let mut path_legs: Vec<usize> = Vec::new();
        let mut path_nodes: Vec<String> = vec![start.clone()];
        walk(start, start, edges, &mut path_legs, &mut path_nodes, &mut out, &mut seen);
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn walk(
    start: &str,
    current: &str,
    edges: &[(usize, String, String)],
    path_legs: &mut Vec<usize>,
    path_nodes: &mut Vec<String>,
    out: &mut Vec<Vec<usize>>,
    seen: &mut HashSet<Vec<usize>>,
) {
    for (idx, from, to) in edges {
        if from != current || path_legs.contains(idx) {
            continue;
        }
        if to == start {
            let mut cyc = path_legs.clone();
            cyc.push(*idx);
            let mut canon = cyc.clone();
            canon.sort_unstable();
            if seen.insert(canon) {
                out.push(cyc);
            }
            continue;
        }
        if path_nodes.iter().any(|n| n == to) {
            continue; // keep the path simple
        }
        path_legs.push(*idx);
        path_nodes.push(to.clone());
        walk(start, to, edges, path_legs, path_nodes, out, seen);
        path_legs.pop();
        path_nodes.pop();
    }
}

/// The result of analysing a bundle's flow graph — also serialised into the composite
/// invariant manifest so an off-chain gateway can re-check the same structure.
pub struct FlowReport {
    pub cycles: Vec<Vec<usize>>,
    pub flagged: bool,
}

/// Run the graph-based invariant checker over a bundle.
pub fn check_bundle(b: &Bundle) -> Vec<Diagnostic> {
    let mut d = Vec::new();

    if b.meta().iter().all(|k| k.key != "basis") {
        d.push(Diagnostic::warn(
            "META-1",
            b.span,
            "no fiqh basis cited; declare meta { basis: \"...\"; } and have a scholar ratify the composition",
            "",
        ));
    }

    let completeness_ok = b
        .meta()
        .iter()
        .find(|k| k.key == "completeness_attestation")
        .and_then(|k| k.val.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if !completeness_ok {
        d.push(Diagnostic::error(
            "BUNDLE-2",
            b.span,
            "a bundle must declare meta { completeness_attestation: \"...\"; }, an attributed \
             statement that the submitting party has represented every leg of this transaction \
             known to them. The engine can prove a ring absent from the legs it was given; it \
             cannot prove a ring absent from legs it was never shown. A cycle-free verdict without \
             this attestation is not a verdict — it is silence about what was omitted, by whom.",
            "",
        ));
    }

    let flows = build_flows(b, &mut d);
    if flows.is_empty() {
        d.push(Diagnostic::error(
            "BUNDLE-1",
            b.span,
            "a bundle must declare at least one leg in legs { ... }",
            "",
        ));
        return d;
    }

    analyze_flows(&flows, b.span, &mut d);
    d
}

/// The pure graph analysis, separated so the manifest builder can reuse it.
fn analyze_flows(flows: &[FlowLeg], bundle_span: Span, d: &mut Vec<Diagnostic>) -> FlowReport {
    // Distinct assets and the party nodes that touch each.
    let assets: Vec<String> = {
        let mut s: Vec<String> = Vec::new();
        for f in flows {
            if !s.contains(&f.asset) {
                s.push(f.asset.clone());
            }
        }
        s
    };

    let mut all_cycles: Vec<Vec<usize>> = Vec::new();
    // (party, asset) pairs already implicated by an 'inah return-cycle, so the
    // monetization pass does not double-report the same structure.
    let mut inah_pairs: HashSet<(String, String)> = HashSet::new();

    for asset in &assets {
        // Edges that move THIS asset, indexed back into `flows`.
        let edges: Vec<(usize, String, String)> = flows
            .iter()
            .enumerate()
            .filter(|(_, f)| &f.asset == asset)
            .map(|(i, f)| (i, f.from.clone(), f.to.clone()))
            .collect();
        if edges.len() < 2 {
            continue;
        }
        let mut nodes: Vec<String> = Vec::new();
        for (_, fr, to) in &edges {
            for n in [fr, to] {
                if !nodes.contains(n) {
                    nodes.push(n.clone());
                }
            }
        }

        for cyc in simple_cycles(&nodes, &edges) {
            let legs: Vec<&FlowLeg> = cyc.iter().map(|&i| &flows[i]).collect();
            let has_deferred = legs.iter().any(|f| f.deferred);
            let prices: Vec<u64> = legs.iter().map(|f| f.price).collect();
            let price_differential = prices.iter().min() != prices.iter().max();
            let origin = legs[0].from.clone();
            let ids: Vec<String> = legs.iter().map(|f| f.id.clone()).collect();
            let span = legs.last().map(|f| f.span).unwrap_or(bundle_span);

            if has_deferred && price_differential {
                if cyc.len() == 2 {
                    // P -> Q (one leg deferred), Q -> P: the same asset returns to its
                    // origin at a different price across time. The gap is the interest.
                    d.push(Diagnostic::error(
                        "INAH-1",
                        span,
                        format!(
                            "bay' al-'inah detected: asset '{}' is sold by '{}' and bought back along legs [{}] at a different price, with a deferred settlement. The round-trip has no economic substance; the price gap across time is riba.",
                            asset,
                            origin,
                            ids.join(", ")
                        ),
                        C_INAH,
                    ));
                } else {
                    // Longer ring that returns the asset to its origin through
                    // intermediaries with a deferred, marked-up leg: organized tawarruq.
                    d.push(Diagnostic::error(
                        "INAH-2",
                        span,
                        format!(
                            "organized-tawarruq ring detected: asset '{}' cycles back to '{}' through legs [{}] with a deferred, marked-up leg. The ring monetizes a debt — riba by composition.",
                            asset,
                            origin,
                            ids.join(", ")
                        ),
                        C_TAWARRUQ,
                    ));
                }
                for f in &legs {
                    inah_pairs.insert((f.from.clone(), asset.clone()));
                    inah_pairs.insert((f.to.clone(), asset.clone()));
                }
            } else {
                // A round-trip with no time/price asymmetry: no riba is proven, but a
                // circular transfer with no net economic effect is a formalistic red flag.
                d.push(Diagnostic::warn(
                    "INAH-3",
                    span,
                    format!(
                        "asset '{}' makes a round-trip along legs [{}] with no net economic transfer; verify this composition is not a formalistic ruse (hila).",
                        asset,
                        ids.join(", ")
                    ),
                    C_INAH,
                ));
            }
            all_cycles.push(cyc);
        }
    }

    // Monetization pass (catches tawarruq where the asset is flipped to a THIRD party for
    // cash rather than sold back to the financier): a party that acquires an asset on
    // deferred terms and disposes of the SAME asset spot is monetizing a debt.
    let parties: Vec<String> = {
        let mut s: Vec<String> = Vec::new();
        for f in flows {
            for p in [&f.from, &f.to] {
                if !s.contains(p) {
                    s.push(p.clone());
                }
            }
        }
        s
    };
    for party in &parties {
        for asset in &assets {
            if inah_pairs.contains(&(party.clone(), asset.clone())) {
                continue;
            }
            let in_deferred = flows.iter().find(|f| &f.to == party && &f.asset == asset && f.deferred);
            let out_spot = flows.iter().find(|f| &f.from == party && &f.asset == asset && !f.deferred);
            if let (Some(inf), Some(outf)) = (in_deferred, out_spot) {
                d.push(Diagnostic::error(
                    "INAH-2",
                    outf.span,
                    format!(
                        "monetization detected: '{}' acquires asset '{}' on deferred terms (leg '{}') and disposes of the same asset for spot cash (leg '{}'). Receiving cash now against a larger deferred debt is organized tawarruq.",
                        party, asset, inf.id, outf.id
                    ),
                    C_TAWARRUQ,
                ));
            }
        }
    }

    // Dangling-leg pass: a party who takes on a deferred debt for an asset with NO matching
    // disposal leg anywhere in this bundle — neither a buy-back (INAH-1/2) nor a spot flip
    // (the monetization pass above, which requires BOTH legs present to fire). Absence of a
    // ring in this bundle is not proof of absence: the debtor's next move may be booked at a
    // different institution or venue this bundle does not represent. This is a maqsad-risk
    // signal, not a structural violation — it never blocks, only names what to ask about.
    for party in &parties {
        for asset in &assets {
            if inah_pairs.contains(&(party.clone(), asset.clone())) {
                continue;
            }
            let in_deferred = flows.iter().find(|f| &f.to == party && &f.asset == asset && f.deferred);
            let has_any_disposal = flows.iter().any(|f| &f.from == party && &f.asset == asset);
            if let Some(inf) = in_deferred {
                if !has_any_disposal {
                    d.push(Diagnostic::warn(
                        "MAQASID-3",
                        inf.span,
                        format!(
                            "'{}' takes on asset '{}' via deferred leg '{}' with no disposal of that \
                             asset declared anywhere in this bundle. This bundle cannot see what happens \
                             to it next. If the disposal is booked through a venue or party not \
                             represented here, an organized-tawarruq ring could complete outside this \
                             bundle's view. A scholar or auditor should confirm the disposal, if any, \
                             independently.",
                            party, asset, inf.id
                        ),
                        "the completeness of a bundle is what its submitter attests, not what the \
                         graph search can independently verify [scholar-verify]",
                    ));
                }
            }
        }
    }

    let flagged = d.iter().any(|x| x.is_error());
    FlowReport { cycles: all_cycles, flagged }
}

/// Build the composite invariant manifest: the flow graph plus the verdict, so an
/// off-chain gateway can enforce the same structural invariant a ledger cannot express.
pub fn build_manifest(b: &Bundle) -> String {
    let mut diags = Vec::new();
    let flows = build_flows(b, &mut diags);
    let report = analyze_flows(&flows, b.span, &mut diags);

    let legs_json: Vec<serde_json::Value> = flows
        .iter()
        .map(|f| {
            serde_json::json!({
                "id": f.id,
                "from": f.from,
                "to": f.to,
                "asset": f.asset,
                "payment": if f.deferred { "deferred" } else { "spot" },
                "price": f.price,
            })
        })
        .collect();

    let violations: Vec<serde_json::Value> = diags
        .iter()
        .filter(|x| x.is_error())
        .map(|x| {
            serde_json::json!({
                "code": x.code,
                "message": x.message,
                "citation": x.citation,
            })
        })
        .collect();

    let maqasid_warnings: Vec<serde_json::Value> = diags
        .iter()
        .filter(|x| !x.is_error() && x.code.starts_with("MAQASID"))
        .map(|x| serde_json::json!({ "code": x.code, "message": x.message, "citation": x.citation }))
        .collect();

    let completeness_attestation = b
        .meta()
        .iter()
        .find(|k| k.key == "completeness_attestation")
        .and_then(|k| k.val.as_str())
        .map(|s| s.to_string());

    let manifest = serde_json::json!({
        "kind": "composite_invariant_manifest",
        "bundle": b.name,
        "legs": legs_json,
        "cycles_detected": report.cycles.len(),
        "consistent": !report.flagged,
        "violations": violations,
        "completeness_attestation": completeness_attestation,
        "maqasid_warnings": maqasid_warnings,
        "note": "no riba cycle (bay' al-'inah / organized tawarruq) by composition, AMONG THE LEGS ATTESTED COMPLETE; consistency is not a fatwa, and completeness is the submitter's claim, not an independently verified fact",
    });
    serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".to_string())
}
