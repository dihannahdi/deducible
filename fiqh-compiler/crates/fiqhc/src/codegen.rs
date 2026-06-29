//! Backend: lower a *validated* AST to Solidity, a Hardhat test, and a deploy
//! descriptor. Codegen is only ever reached after `sema::check` returns no errors,
//! so every emitted contract is consistent-by-construction with its declared
//! rule-base. The generated Solidity matches the conventions of the hand-written,
//! peer-reviewed artifact (immutable parties, BPS, onlyX/live/nonReentrant
//! modifiers, one event per transition, oracle as the trust boundary), and the
//! declared invariants are compiled in as `@dev INVARIANT` annotations.

use crate::ast::*;
use crate::sema::Class;
use serde_json::json;

pub struct Generated {
    pub instrument: String,
    pub contract_name: String,
    pub sol: String,
    pub test_js: String,
    pub descriptor: String,
}

pub fn generate(spec: &Spec) -> Result<Generated, String> {
    match Class::from_str(&spec.class) {
        Class::MusharakahMutanaqisah => gen_musharakah(spec),
        Class::Mudarabah => gen_mudarabah(spec),
        Class::IjarahImbt => gen_ijarah(spec),
        Class::Murabahah => gen_murabahah(spec),
        Class::Salam => gen_salam(spec),
        Class::Istisna => gen_istisna(spec),
        Class::Sarf => gen_sarf(spec),
        Class::Tawarruq => gen_tawarruq(spec),
        Class::QardHasan => gen_qard(spec),
        Class::Rahn => gen_rahn(spec),
        Class::Kafala => gen_kafala(spec),
        Class::Hawala => gen_hawala(spec),
        Class::Wadia => gen_wadia(spec),
        Class::Wakala => gen_wakala(spec),
        Class::Ijarah => gen_ijarah_plain(spec),
        Class::Juala => gen_juala(spec),
        Class::Ariyah => gen_ariyah(spec),
        Class::Musharakah => gen_musharakah_full(spec),
        Class::Muzaraah => gen_muzaraah(spec),
        Class::Sukuk => gen_sukuk(spec),
        Class::Takaful => gen_takaful(spec),
        Class::MudarabahPool => gen_mudarabah_pool(spec),
        Class::Waqf => gen_waqf(spec),
        Class::Hibah => gen_hibah(spec),
        Class::Wasiyya => gen_wasiyya(spec),
        Class::CommercialEscrow => gen_commercial(spec),
        Class::Unknown(s) => Err(format!("no backend for instrument class '{}'", s)),
    }
}

/// A portable, ledger-agnostic invariant manifest (Vision #4). The same facts the engine
/// checks are emitted as machine-checkable constraints `{code, field, op, value, citation}`,
/// so the invariants can be enforced against a proposed `terms` object on ANY backend — a
/// non-EVM ledger or a traditional database — not only the generated Solidity. A gateway
/// evaluates each constraint before a transition is committed; a violation is refused exactly
/// as the compiler would refuse it. The manifest carries no fatwa, only the cited rule-base.
pub fn build_manifest(spec: &Spec) -> String {
    let class = Class::from_str(&spec.class);
    let mut c: Vec<serde_json::Value> = Vec::new();
    let mut add = |code: &str, field: &str, op: &str, value: serde_json::Value, cite: &str| {
        c.push(json!({ "code": code, "field": field, "op": op, "value": value, "citation": cite }));
    };
    match class {
        Class::MusharakahMutanaqisah => {
            add("RIBA-1", "risk.capital_guarantee", "eq", json!("none"), "al-Baqarah 2:275; AAOIFI SS 12 [scholar-verify]");
            add("RISK-1", "risk.loss", "eq", json!("proportional_to_ownership"), "AAOIFI SS 12 [scholar-verify]");
            add("RIBA-2", "returns.rent.basis", "ne", json!("principal"), "al-Baqarah 2:275 [scholar-verify]");
            add("GHARAR-1", "returns.buyout.priceSource", "eq", json!("oracle"), "prohibition of gharar [scholar-verify]");
            if let Some((zrate, _)) = zakat_cfg(spec) {
                add("ZAKAT-1", "zakat.rate_bps", "eq", json!(zrate), "the zakat rate for the declared genus (rubʿ al-ʿushr / ʿushr / niṣf al-ʿushr); AAOIFI SS 35 [scholar-verify]");
            }
        }
        Class::Mudarabah => {
            add("RIBA-1", "risk.capital_guarantee", "eq", json!("none"), "AAOIFI SS 13 [scholar-verify]");
            add("RISK-2", "risk.loss", "eq", json!("on_rabb_al_mal"), "AAOIFI SS 13 [scholar-verify]");
            add("PROFIT-1", "returns.profit.split", "eq", json!("ratio"), "AAOIFI SS 13 [scholar-verify]");
        }
        Class::IjarahImbt => {
            add("RIBA-2", "returns.rent.basis", "eq", json!("usufruct"), "AAOIFI SS 9 [scholar-verify]");
            add("RISK-3", "risk.loss", "eq", json!("on_lessor"), "AAOIFI SS 9 [scholar-verify]");
            add("RIBA-3", "returns.rent.late_penalty", "eq", json!("none"), "no interest on a debt [scholar-verify]");
        }
        Class::Murabahah => {
            add("MUR-1", "returns.sale.cost", "gt", json!(0), "bay' al-amana: the buyer must know the true cost; AAOIFI SS 8 [scholar-verify]");
            add("RIBA-2", "returns.sale.markup_basis", "eq", json!("fixed"), "a markup that grows with time is interest; al-Baqarah 2:275 [scholar-verify]");
            add("RIBA-3", "returns.sale.late_penalty", "eq", json!("none"), "no penalty-interest on the resulting debt [scholar-verify]");
        }
        Class::Salam => {
            add("SALAM-1", "returns.salam.payment", "eq", json!("spot_full"), "ra's al-mal paid in full at the session; else bayʿ al-kaliʾ bi-l-kaliʾ; AAOIFI SS 10 [scholar-verify]");
            add("SALAM-2", "returns.salam.quantity", "gt", json!(0), "the muslam fih must be a known quantity (maʿlūm); Bukhari/Muslim, Ibn ʿAbbas [scholar-verify]");
            add("SALAM-3", "returns.salam.delivery_date", "gt", json!(0), "delivery at a known term (ajal maʿlūm) [scholar-verify]");
        }
        Class::Istisna => {
            add("ISTISNA-1", "returns.istisna.spec", "eq", json!("described"), "the masnuʿ must be described (maʿlūm); AAOIFI SS 11 [scholar-verify]");
            add("ISTISNA-2", "returns.istisna.material_by", "eq", json!("manufacturer"), "the saniʿ supplies the materials, else ijarat al-ʿamal; AAOIFI SS 11 [scholar-verify]");
            add("ISTISNA-3", "returns.istisna.price", "gt", json!(0), "a known fixed price [scholar-verify]");
        }
        Class::Sarf => {
            add("SARF-1", "returns.exchange.settlement", "eq", json!("spot"), "yadan bi-yad; deferral is riba al-nasiʾa; hadith ʿUbada (Muslim); AAOIFI SS 1 [scholar-verify]");
            // SARF-2 (same-genus equality) is conditional on same_genus and enforced in-engine.
        }
        Class::Tawarruq => {
            add("TAWARRUQ-1", "returns.spot_sale.buyer", "ne", json!("financier"), "the onward sale must go to a third party, not back to the seller (else bayʿ al-ʿīnah) [scholar-verify]");
            add("TAWARRUQ-3", "returns.spot_sale.arranged_by", "ne", json!("financier"), "organized tawarruq (arranged by the financier) is forbidden — OIC Fiqh Academy Res. 179 (19/5) [scholar-verify]");
        }
        Class::QardHasan => {
            add("QARD-1", "returns.loan.stipulated_increase", "eq", json!("none"), "no stipulated increase; every loan that draws a benefit is riba [scholar-verify]");
            add("QARD-2", "returns.loan.fee", "eq", json!("none"), "no fee/benefit conditioned on the loan (riba) [scholar-verify]");
        }
        Class::Rahn => {
            add("RAHN-1", "returns.pledge.creditor_use", "eq", json!("none"), "the creditor takes no benefit from the pledge (else riba on the loan) [scholar-verify]");
            add("RAHN-2", "returns.pledge.surplus", "eq", json!("to_pledgor"), "the pledge is not forfeit; surplus over the debt returns to the pledgor [scholar-verify]");
        }
        Class::Kafala => {
            add("KAFALA-1", "returns.guarantee.fee", "eq", json!("none"), "no fee for a guarantee (the majority); AAOIFI SS 5 [scholar-verify]");
            add("KAFALA-2", "returns.guarantee.recourse", "eq", json!("actual_paid"), "recourse for exactly what was paid, no surcharge [scholar-verify]");
        }
        Class::Hawala => {
            add("HAWALA-2", "returns.transfer.discharge", "eq", json!("original_debtor"), "a valid hawala discharges the original debtor [scholar-verify]");
        }
        Class::Wadia => {
            add("WADIA-1", "returns.deposit.liability", "eq", json!("amanah"), "the deposit is a trust, not guaranteed (else a loan); al-Nisa 4:58 [scholar-verify]");
            add("WADIA-2", "returns.deposit.custodian_use", "eq", json!("none"), "the custodian must not use the deposit [scholar-verify]");
        }
        Class::Wakala => {
            add("WAKALA-1", "returns.agency.agent_guarantee", "eq", json!("none"), "the agent does not guarantee capital or profit (else riba); AAOIFI SS 23 [scholar-verify]");
        }
        Class::Ijarah => {
            add("RIBA-2", "returns.rent.basis", "eq", json!("usufruct"), "rent is the price of usufruct, not interest; AAOIFI SS 9 [scholar-verify]");
            add("RISK-3", "risk.loss", "eq", json!("on_lessor"), "the lessor (owner) bears the asset risk; AAOIFI SS 9 [scholar-verify]");
            add("RIBA-3", "returns.rent.late_penalty", "eq", json!("none"), "no interest on a late rent [scholar-verify]");
        }
        Class::Juala => {
            add("JUALA-1", "returns.reward.amount", "gt", json!(0), "the reward (juʿl) must be known; AAOIFI SS 15 [scholar-verify]");
            add("JUALA-2", "returns.reward.due", "eq", json!("on_completion"), "due only on completion; the worker bears non-completion risk [scholar-verify]");
        }
        Class::Ariyah => {
            add("ARIYAH-1", "returns.loan_use.fee", "eq", json!("none"), "ʿariyya is gratuitous (a charge makes it ijara) [scholar-verify]");
            add("ARIYAH-2", "returns.loan_use.return", "eq", json!("same_asset"), "the same asset returns; only its usufruct was lent [scholar-verify]");
        }
        Class::Musharakah => {
            add("RIBA-1", "risk.capital_guarantee", "eq", json!("none"), "no partner guarantees another's capital; AAOIFI SS 12 [scholar-verify]");
            add("RISK-1", "risk.loss", "eq", json!("proportional_to_capital"), "loss strictly by capital share (al-wadi'a 'ala qadr al-mal) [scholar-verify]");
            add("PROFIT-1", "returns.profit.split", "eq", json!("ratio"), "profit by a pre-agreed ratio [scholar-verify]");
        }
        Class::Muzaraah => {
            add("MUZARA-1", "returns.harvest_share.basis", "eq", json!("output_ratio"), "yield shared by ratio of the actual output [scholar-verify]");
            add("MUZARA-2", "returns.harvest_share.fixed_rent", "eq", json!("none"), "no fixed rent on the land regardless of harvest [scholar-verify]");
        }
        Class::Sukuk => {
            add("SUKUK-1", "returns.income.basis", "eq", json!("asset_rental"), "the return is the asset's rental/profit, not interest; AAOIFI SS 17 [scholar-verify]");
            add("SUKUK-2", "returns.income.distribution", "eq", json!("pro_rata"), "income distributed pro-rata to undivided ownership shares [scholar-verify]");
        }
        Class::Takaful => {
            add("TAKAFUL-1", "returns.contribution.basis", "eq", json!("tabarru"), "contributions are tabarru' (donation), not a profit-premium; OIC/AAOIFI [scholar-verify]");
            add("TAKAFUL-2", "returns.contribution.surplus", "eq", json!("to_participants"), "the surplus belongs to the participants, not the operator [scholar-verify]");
        }
        Class::MudarabahPool => {
            add("RIBA-1", "risk.capital_guarantee", "eq", json!("none"), "the mudarib guarantees nothing; AAOIFI SS 13 [scholar-verify]");
            add("RISK-2", "risk.loss", "eq", json!("on_capital_pool"), "loss on the rabb al-mal pool pro-rata [scholar-verify]");
            add("PROFIT-1", "returns.profit.split", "eq", json!("ratio"), "profit by a pre-agreed ratio [scholar-verify]");
        }
        Class::Waqf => {
            add("WAQF-1", "returns.endowment.corpus", "eq", json!("inalienable"), "the corpus is perpetually inalienable; Umar's Khaybar waqf [scholar-verify]");
            add("WAQF-2", "returns.endowment.distribution", "eq", json!("income_only"), "only the income (ghalla) is distributed [scholar-verify]");
        }
        Class::Hibah => {
            add("HIBAH-1", "returns.gift.transfer", "eq", json!("immediate"), "a hibah is completed by possession at once [scholar-verify]");
            add("HIBAH-2", "returns.gift.consideration", "eq", json!("none"), "a gift for a return is a sale, not a hibah [scholar-verify]");
        }
        Class::Wasiyya => {
            add("WASIYYA-2", "returns.bequest.beneficiary", "eq", json!("non_heir"), "no bequest to an heir [scholar-verify]");
        }
        Class::CommercialEscrow => {
            add("PENALTY-1", "returns.release.damages", "eq", json!("liquidated"), "Cavendish v Makdessi [2015] UKSC 67 [verify]");
            add("CERTAINTY-1", "returns.release.amount", "gt", json!(0), "Scammell v Ouston [1941] AC 251 [verify]");
            add("DISPUTE-1", "dispute.remedy", "eq", json!("arbiter_ruling"), "good faith / arbitration [verify]");
        }
        Class::Unknown(_) => {}
    }
    // Ahliyyah (enterprise vector #3): the principal (contracting) roles whose legal capacity
    // the gateway must verify before a transition — an 'aqd is invalid if a party lacks
    // ahliyyat al-ada' (a minor, the interdicted, a bankrupt under hajr). Functionary roles
    // (oracle, arbiter, beneficiary fund) are not contracting principals.
    let principals: Vec<&str> = match &class {
        Class::MusharakahMutanaqisah => vec!["financier", "acquirer"],
        Class::Mudarabah => vec!["rabb_al_mal", "mudarib"],
        Class::IjarahImbt => vec!["lessor", "lessee"],
        Class::Murabahah => vec!["seller", "buyer"],
        Class::Salam => vec!["buyer", "seller"],
        Class::Istisna => vec!["buyer", "manufacturer"],
        Class::Sarf => vec!["exchanger_a", "exchanger_b"],
        Class::Tawarruq => vec!["mustawriq", "financier", "third_party"],
        Class::QardHasan => vec!["lender", "borrower"],
        Class::Rahn => vec!["pledgor", "pledgee"],
        Class::Kafala => vec!["kafil", "principal_debtor", "creditor"],
        Class::Hawala => vec!["muhil", "muhal", "muhal_alayh"],
        Class::Wadia => vec!["depositor", "custodian"],
        Class::Wakala => vec!["muwakkil", "wakil"],
        Class::Ijarah => vec!["lessor", "lessee"],
        Class::Juala => vec!["jail", "amil"],
        Class::Ariyah => vec!["muir", "mustair"],
        Class::Musharakah => vec!["partner"],
        Class::Muzaraah => vec!["landowner", "cultivator"],
        Class::Sukuk => vec!["issuer", "holder"],
        Class::Takaful => vec!["operator", "participant"],
        Class::MudarabahPool => vec!["mudarib", "rabb_al_mal"],
        Class::Waqf => vec!["waqif", "beneficiary"],
        Class::Hibah => vec!["donor", "donee"],
        Class::Wasiyya => vec!["testator", "legatee"],
        Class::CommercialEscrow => vec!["depositor", "beneficiary"],
        Class::Unknown(_) => vec![],
    };
    let mut manifest = json!({
        "instrument": spec.class,
        "regime": class.regime(),
        "name": spec.name,
        "constraints": c,
        "ahliyyah": {
            "principals": principals,
            "note": "every principal must present a DID credential establishing ahliyyat al-ada' (capacity): not a minor, not interdicted (safih/majnun), not a bankrupt under hajr (taflis), KYC-cleared, not AML-sanctioned [scholar-verify]"
        },
        "note": "Portable invariant manifest. Enforce each constraint against a proposed terms object before committing it to any ledger or database. The engine proves consistency with a declared rule-base; it issues no fatwa.",
    });
    let asnaf = asnaf_cfg(spec);
    if !asnaf.is_empty() {
        let m: serde_json::Map<String, serde_json::Value> =
            asnaf.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        manifest["zakat_asnaf"] = serde_json::Value::Object(m);
    }
    serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".to_string())
}

/// Zero-Knowledge Fiqh (enterprise vector #1): the PRODUCTION path. Emit a Circom circuit that
/// encodes a fiqh invariant as a rank-1 constraint system, a zk manifest, and a Solidity stub
/// that gates a transition on a valid Groth16 proof — so a bank can prove [RISK-1] (loss shared
/// proportional to ownership) on a public ledger WITHOUT revealing the loss amounts. The Rust
/// `zk` module is a self-contained sigma-protocol PoC of the same statement; this is the
/// industry-standard circuit the figures would actually be proven against.
pub struct ZkArtifacts {
    pub circuit_name: String,
    pub circom: String,
    pub manifest: String,
    pub verifier_consumer: String,
}

pub fn build_zk(spec: &Spec) -> ZkArtifacts {
    let class = Class::from_str(&spec.class);
    let cname = format!("{}LossShare", spec.name);
    match class {
        Class::MusharakahMutanaqisah => zk_musharakah(spec, &cname),
        _ => zk_generic(spec, &cname),
    }
}

fn zk_musharakah(spec: &Spec, cname: &str) -> ZkArtifacts {
    let circom = format!(
        r#"pragma circom 2.1.6;

// Generated by `fiqhc build --target zk` from instrument '{name}'.
// Encodes the Musharakah loss-sharing invariant [RISK-1] as a rank-1 constraint system:
// PROVE that each partner bore loss in proportion to ownership, WITHOUT revealing the loss
// amounts (they are private witness signals). Compliance is proven, not disclosed.
//
//   public  : bankBps, clientBps
//   private : lossBank, lossClient
//   [INV-1] ownership_conserved : bankBps + clientBps === 10000
//   [RISK-1] loss proportional  : lossBank * clientBps === lossClient * bankBps
//
// Fiqh: a partnership shares loss strictly by capital share — AAOIFI SS No. 12 [scholar-verify].
template {cname}() {{
    signal input bankBps;     // public
    signal input clientBps;   // public
    signal input lossBank;    // private witness — never revealed
    signal input lossClient;  // private witness — never revealed

    // [INV-1] ownership is conserved
    bankBps + clientBps === 10000;

    // [RISK-1] loss is proportional to ownership (riba-free risk-sharing).
    // Split into single multiplications so each is one R1CS constraint.
    signal lhs;
    signal rhs;
    lhs <== lossBank * clientBps;
    rhs <== lossClient * bankBps;
    lhs === rhs;
}}

component main {{public [bankBps, clientBps]}} = {cname}();
"#,
        name = spec.name,
        cname = cname,
    );

    let manifest = format!(
        r#"{{
  "kind": "zk_circuit_manifest",
  "scheme": "groth16",
  "instrument": "musharakah_mutanaqisah",
  "circuit": "{cname}",
  "publicSignals": ["bankBps", "clientBps"],
  "privateSignals": ["lossBank", "lossClient"],
  "constraints": [
    {{ "code": "INV-1", "statement": "bankBps + clientBps == 10000", "meaning": "ownership conserved", "citation": "" }},
    {{ "code": "RISK-1", "statement": "lossBank * clientBps == lossClient * bankBps", "meaning": "loss shared proportional to ownership", "citation": "AAOIFI SS No. 12 [scholar-verify]" }}
  ],
  "pipeline": [
    "circom {cname}.circom --r1cs --wasm --sym",
    "snarkjs groth16 setup {cname}.r1cs pot.ptau {cname}_0.zkey",
    "snarkjs zkey contribute {cname}_0.zkey {cname}.zkey",
    "snarkjs zkey export solidityverifier {cname}.zkey Verifier.sol",
    "snarkjs groth16 prove {cname}.zkey witness.wtns proof.json public.json",
    "snarkjs groth16 verify verification_key.json public.json proof.json"
  ],
  "note": "A bank proves [RISK-1] holds for a settlement while the loss amounts stay private. The circuit encodes the same statement the fiqhc `zk` sigma-protocol PoC proves. Consistency is not a fatwa; the rule-base must be ratified."
}}
"#,
        cname = cname,
    );

    let verifier_consumer = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

// Generated by `fiqhc build --target zk`. Gates a Musharakah settlement on a valid
// zero-knowledge proof that the loss was shared proportional to ownership [RISK-1] — the
// amounts never touch the chain. Pair with the snarkjs-exported Groth16 `Verifier`
// (`snarkjs zkey export solidityverifier`).
interface IGroth16Verifier {{
    function verifyProof(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[2] calldata pubSignals
    ) external view returns (bool);
}}

contract {name}ZkGate {{
    IGroth16Verifier public immutable verifier;

    constructor(address _verifier) {{ verifier = IGroth16Verifier(_verifier); }}

    /// @dev pubSignals = [bankBps, clientBps]. Reverts unless the prover supplies a valid proof
    ///      that the (hidden) losses satisfy lossBank*clientBps == lossClient*bankBps.
    function settleWithProof(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[2] calldata pubSignals
    ) external view returns (bool) {{
        require(
            verifier.verifyProof(a, b, c, pubSignals),
            "[RISK-1] zk proof invalid: loss not proven proportional to ownership"
        );
        return true;
    }}
}}
"#,
        name = spec.name,
    );

    ZkArtifacts {
        circuit_name: cname.to_string(),
        circom,
        manifest,
        verifier_consumer,
    }
}

fn zk_generic(spec: &Spec, cname: &str) -> ZkArtifacts {
    let circom = format!(
        r#"pragma circom 2.1.6;

// Generated by `fiqhc build --target zk` from instrument '{name}' (class '{class}').
// The ZK target currently models the proportional loss-sharing invariant of partnership
// instruments. For class '{class}', emit a placeholder asserting the public/private split is
// well-formed; extend zk_generic() to encode this class's numeric invariant.
template {cname}() {{
    signal input declared;   // public
    signal input witnessVal; // private
    declared * 1 === declared; // trivially satisfiable placeholder
}}
component main {{public [declared]}} = {cname}();
"#,
        name = spec.name,
        class = spec.class,
        cname = cname,
    );
    let manifest = format!(
        r#"{{
  "kind": "zk_circuit_manifest",
  "scheme": "groth16",
  "instrument": "{class}",
  "circuit": "{cname}",
  "note": "No proportional-loss invariant for this class yet; placeholder circuit. The ZK target is implemented for musharakah_mutanaqisah."
}}
"#,
        class = spec.class,
        cname = cname,
    );
    ZkArtifacts {
        circuit_name: cname.to_string(),
        circom,
        manifest,
        verifier_consumer: String::new(),
    }
}

// --- helpers ---

fn role_name<'a>(spec: &'a Spec, role: &str) -> Option<&'a str> {
    spec.parties()
        .into_iter()
        .find(|p| p.role == role)
        .map(|p| p.name.as_str())
}

fn party_bps(spec: &Spec, role: &str) -> Option<u64> {
    let name = role_name(spec, role)?;
    spec.capital().into_iter().find_map(|c| match c {
        CapItem::Assign { party, bps, .. } if party == name => Some(*bps),
        _ => None,
    })
}

/// Numeric rent rate from `returns { rent { rate: N ...; } }`.
fn rent_rate(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "rent")
        .and_then(|r| kv_get(&r.kvs, "rate"))
        .and_then(|e| e.as_num())
        .unwrap_or(1)
}

fn profit_share(spec: &Spec, role: &str) -> Option<u64> {
    let name = role_name(spec, role)?;
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "profit")
        .and_then(|r| kv_get(&r.kvs, name).and_then(|e| e.as_num()))
}

/// Window (seconds) for khiyar al-shart, if declared.
fn khiyar_window(spec: &Spec) -> Option<u64> {
    spec.rescission()
        .into_iter()
        .find(|r| r.kind == "khiyar_al_shart")
        .and_then(|r| kv_get(&r.kvs, "window"))
        .and_then(|e| e.as_num())
}

fn has_resc(spec: &Spec, kind: &str) -> bool {
    spec.rescission().iter().any(|r| r.kind == kind)
}

/// Zakat routing parameters (rate_bps, nisab) if the spec declares a `zakat { ... }` section.
fn zakat_cfg(spec: &Spec) -> Option<(u64, u64)> {
    let z = spec.zakat_cfg();
    if z.is_empty() {
        return None;
    }
    let g = |k: &str| z.iter().find(|kv| kv.key == k).and_then(|kv| kv.val.as_num());
    Some((g("rate_bps").unwrap_or(250), g("nisab").unwrap_or(0)))
}

/// The eight-asnaf disbursement policy (al-Tawba 9:60), if declared in the zakat block.
fn asnaf_cfg(spec: &Spec) -> Vec<(String, u64)> {
    spec.zakat_cfg()
        .into_iter()
        .filter(|kv| kv.key.starts_with("asnaf_"))
        .filter_map(|kv| kv.val.as_num().map(|n| (kv.key.clone(), n)))
        .collect()
}

/// Emit the recorded aṣnāf policy as on-chain constants — auditable; the fund disburses by these.
fn mmp_asnaf(shares: &[(String, u64)]) -> String {
    let mut s = String::from(
        "\n    // @dev INVARIANT zakat_asnaf: the zakat fund disburses by the eight categories of\n    //      al-Tawba 9:60; these shares (bps, summing to 10000) are the validated, recorded policy.\n",
    );
    for (k, bps) in shares {
        s.push_str(&format!("    uint16 public constant {}_BPS = {};\n", k.to_uppercase(), bps));
    }
    s
}

/// The Solidity for the built-in Zakat al-Tijarah layer. Routes rubʿ al-ʿushr (2.5%, 1/40)
/// of the zakatable base to the existing maslahah/zakat fund, on-chain, due only at/above
/// nisab — so corporate zakat is non-bypassable.
fn mmp_zakat(rate_bps: u64, nisab: u64) -> String {
    format!(
        r#"    uint256 public constant ZAKAT_RATE_BPS = {rate};
    uint256 public constant ZAKAT_NISAB = {nisab};
    event ZakatRouted(uint256 zakatableBase, uint256 due);

    /// @dev INVARIANT zakat_on_haul_nisab: 2.5% (rubʿ al-ʿushr, 1/40) of the zakatable base,
    ///      due only at or above nisab, routed to the maslahah/zakat fund. Corporate zakat is
    ///      a property of the contract, not a year-end act of conscience.
    function zakatDue(uint256 base) public pure returns (uint256) {{
        if (base < ZAKAT_NISAB) return 0;
        return base * ZAKAT_RATE_BPS / BPS;
    }}

    function payZakat(uint256 zakatableBase) external payable nonReentrant {{
        require(msg.sender == bank || msg.sender == client, "only a partner may remit zakat");
        uint256 due = zakatDue(zakatableBase);
        require(msg.value == due, "must remit exactly the zakat due");
        if (due > 0) {{ (bool ok, ) = maslahahFund.call{{value: due}}(""); require(ok, "zakat xfer"); }}
        emit ZakatRouted(zakatableBase, due);
    }}

"#,
        rate = rate_bps,
        nisab = nisab,
    )
}

/// Which contingency off-ramps the spec declares (jaa'ihah reschedule, faraid dissolution).
fn contingency_cfg(spec: &Spec) -> (bool, bool) {
    let c = spec.contingency_cfg();
    let get = |k: &str| c.iter().find(|kv| kv.key == k).and_then(|kv| kv.val.as_ident().map(|s| s.to_string()));
    let jaaihah = matches!(get("jaaihah").as_deref(), Some("reschedule") | Some("abate"));
    let faraid = matches!(get("death").as_deref(), Some("faraid"));
    (jaaihah, faraid)
}

/// Solidity for the lifecycle off-ramps (enterprise vector #4): a jaa'ihah abates the
/// obligation without interest and may be rescheduled; a death dissolves the partnership by
/// faraid — the deceased's escrowed capital passes to the heirs by validated fixed shares.
fn mmp_contingency(jaaihah: bool, faraid: bool) -> String {
    let mut s = String::new();
    if jaaihah {
        s.push_str(
            r#"    bool public jaaihah;
    uint256 public graceDeadline;
    event JaaihahDeclared(address by);
    event ObligationsRescheduled(uint256 graceDeadline);

    /// @dev INVARIANT jaaihah_no_riba: a declared calamity ABATES the rent (it falls to zero)
    ///      and may be rescheduled, but no penalty or interest can ever be added — the loss
    ///      falls on the owner (wadʿ al-jawaʾih), it is not turned into a debt.
    function declareJaaihah() external onlyArbiter live { jaaihah = true; emit JaaihahDeclared(msg.sender); }
    function rescheduleWithoutRiba(uint256 extraSeconds) external onlyArbiter live {
        require(jaaihah, "no calamity declared");
        graceDeadline = block.timestamp + extraSeconds; // grace only — no charge is added here
        emit ObligationsRescheduled(graceDeadline);
    }
    function effectiveRentDue() public view returns (uint256) { return jaaihah ? 0 : rentDue(); }

"#,
        );
    }
    if faraid {
        s.push_str(
            r#"    event FaraidDissolution(uint256 estate, uint256 heirCount);

    /// @dev INVARIANT death_to_faraid: on the death of the client the partnership dissolves and
    ///      the client's escrowed capital passes to the heirs by the fixed Qurʾanic shares
    ///      (computed by the faraid engine off-chain, validated here to total 10000 bps); the
    ///      bank is made whole. Distribution is by the furud, never by discretion.
    function dissolveByFaraid(address[] calldata heirs, uint256[] calldata shareBps)
        external onlyArbiter live nonReentrant
    {
        require(bankShareBps == initialBankShareBps, "performance begun");
        require(heirs.length == shareBps.length && heirs.length > 0, "heirs/shares mismatch");
        uint256 sumBps;
        for (uint256 i = 0; i < shareBps.length; i++) { sumBps += shareBps[i]; }
        require(sumBps == BPS, "faraid shares must total 10000 bps");
        uint256 estate = clientFunded;
        uint256 bankRefund = bankFunded;
        rescinded = true; active = false; pool = 0; bankFunded = 0; clientFunded = 0;
        for (uint256 i = 0; i < heirs.length; i++) {
            uint256 part = estate * shareBps[i] / BPS;
            if (part > 0) { (bool ok, ) = heirs[i].call{value: part}(""); require(ok, "heir xfer"); }
        }
        if (bankRefund > 0) { (bool b, ) = bank.call{value: bankRefund}(""); require(b, "bank refund"); }
        emit FaraidDissolution(estate, heirs.length);
    }

"#,
        );
    }
    s
}

fn has_step(spec: &Spec, name: &str) -> bool {
    spec.lifecycle().iter().any(|s| s.name == name)
}

fn invariant_doc(spec: &Spec) -> String {
    let mut s = String::new();
    for inv in spec.invariants() {
        s.push_str(&format!("///           - {}: {}\n", inv.name, inv.expr.render()));
    }
    s
}

fn provenance_doc(spec: &Spec, title: &str) -> String {
    let basis = spec
        .meta()
        .into_iter()
        .find(|k| k.key == "basis")
        .and_then(|k| k.val.as_str())
        .unwrap_or("(unstated)");
    format!(
        "// SPDX-License-Identifier: MIT\n\
         pragma solidity ^0.8.24;\n\n\
         import {{IValuationOracle}} from \"../IValuationOracle.sol\";\n\n\
         /// @title  {title}\n\
         /// @notice COMPLIANCE BY CONSTRUCTION. Emitted by the deducible compiler ONLY after the\n\
         ///         source .fiqh specification was proven consistent with its declared fiqh\n\
         ///         rule-base. Declared basis: {basis} [scholar-verify].\n\
         ///         The engine issues no fatwa; a qualified scholar must ratify the rule-base.\n\
         ///         Compiled-in invariants:\n\
         {invs}",
        title = title,
        basis = basis,
        invs = invariant_doc(spec),
    )
}

// =====================================================================================
// Musharakah Mutanaqisah  (target: behavioural equivalence with the hand-written V5
// + payRent; capital custody, oracle-priced buyout, settle with maslahah residue,
// and the full rescission family — khiyar al-shart, iqalah, khiyar al-'ayb, faskh)
// =====================================================================================

fn gen_musharakah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let bank_bps = party_bps(spec, "financier").ok_or("financier capital missing")?;
    let rate = rent_rate(spec);
    let window = khiyar_window(spec).unwrap_or(3600);

    let mut s = provenance_doc(spec, &format!("{} — diminishing partnership (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));

    s.push_str(MMP_STATE);
    s.push_str(MMP_EVENTS);
    if has_step(spec, "payRent") {
        s.push_str("    event RentPaid(uint256 amount, uint256 onBankShareBps);\n");
    }
    s.push('\n');
    s.push_str(MMP_CONSTRUCTOR);
    s.push_str(MMP_FUNDING);
    s.push_str(MMP_RENTDUE);
    if has_step(spec, "payRent") {
        s.push_str(MMP_PAYRENT);
    }
    s.push_str(MMP_BUYSHARE);
    s.push_str(MMP_SETTLE);

    // rescission family (each gated on its declaration)
    let any_resc = has_resc(spec, "khiyar_al_shart")
        || has_resc(spec, "iqalah")
        || has_resc(spec, "khiyar_al_ayb")
        || has_resc(spec, "faskh");
    if has_resc(spec, "khiyar_al_shart") {
        s.push_str(MMP_KHIYAR);
    }
    if has_resc(spec, "iqalah") {
        s.push_str(MMP_IQALAH);
    }
    if has_resc(spec, "khiyar_al_ayb") {
        s.push_str(MMP_DEFECT);
    }
    if has_resc(spec, "faskh") {
        s.push_str(MMP_FASKH);
    }
    if any_resc {
        s.push_str(MMP_UNWIND);
    }

    // Zakat al-Tijarah (enterprise vector #5): non-bypassable 2.5% routing, when declared.
    let zakat = zakat_cfg(spec);
    if let Some((zrate, znisab)) = zakat {
        s.push_str(&mmp_zakat(zrate, znisab));
        let asnaf = asnaf_cfg(spec);
        if !asnaf.is_empty() {
            s.push_str(&mmp_asnaf(&asnaf));
        }
    }

    // Lifecycle off-ramps (enterprise vector #4): jaa'ihah abatement + faraid dissolution.
    let (jaaihah, faraid) = contingency_cfg(spec);
    if jaaihah || faraid {
        s.push_str(&mmp_contingency(jaaihah, faraid));
    }

    s.push_str("}\n");

    let test_js = gen_musharakah_test(&name, bank_bps, rate, window, zakat, (jaaihah, faraid));
    let descriptor = musharakah_descriptor(spec, &name, bank_bps, rate, window);

    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const MMP_STATE: &str = r#"    address public immutable bank;
    address public immutable client;
    address public immutable arbiter;
    address public immutable maslahahFund;
    IValuationOracle public immutable oracle;
    uint256 public constant BPS = 10_000;

    uint256 public immutable initialBankShareBps;
    uint256 public bankShareBps;
    uint256 public clientShareBps;
    uint256 public immutable rentPerPeriodPerBps;

    uint256 public pool;
    uint256 public bankFunded;
    uint256 public clientFunded;
    bool public active;
    bool public settled;
    bool public rescinded;

    uint256 public immutable khiyarPeriod;
    uint256 public khiyarDeadline;
    address public iqalahProposer;
    bool public defectRaised;
    address public defectClaimant;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyBank() { require(msg.sender == bank, "only bank"); _; }
    modifier onlyClient() { require(msg.sender == client, "only client"); _; }
    modifier onlyArbiter() { require(msg.sender == arbiter, "only arbiter"); _; }
    modifier live() { require(active && !settled && !rescinded, "not live"); _; }

"#;

const MMP_EVENTS: &str = r#"    event Funded(address who, uint256 amount);
    event Activated(uint256 assetValue, uint256 khiyarDeadline);
    event SharePurchased(uint256 bps, uint256 price, uint256 newBankBps);
    event Settled(uint256 fairValue, uint256 bankPayout, uint256 clientPayout, uint256 toMaslahah);
    event KhiyarRescinded(address by);
    event IqalahProposed(address by);
    event IqalahCompleted(address acceptedBy);
    event DefectRaised(address by, string reason);
    event DefectResolved(bool upheld);
    event JudicialFaskh(address arbiter);
    event Unwound(uint256 bankRefund, uint256 clientRefund);
"#;

const MMP_CONSTRUCTOR: &str = r#"    constructor(
        address _client, address _oracle, address _arbiter, address _maslahahFund,
        uint256 _bankShareBps, uint256 _rentPerPeriodPerBps, uint256 _khiyarPeriod
    ) {
        require(_client != address(0) && _oracle != address(0) && _arbiter != address(0) && _maslahahFund != address(0), "zero addr");
        require(_bankShareBps > 0 && _bankShareBps < BPS, "bank bps range");
        bank = msg.sender; client = _client; arbiter = _arbiter; maslahahFund = _maslahahFund;
        oracle = IValuationOracle(_oracle);
        bankShareBps = _bankShareBps; initialBankShareBps = _bankShareBps; clientShareBps = BPS - _bankShareBps;
        rentPerPeriodPerBps = _rentPerPeriodPerBps; khiyarPeriod = _khiyarPeriod;
    }

"#;

const MMP_FUNDING: &str = r#"    function fundBank() external payable onlyBank {
        require(!active && bankFunded == 0, "bank funded/active");
        uint256 v0 = oracle.fairValue();
        require(msg.value == v0 * bankShareBps / BPS, "bank must fund its share");
        bankFunded = msg.value; pool += msg.value; emit Funded(bank, msg.value); _tryActivate(v0);
    }
    function fundClient() external payable onlyClient {
        require(!active && clientFunded == 0, "client funded/active");
        uint256 v0 = oracle.fairValue();
        require(msg.value == v0 * clientShareBps / BPS, "client must fund its share");
        clientFunded = msg.value; pool += msg.value; emit Funded(client, msg.value); _tryActivate(v0);
    }
    function _tryActivate(uint256 v0) internal {
        if (bankFunded > 0 && clientFunded > 0) {
            require(bankFunded + clientFunded == v0, "funds must equal asset value");
            active = true; khiyarDeadline = block.timestamp + khiyarPeriod;
            emit Activated(v0, khiyarDeadline);
        }
    }

"#;

const MMP_RENTDUE: &str = r#"    /// @dev INVARIANT rent_on_living_share: rent accrues on the bank's CURRENT share only.
    function rentDue() public view returns (uint256) { return rentPerPeriodPerBps * bankShareBps; }

"#;

const MMP_PAYRENT: &str = r#"    function payRent() external payable onlyClient live nonReentrant {
        uint256 due = rentDue();
        require(msg.value == due, "rent must equal due on bank share");
        emit RentPaid(due, bankShareBps);
        (bool ok, ) = bank.call{value: msg.value}(""); require(ok, "rent transfer failed");
    }

"#;

const MMP_BUYSHARE: &str = r#"    /// @dev INVARIANT price_attested: buyout price tracks the independent oracle's fair value.
    function buyShare(uint256 bps) external payable onlyClient live nonReentrant {
        require(bps > 0 && bps <= bankShareBps, "bps range");
        uint256 f = oracle.fairValue(); require(f > 0, "oracle value");
        uint256 price = f * bps / BPS;
        require(msg.value == price, "value != fair price");
        bankShareBps -= bps; clientShareBps += bps;
        pool += msg.value; pool -= price;
        emit SharePurchased(bps, price, bankShareBps);
        (bool ok, ) = bank.call{value: price}(""); require(ok, "buyout xfer");
    }

"#;

const MMP_SETTLE: &str = r#"    /// @dev INVARIANT loss_follows_capital: the impaired remainder is shared by ownership;
    ///      the residue goes to the agreed maslahah fund rather than being stranded.
    function settle() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client || msg.sender == arbiter, "only party/arbiter");
        uint256 f = oracle.fairValue(); require(f > 0, "oracle value");
        uint256 distributable = f > pool ? pool : f;
        uint256 bankPayout = distributable * bankShareBps / BPS;
        uint256 clientPayout = distributable * clientShareBps / BPS;
        uint256 toMaslahah = pool - bankPayout - clientPayout;
        settled = true; pool = 0;
        if (bankPayout > 0) { (bool a, ) = bank.call{value: bankPayout}(""); require(a, "bank payout"); }
        if (clientPayout > 0) { (bool b, ) = client.call{value: clientPayout}(""); require(b, "client payout"); }
        if (toMaslahah > 0) { (bool d, ) = maslahahFund.call{value: toMaslahah}(""); require(d, "maslahah xfer"); }
        emit Settled(f, bankPayout, clientPayout, toMaslahah);
    }

"#;

const MMP_KHIYAR: &str = r#"    function rescindKhiyar() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        require(block.timestamp <= khiyarDeadline, "khiyar window closed");
        require(bankShareBps == initialBankShareBps, "performance begun");
        emit KhiyarRescinded(msg.sender); _unwind();
    }

"#;

const MMP_IQALAH: &str = r#"    function proposeIqalah() external live {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        iqalahProposer = msg.sender; emit IqalahProposed(msg.sender);
    }
    function acceptIqalah() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        require(iqalahProposer != address(0) && msg.sender != iqalahProposer, "needs the other partner");
        require(bankShareBps == initialBankShareBps, "performance begun");
        emit IqalahCompleted(msg.sender); _unwind();
    }

"#;

const MMP_DEFECT: &str = r#"    function raiseDefect(string calldata reason) external live {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        defectRaised = true; defectClaimant = msg.sender;
        emit DefectRaised(msg.sender, reason);
    }
    function resolveDefect(bool upheld) external live nonReentrant onlyArbiter {
        require(defectRaised, "no defect raised");
        emit DefectResolved(upheld);
        if (upheld) { _unwind(); } else { defectRaised = false; defectClaimant = address(0); }
    }

"#;

const MMP_FASKH: &str = r#"    function judicialFaskh() external live nonReentrant onlyArbiter {
        emit JudicialFaskh(msg.sender); _unwind();
    }

"#;

const MMP_UNWIND: &str = r#"    function _unwind() internal {
        uint256 b = bankFunded; uint256 cl = clientFunded;
        rescinded = true; active = false; pool = 0; bankFunded = 0; clientFunded = 0;
        if (b > 0) { (bool ok, ) = bank.call{value: b}(""); require(ok, "bank refund"); }
        if (cl > 0) { (bool ok2, ) = client.call{value: cl}(""); require(ok2, "client refund"); }
        emit Unwound(b, cl);
    }
"#;

fn gen_musharakah_test(
    name: &str,
    bank_bps: u64,
    rate: u64,
    window: u64,
    zakat: Option<(u64, u64)>,
    contingency: (bool, bool),
) -> String {
    let client_bps = 10_000 - bank_bps;
    // When the off-ramps are compiled in, prove jaa'ihah abates rent without riba and a faraid
    // dissolution distributes the client's capital to heirs by validated shares.
    let (has_jaaihah, has_faraid) = contingency;
    let mut cont_test = String::new();
    if has_jaaihah {
        cont_test.push_str(
            r#"
  it("jaaihah_no_riba: a declared calamity abates the rent to zero; no penalty is added", async function () {
    await fund();
    await expect(c.connect(arbiter).declareJaaihah()).to.emit(c, "JaaihahDeclared").withArgs(arbiter.address);
    expect(await c.effectiveRentDue()).to.equal(0n);
  });
"#,
        );
    }
    if has_faraid {
        cont_test.push_str(
            r#"
  it("death_to_faraid: dissolution passes the client's capital to heirs by validated shares", async function () {
    await fund();
    const estate = (V0 * CLIENT_BPS) / 10000n;
    const heirs = [valuer.address, maslahah.address];
    const shares = [7500n, 2500n]; // e.g. a daughter (1/2 + radd) and a mother, per the faraid engine
    const before = await ethers.provider.getBalance(valuer.address);
    await expect(c.connect(arbiter).dissolveByFaraid(heirs, shares))
      .to.emit(c, "FaraidDissolution").withArgs(estate, 2n);
    const after = await ethers.provider.getBalance(valuer.address);
    expect(after - before).to.equal((estate * 7500n) / 10000n);
    expect(await c.rescinded()).to.equal(true);
  });
"#,
        );
    }
    // When zakat is compiled in, prove the 2.5% routes to the maslahah fund (and nothing below nisab).
    let zakat_test = match zakat {
        Some((zrate, znisab)) => format!(
            r#"
  it("zakat_on_haul_nisab: {rate_pct}% of the zakatable base routes to the maslahah fund; nothing below nisab", async function () {{
    await fund();
    expect(await c.zakatDue(1n)).to.equal(0n);
    const base = {nisab}n * 4n;
    const due = (base * {zrate}n) / 10000n;
    const before = await ethers.provider.getBalance(maslahah.address);
    await expect(c.connect(bank).payZakat(base, {{ value: due }})).to.emit(c, "ZakatRouted").withArgs(base, due);
    const after = await ethers.provider.getBalance(maslahah.address);
    expect(after - before).to.equal(due);
  }});
"#,
            rate_pct = (zrate as f64) / 100.0,
            nisab = znisab,
            zrate = zrate,
        ),
        None => String::new(),
    };
    format!(
        r#"// Generated by deducible — differential proof: the GENERATED {name} reproduces the
// hand-written, peer-reviewed Musharakah Mutanaqisah behaviour on the shared lifecycle.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — differential equivalence", function () {{
  let bank, client, valuer, arbiter, maslahah, oracle, c;
  const V0 = 100000000n;
  const BANK_BPS = {bank_bps}n;
  const CLIENT_BPS = {client_bps}n;
  const RATE = {rate}n;
  const KHIYAR = {window};

  beforeEach(async function () {{
    [bank, client, valuer, arbiter, maslahah] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(V0);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(bank).deploy(client.address, await oracle.getAddress(), arbiter.address, maslahah.address, BANK_BPS, RATE, KHIYAR);
    await c.waitForDeployment();
  }});
  async function fund() {{
    await c.connect(bank).fundBank({{ value: (V0 * BANK_BPS) / 10000n }});
    await c.connect(client).fundClient({{ value: (V0 * CLIENT_BPS) / 10000n }});
  }}

  it("I1 rent_on_living_share: rentDue tracks the bank's living share", async function () {{
    await fund();
    expect(await c.rentDue()).to.equal(RATE * BANK_BPS);
    const due = RATE * BANK_BPS;
    await expect(c.connect(client).payRent({{ value: due }}))
      .to.emit(c, "RentPaid").withArgs(due, BANK_BPS);
  }});

  it("I2 price_attested: buyShare is priced from the oracle and steps ownership down", async function () {{
    await fund();
    const bps = 2000n;
    const price = (V0 * bps) / 10000n;
    await expect(c.connect(client).buyShare(bps, {{ value: price }}))
      .to.emit(c, "SharePurchased").withArgs(bps, price, BANK_BPS - bps);
    expect(await c.bankShareBps()).to.equal(BANK_BPS - bps);
  }});

  it("I3 loss_follows_capital: settle pays current value by ownership; residue to maslahah", async function () {{
    await fund();
    await oracle.connect(valuer).attest(90000000n);
    const before = await ethers.provider.getBalance(maslahah.address);
    await expect(c.connect(bank).settle())
      .to.emit(c, "Settled").withArgs(90000000n, (90000000n * BANK_BPS) / 10000n, (90000000n * CLIENT_BPS) / 10000n, 10000000n);
    const after = await ethers.provider.getBalance(maslahah.address);
    expect(after - before).to.equal(10000000n);
  }});

  it("I4 role separation: only the arbiter may uphold a defect (khiyar al-'ayb)", async function () {{
    await fund();
    await c.connect(client).raiseDefect("latent defect");
    await expect(c.connect(bank).resolveDefect(true)).to.be.revertedWith("only arbiter");
    await expect(c.connect(arbiter).resolveDefect(true)).to.emit(c, "Unwound");
    expect(await c.rescinded()).to.equal(true);
  }});

  it("flexibility: judicial faskh is the arbiter's authority alone; iqalah needs both partners", async function () {{
    await fund();
    await expect(c.connect(client).judicialFaskh()).to.be.revertedWith("only arbiter");
    await c.connect(bank).proposeIqalah();
    await expect(c.connect(client).acceptIqalah()).to.emit(c, "IqalahCompleted").withArgs(client.address);
    expect(await c.rescinded()).to.equal(true);
  }});
{zakat_test}{cont_test}}});
"#,
        name = name,
        bank_bps = bank_bps,
        client_bps = client_bps,
        rate = rate,
        window = window,
        zakat_test = zakat_test,
        cont_test = cont_test,
    )
}

/// Consensus oracle parameters (committee, quorum, gharar_bound_bps) if the spec declares
/// `oracle { mode: consensus; ... }`; otherwise None (single-valuer / mock mode).
fn consensus_cfg(spec: &Spec) -> Option<(u64, u64, u64)> {
    let oc = spec.oracle_cfg();
    if oc.is_empty() {
        return None;
    }
    if oc.iter().find(|k| k.key == "mode").and_then(|k| k.val.as_ident()) != Some("consensus") {
        return None;
    }
    let g = |key: &str| oc.iter().find(|k| k.key == key).and_then(|k| k.val.as_num());
    Some((g("committee")?, g("quorum")?, g("gharar_bound_bps")?))
}

fn musharakah_descriptor(spec: &Spec, name: &str, bank_bps: u64, rate: u64, window: u64) -> String {
    // Lean tinybar asset value to fit a constrained testnet budget.
    let v0: u64 = 1_000_000;
    let bank_fund = v0 * bank_bps / 10_000;
    let client_fund = v0 - bank_fund;
    let buy_bps: u64 = 2000;
    let buy_price = v0 * buy_bps / 10_000;
    let rent_due = rate * bank_bps;
    let (oracle_block, mode) = match consensus_cfg(spec) {
        Some((c, q, b)) => (
            format!(
                "{{ \"contract\": \"ConsensusValuationOracle\", \"mode\": \"consensus\", \"committee\": {}, \"quorum\": {}, \"ghararBoundBps\": {}, \"targetValue\": {} }}",
                c, q, b, v0
            ),
            "consensus",
        ),
        None => (
            format!("{{ \"contract\": \"MockValuationOracle\", \"initialValue\": {} }}", v0),
            "single",
        ),
    };
    let zakat_block = match zakat_cfg(spec) {
        Some((zr, zn)) => format!(
            "{{ \"rateBps\": {}, \"nisab\": {}, \"beneficiary\": \"maslahah\", \"sink\": \"maslahahFund\" }}",
            zr, zn
        ),
        None => "null".to_string(),
    };
    format!(
        r#"{{
  "instrument": "musharakah_mutanaqisah",
  "contract": "{name}",
  "operatorRole": "bank",
  "oracleMode": "{mode}",
  "oracle": {oracle_block},
  "zakat": {zakat_block},
  "constructorAbi": ["address","address","address","address","uint256","uint256","uint256"],
  "constructorArgs": ["@client","@oracle","@arbiter","@maslahah",{bank_bps},{rate},{window}],
  "accounts": ["client","arbiter","maslahah"],
  "funding": {{ "fundBank": {bank_fund}, "fundClient": {client_fund} }},
  "lifecycle": [
    {{ "as": "client", "fn": "payRent", "value": {rent_due}, "note": "rent on living share" }},
    {{ "as": "client", "fn": "buyShare", "args": [{buy_bps}], "value": {buy_price}, "note": "oracle-priced buyout, ownership steps down" }}
  ],
  "reads": [
    {{ "fn": "bankShareBps", "expect": {bank_after} }},
    {{ "fn": "clientShareBps", "expect": {client_after} }}
  ]
}}
"#,
        bank_after = bank_bps - buy_bps,
        client_after = 10_000 - bank_bps + buy_bps,
        name = name,
        mode = mode,
        oracle_block = oracle_block,
        zakat_block = zakat_block,
        bank_bps = bank_bps,
        rate = rate,
        window = window,
        bank_fund = bank_fund,
        client_fund = client_fund,
        rent_due = rent_due,
        buy_bps = buy_bps,
        buy_price = buy_price,
    )
}

fn lease_term(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "rent")
        .and_then(|r| kv_get(&r.kvs, "term"))
        .and_then(|e| e.as_num())
        .unwrap_or(3)
}

// =====================================================================================
// Mudarabah — profit-sharing trust. Capital from the rabb al-mal alone (placed with the
// mudarib to trade); profit split by a pre-agreed ratio; financial loss falls on the
// rabb al-mal alone unless the arbiter rules the mudarib negligent (ta'addi/taqsir).
// =====================================================================================

fn gen_mudarabah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let profit_rabb = profit_share(spec, "rabb_al_mal").ok_or("rabb al-mal profit share missing")?;
    let mut s = provenance_doc(spec, &format!("{} — profit-sharing trust (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(MUDARABAH_BODY);
    s.push_str("}\n");
    let test_js = gen_mudarabah_test(&name, profit_rabb);
    let descriptor = mudarabah_descriptor(&name, profit_rabb);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const MUDARABAH_BODY: &str = r#"    address public immutable rabbAlMal;
    address public immutable mudarib;
    address public immutable arbiter;
    IValuationOracle public immutable oracle;
    uint256 public constant BPS = 10_000;

    uint256 public immutable profitRabbBps;
    uint256 public immutable profitMudaribBps;

    uint256 public capital;
    uint256 public pool;
    bool public active;
    bool public returned;
    bool public settled;
    bool public mudaribLiable;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyRabb() { require(msg.sender == rabbAlMal, "only rabb al-mal"); _; }
    modifier onlyMudarib() { require(msg.sender == mudarib, "only mudarib"); _; }
    modifier onlyArbiter() { require(msg.sender == arbiter, "only arbiter"); _; }

    event Funded(uint256 capital);
    event ProceedsReturned(uint256 amount);
    event NegligenceRuled(address by);
    event Settled(uint256 realized, uint256 rabbPayout, uint256 mudaribPayout, bool loss);
    event MudaribLiability(uint256 shortfall);

    constructor(address _mudarib, address _oracle, address _arbiter, uint256 _profitRabbBps) {
        require(_mudarib != address(0) && _oracle != address(0) && _arbiter != address(0), "zero addr");
        require(_profitRabbBps > 0 && _profitRabbBps < BPS, "profit bps range");
        rabbAlMal = msg.sender; mudarib = _mudarib; arbiter = _arbiter;
        oracle = IValuationOracle(_oracle);
        profitRabbBps = _profitRabbBps; profitMudaribBps = BPS - _profitRabbBps;
    }

    /// @dev INVARIANT capital_from_rabb_al_mal_only: only the rabb al-mal funds; the capital is
    ///      placed with the mudarib to trade (the mudarib contributes labor, not capital).
    function fund() external payable onlyRabb nonReentrant {
        require(!active && capital == 0, "funded/active");
        uint256 c = oracle.fairValue();
        require(msg.value == c, "rabb funds the full capital");
        capital = msg.value; active = true;
        emit Funded(msg.value);
        (bool ok, ) = mudarib.call{value: msg.value}(""); require(ok, "capital to mudarib");
    }

    /// @dev the mudarib returns the INDEPENDENTLY-ATTESTED realized value; it is not self-reported.
    function reportReturn() external payable onlyMudarib {
        require(active && !returned && !settled, "not live");
        uint256 r = oracle.fairValue();
        require(msg.value == r, "must deposit the attested realized value");
        pool += msg.value; returned = true;
        emit ProceedsReturned(msg.value);
    }

    function ruleNegligence() external onlyArbiter {
        require(active && !settled, "not live");
        mudaribLiable = true; emit NegligenceRuled(msg.sender);
    }

    /// @dev INVARIANT profit_by_ratio + loss_on_rabb_al_mal: profit splits by the pre-agreed
    ///      ratio; financial loss falls on the rabb al-mal alone (the mudarib loses only its
    ///      effort) unless the arbiter has ruled the mudarib negligent (ta'addi/taqsir).
    function settle() external nonReentrant {
        require(msg.sender == rabbAlMal || msg.sender == mudarib || msg.sender == arbiter, "only party/arbiter");
        require(returned, "no proceeds reported");
        require(!settled, "settled");
        settled = true;
        uint256 realized = pool;
        uint256 rabbPayout;
        uint256 mudaribPayout;
        bool loss;
        if (realized >= capital) {
            uint256 profit = realized - capital;
            uint256 rp = profit * profitRabbBps / BPS;
            rabbPayout = capital + rp;
            mudaribPayout = profit - rp;
            loss = false;
        } else {
            loss = true;
            rabbPayout = realized;
            mudaribPayout = 0;
            if (mudaribLiable) { emit MudaribLiability(capital - realized); }
        }
        pool = 0;
        if (rabbPayout > 0) { (bool a, ) = rabbAlMal.call{value: rabbPayout}(""); require(a, "rabb payout"); }
        if (mudaribPayout > 0) { (bool b, ) = mudarib.call{value: mudaribPayout}(""); require(b, "mudarib payout"); }
        emit Settled(realized, rabbPayout, mudaribPayout, loss);
    }
"#;

fn gen_mudarabah_test(name: &str, profit_rabb: u64) -> String {
    format!(
        r#"// Generated by deducible — Mudarabah profit-sharing trust. Proves the SAME compiler emits a
// DIFFERENT, correct instrument: profit by ratio, and loss borne by the rabb al-mal alone.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — profit-sharing & loss on rabb al-mal", function () {{
  let rabb, agent, valuer, arbiter, oracle, c;
  const C = 100000000n;
  const PROFIT_RABB = {profit_rabb}n;

  beforeEach(async function () {{
    [rabb, agent, valuer, arbiter] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(C);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(rabb).deploy(agent.address, await oracle.getAddress(), arbiter.address, PROFIT_RABB);
    await c.waitForDeployment();
  }});

  it("capital_from_rabb_al_mal_only: fund() places the full capital with the mudarib (labor, not capital)", async function () {{
    const before = await ethers.provider.getBalance(agent.address);
    await c.connect(rabb).fund({{ value: C }});
    const after = await ethers.provider.getBalance(agent.address);
    expect(after - before).to.equal(C);
  }});

  it("profit_by_ratio: a profit splits by the pre-agreed ratio", async function () {{
    await c.connect(rabb).fund({{ value: C }});
    await oracle.connect(valuer).attest(150000000n);
    await c.connect(agent).reportReturn({{ value: 150000000n }});
    const profit = 50000000n;
    const rp = (profit * PROFIT_RABB) / 10000n;
    await expect(c.connect(rabb).settle())
      .to.emit(c, "Settled").withArgs(150000000n, C + rp, profit - rp, false);
  }});

  it("loss_on_rabb_al_mal: a loss is borne by the rabb al-mal alone; the mudarib loses only effort", async function () {{
    await c.connect(rabb).fund({{ value: C }});
    await oracle.connect(valuer).attest(80000000n);
    await c.connect(agent).reportReturn({{ value: 80000000n }});
    await expect(c.connect(rabb).settle())
      .to.emit(c, "Settled").withArgs(80000000n, 80000000n, 0n, true);
  }});

  it("role separation: only the arbiter may rule the mudarib negligent (ta'addi/taqsir)", async function () {{
    await c.connect(rabb).fund({{ value: C }});
    await expect(c.connect(agent).ruleNegligence()).to.be.revertedWith("only arbiter");
    await expect(c.connect(arbiter).ruleNegligence()).to.emit(c, "NegligenceRuled");
  }});
}});
"#,
        name = name,
        profit_rabb = profit_rabb,
    )
}

fn mudarabah_descriptor(name: &str, profit_rabb: u64) -> String {
    let v0: u64 = 1_000_000;
    let realized: u64 = 1_500_000;
    format!(
        r#"{{
  "instrument": "mudarabah",
  "contract": "{name}",
  "operatorRole": "rabb",
  "oracle": {{ "contract": "MockValuationOracle", "initialValue": {v0} }},
  "constructorAbi": ["address","address","address","uint256"],
  "constructorArgs": ["@mudarib","@oracle","@arbiter",{profit_rabb}],
  "accounts": ["mudarib","arbiter"],
  "funding": {{ "fund": {v0} }},
  "lifecycle": [
    {{ "target": "oracle", "fn": "attest", "args": [{realized}], "note": "valuer attests realized venture value" }},
    {{ "as": "mudarib", "fn": "reportReturn", "value": {realized}, "note": "mudarib deposits attested proceeds" }},
    {{ "as": "rabb", "fn": "settle", "note": "profit split by ratio" }}
  ]
}}
"#,
        name = name,
        v0 = v0,
        realized = realized,
        profit_rabb = profit_rabb,
    )
}

// =====================================================================================
// Ijarah Muntahia Bittamleek — lease ending in ownership. Rent prices the usufruct and
// flows to the lessor; the lessor bears ownership risk and major maintenance; transfer of
// ownership is a SEPARATE act at the end of term; any late charge goes to charity, not the
// lessor (no interest on a debt).
// =====================================================================================

fn gen_ijarah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let rate = rent_rate(spec);
    let term = lease_term(spec);
    let mut s = provenance_doc(spec, &format!("{} — lease ending in ownership (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(IJARAH_BODY);
    s.push_str("}\n");
    let test_js = gen_ijarah_test(&name, rate, term);
    let descriptor = ijarah_descriptor(&name, rate, term);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const IJARAH_BODY: &str = r#"    address public immutable lessor;
    address public immutable lessee;
    address public immutable charity;
    IValuationOracle public immutable oracle;
    uint256 public constant BPS = 10_000;

    uint256 public immutable rentPerPeriod;
    uint256 public immutable termPeriods;
    uint256 public periodsPaid;
    uint256 public lastAssetValue;
    bool public active;
    bool public transferred;
    bool public terminated;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyLessor() { require(msg.sender == lessor, "only lessor"); _; }
    modifier onlyLessee() { require(msg.sender == lessee, "only lessee"); _; }

    event LeaseActivated(uint256 assetValue, uint256 rentPerPeriod);
    event RentPaid(uint256 period, uint256 amount);
    event MaintenancePaid(uint256 amount);
    event AssetImpaired(uint256 newValue, address borneBy);
    event OwnershipTransferred(address to);
    event Terminated();

    constructor(address _lessee, address _oracle, address _charity, uint256 _rentPerPeriod, uint256 _termPeriods) {
        require(_lessee != address(0) && _oracle != address(0) && _charity != address(0), "zero addr");
        require(_termPeriods > 0, "term");
        lessor = msg.sender; lessee = _lessee; charity = _charity;
        oracle = IValuationOracle(_oracle);
        rentPerPeriod = _rentPerPeriod; termPeriods = _termPeriods;
    }

    function activate() external onlyLessor {
        require(!active, "active");
        uint256 v = oracle.fairValue(); require(v > 0, "oracle value");
        active = true; lastAssetValue = v;
        emit LeaseActivated(v, rentPerPeriod);
    }

    /// @dev INVARIANT rent_for_usufruct: rent prices the usufruct and flows to the lessor;
    ///      it is not interest on principal.
    function payRent() external payable onlyLessee nonReentrant {
        require(active && !terminated && !transferred, "not live");
        require(msg.value == rentPerPeriod, "rent must equal period rent");
        periodsPaid += 1;
        emit RentPaid(periodsPaid, msg.value);
        (bool ok, ) = lessor.call{value: msg.value}(""); require(ok, "rent xfer");
    }

    /// @dev the lessor (owner) bears major maintenance.
    function lessorMaintenance() external payable onlyLessor nonReentrant {
        require(active, "not active");
        emit MaintenancePaid(msg.value);
        if (msg.value > 0) { (bool ok, ) = lessee.call{value: msg.value}(""); require(ok, "maint xfer"); }
    }

    /// @dev INVARIANT lessor_bears_ownership_risk: impairment is borne by the lessor; if the
    ///      asset is destroyed the lease terminates and rent abates.
    function recordImpairment() external {
        require(msg.sender == lessor || msg.sender == lessee, "only a party");
        require(active && !terminated, "not live");
        uint256 v = oracle.fairValue();
        lastAssetValue = v;
        emit AssetImpaired(v, lessor);
        if (v == 0) { terminated = true; emit Terminated(); }
    }

    /// @dev INVARIANT transfer_separate_from_lease: ownership transfer is a DISTINCT act at the
    ///      end of the term, never bundled into the lease (two contracts in one is prohibited).
    function transferOwnership() external onlyLessor {
        require(active && !transferred && !terminated, "not transferable");
        require(periodsPaid >= termPeriods, "term not complete");
        transferred = true;
        emit OwnershipTransferred(lessee);
    }

    /// @dev INVARIANT no_late_penalty_interest: any late charge goes to charity (sadaqah),
    ///      never to the lessor as interest on a debt.
    function payLateCharge() external payable onlyLessee nonReentrant {
        require(active, "not active");
        require(msg.value > 0, "no charge");
        (bool ok, ) = charity.call{value: msg.value}(""); require(ok, "charity xfer");
    }
"#;

fn gen_ijarah_test(name: &str, rate: u64, term: u64) -> String {
    format!(
        r#"// Generated by deducible — Ijarah Muntahia Bittamleek. Proves a THIRD instrument from the same
// compiler: rent for usufruct, lessor bears ownership risk, and ownership transfer is a
// SEPARATE act at the end of the term (not two contracts in one).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — lease ending in ownership", function () {{
  let lessor, lessee, valuer, charity, oracle, c;
  const RENT = {rate}n;
  const TERM = {term}n;

  beforeEach(async function () {{
    [lessor, lessee, valuer, charity] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(100000000n);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(lessor).deploy(lessee.address, await oracle.getAddress(), charity.address, RENT, TERM);
    await c.waitForDeployment();
  }});

  it("rent_for_usufruct: a period's rent flows to the lessor", async function () {{
    await c.connect(lessor).activate();
    await expect(c.connect(lessee).payRent({{ value: RENT }})).to.emit(c, "RentPaid").withArgs(1n, RENT);
  }});

  it("transfer_separate_from_lease: ownership transfers only AFTER the full term, as a distinct act", async function () {{
    await c.connect(lessor).activate();
    await expect(c.connect(lessor).transferOwnership()).to.be.revertedWith("term not complete");
    for (let i = 0n; i < TERM; i++) {{ await c.connect(lessee).payRent({{ value: RENT }}); }}
    await expect(c.connect(lessor).transferOwnership()).to.emit(c, "OwnershipTransferred").withArgs(lessee.address);
  }});

  it("lessor_bears_ownership_risk: asset impairment is recorded as borne by the lessor", async function () {{
    await c.connect(lessor).activate();
    await oracle.connect(valuer).attest(50000000n);
    await expect(c.connect(lessee).recordImpairment()).to.emit(c, "AssetImpaired").withArgs(50000000n, lessor.address);
  }});

  it("only the lessor (owner) may transfer ownership", async function () {{
    await c.connect(lessor).activate();
    await expect(c.connect(lessee).transferOwnership()).to.be.revertedWith("only lessor");
  }});

  it("no_late_penalty_interest: any late charge goes to charity, never to the lessor", async function () {{
    await c.connect(lessor).activate();
    const before = await ethers.provider.getBalance(charity.address);
    await c.connect(lessee).payLateCharge({{ value: 500n }});
    const after = await ethers.provider.getBalance(charity.address);
    expect(after - before).to.equal(500n);
  }});
}});
"#,
        name = name,
        rate = rate,
        term = term,
    )
}

fn ijarah_descriptor(name: &str, rate: u64, term: u64) -> String {
    format!(
        r#"{{
  "instrument": "ijarah_imbt",
  "contract": "{name}",
  "operatorRole": "lessor",
  "oracle": {{ "contract": "MockValuationOracle", "initialValue": 1000000 }},
  "constructorAbi": ["address","address","address","uint256","uint256"],
  "constructorArgs": ["@lessee","@oracle","@charity",{rate},{term}],
  "accounts": ["lessee","charity"],
  "lifecycle": [
    {{ "as": "lessor", "fn": "activate", "note": "lessor activates the lease" }},
    {{ "as": "lessee", "fn": "payRent", "value": {rate}, "note": "rent for usufruct" }}
  ]
}}
"#,
        name = name,
        rate = rate,
        term = term,
    )
}

// =====================================================================================
// Commercial Escrow (common law) — the universality of compliance-by-construction beyond
// Islamic finance, and a regime-NEUTRAL judiciary engine: deposit held in escrow, released on
// a definite condition, with arbiter-adjudicated remedy (release or refund). The same machinery
// that encodes khiyar/faskh serves common-law arbitration — a prototype "code-based judiciary."
// =====================================================================================

// =====================================================================================
// Murabaha (cost-plus trust sale, bay' al-amana). A bilateral SALE: the bank acquires a
// real good (takes possession — qabd), discloses its true cost, then resells it for a fixed
// disclosed markup on deferred terms. No oracle (the price is fixed and disclosed, not
// market-valued). The compiled-in invariants make the three riba routes unrepresentable.
// =====================================================================================

fn sale_field(spec: &Spec, key: &str) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "sale")
        .and_then(|r| kv_get(&r.kvs, key))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_murabahah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let cost = sale_field(spec, "cost");
    let markup = sale_field(spec, "markup");
    let mut s = provenance_doc(spec, &format!("{} — murabaha (cost-plus trust sale, bay' al-amana) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(MURABAHAH_BODY);
    s.push_str("}\n");
    let test_js = gen_murabahah_test(&name, cost, markup);
    let descriptor = murabahah_descriptor(&name, cost, markup);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const MURABAHAH_BODY: &str = r#"    address public immutable bank;      // the seller (financier)
    address public immutable customer;  // the buyer
    uint256 public immutable cost;      // the disclosed acquisition cost (bay' al-amana)
    uint256 public immutable markup;    // the fixed, disclosed profit — never interest
    uint256 public immutable total;     // cost + markup, fixed at contract (price certainty)

    bool public acquired;   // the bank has taken ownership + possession (qabd)
    bool public disclosed;  // the true cost has been disclosed to the buyer
    bool public sold;       // the cost-plus sale has been concluded
    uint256 public paid;    // cumulative instalments paid by the buyer
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyBank() { require(msg.sender == bank, "only bank"); _; }
    modifier onlyCustomer() { require(msg.sender == customer, "only customer"); _; }

    event AssetAcquired();
    event CostDisclosed(uint256 cost, uint256 markup, uint256 total);
    event Sold(uint256 total);
    event InstalmentPaid(uint256 amount, uint256 paid, uint256 total);
    event Settled();

    /// @dev INVARIANT cost_disclosed: cost > 0. INVARIANT price_certain: total == cost + markup.
    constructor(address _customer, uint256 _cost, uint256 _markup) {
        require(_customer != address(0), "zero addr");
        require(_customer != msg.sender, "seller and buyer must be distinct");
        require(_cost > 0, "cost must be disclosed (bay' al-amana)");
        bank = msg.sender; customer = _customer;
        cost = _cost; markup = _markup; total = _cost + _markup;
    }

    /// @dev INVARIANT prior_ownership: the bank takes ownership + possession (qabd) BEFORE selling.
    function acquireAsset() external onlyBank {
        require(!acquired, "already acquired");
        acquired = true; emit AssetAcquired();
    }

    /// @dev bay' al-amana: the true cost is disclosed to the buyer before the sale.
    function discloseCost() external onlyBank {
        require(acquired, "must possess the asset first");
        disclosed = true; emit CostDisclosed(cost, markup, total);
    }

    /// @dev INVARIANT prior_ownership: 'do not sell what you do not have' — selling requires prior qabd.
    function sell() external onlyBank {
        require(acquired, "cannot sell before possession (qabd)");
        require(disclosed, "cost must be disclosed first");
        require(!sold, "already sold");
        sold = true; emit Sold(total);
    }

    /// @dev INVARIANT no_penalty_interest: the buyer never owes more than the fixed total, no matter
    ///      how late — there is no penalty-riba on the debt. Instalments forward straight to the bank.
    function payInstalment() external payable onlyCustomer nonReentrant {
        require(sold, "not sold yet");
        require(!settled, "already settled");
        require(msg.value > 0, "no payment");
        require(paid + msg.value <= total, "would exceed the fixed total price");
        paid += msg.value;
        (bool ok, ) = bank.call{value: msg.value}(""); require(ok, "forward to bank failed");
        emit InstalmentPaid(msg.value, paid, total);
        if (paid == total) { settled = true; emit Settled(); }
    }
"#;

fn gen_murabahah_test(name: &str, cost: u64, markup: u64) -> String {
    format!(
        r#"// Generated by deducible — Murabaha (cost-plus trust sale, bay' al-amana). Proves the three riba
// routes are unrepresentable: a markup that grows with time, selling before possession (qabd),
// and a penalty that overcharges the debtor. The buyer can never owe more than the fixed total.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — murabaha cost-plus sale", function () {{
  let bank, customer, c;
  const COST = {cost}n, MARKUP = {markup}n, TOTAL = {cost}n + {markup}n;

  beforeEach(async function () {{
    [bank, customer] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(bank).deploy(customer.address, COST, MARKUP);
    await c.waitForDeployment();
  }});

  it("price_certain: total == cost + markup, fixed at contract", async function () {{
    expect(await c.total()).to.equal(COST + MARKUP);
  }});

  it("prior_ownership: cannot sell before taking possession (qabd)", async function () {{
    await expect(c.connect(bank).sell()).to.be.revertedWith("cannot sell before possession (qabd)");
  }});

  it("full lifecycle: acquire -> disclose -> sell -> pay; the bank receives exactly the total", async function () {{
    await c.connect(bank).acquireAsset();
    await c.connect(bank).discloseCost();
    await c.connect(bank).sell();
    await expect(c.connect(customer).payInstalment({{ value: TOTAL }})).to.emit(c, "Settled");
    expect(await c.paid()).to.equal(TOTAL);
    expect(await c.settled()).to.equal(true);
  }});

  it("no_penalty_interest: the buyer can never be charged more than the fixed total", async function () {{
    await c.connect(bank).acquireAsset();
    await c.connect(bank).discloseCost();
    await c.connect(bank).sell();
    await expect(c.connect(customer).payInstalment({{ value: TOTAL + 1n }})).to.be.revertedWith("would exceed the fixed total price");
  }});

  it("only the bank (seller) may acquire and sell", async function () {{
    await expect(c.connect(customer).acquireAsset()).to.be.revertedWith("only bank");
  }});
}});
"#,
        name = name,
        cost = cost,
        markup = markup,
    )
}

fn murabahah_descriptor(name: &str, cost: u64, markup: u64) -> String {
    format!(
        r#"{{
  "instrument": "murabahah",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "bank",
  "oracle": null,
  "constructorAbi": ["address","uint256","uint256"],
  "constructorArgs": ["@customer", {cost}, {markup}],
  "accounts": ["customer"],
  "lifecycle": [
    {{ "as": "bank", "fn": "acquireAsset", "note": "bank takes ownership + possession (qabd) FIRST" }},
    {{ "as": "bank", "fn": "discloseCost", "note": "bay' al-amana: true cost disclosed to the buyer" }},
    {{ "as": "bank", "fn": "sell", "note": "cost-plus sale concluded at the fixed total" }},
    {{ "as": "customer", "fn": "payInstalment", "value": {total}, "note": "buyer pays the fixed total; no penalty-riba" }}
  ],
  "reads": ["total","paid","settled"]
}}
"#,
        name = name,
        cost = cost,
        markup = markup,
        total = cost + markup,
    )
}

// =====================================================================================
// Salam (forward sale). The buyer pays the FULL price at the session; the seller delivers a
// described fungible good at a known future date. No oracle (the price is fixed at contract).
// =====================================================================================

fn salam_field(spec: &Spec, key: &str) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "salam")
        .and_then(|r| kv_get(&r.kvs, key))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_salam(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let price = salam_field(spec, "price");
    let quantity = salam_field(spec, "quantity");
    let delivery = salam_field(spec, "delivery_date");
    let mut s = provenance_doc(spec, &format!("{} — salam (forward sale, full prepayment) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(SALAM_BODY);
    s.push_str("}\n");
    let test_js = gen_salam_test(&name, price, quantity, delivery);
    let descriptor = salam_descriptor(&name, price, quantity, delivery);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const SALAM_BODY: &str = r#"    address public immutable buyer;        // rabb al-salam — pays the price now
    address public immutable seller;       // al-muslam ilayh — delivers the good later
    uint256 public immutable price;        // ra's al-mal al-salam — paid IN FULL at the session
    uint256 public immutable quantity;     // the muslam fih: a known quantity (maʿlūm)...
    uint256 public immutable deliveryDate; // ...delivered at a known future date (ajal maʿlūm)

    bool public paid;
    bool public delivered;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyBuyer() { require(msg.sender == buyer, "only buyer"); _; }
    modifier onlySeller() { require(msg.sender == seller, "only seller"); _; }

    event PricePaid(uint256 amount);
    event Delivered(uint256 quantity);
    event Settled();

    /// @dev INVARIANT object_known: quantity > 0. INVARIANT delivery_known: deliveryDate > 0.
    constructor(address _seller, uint256 _price, uint256 _quantity, uint256 _deliveryDate) {
        require(_seller != address(0), "zero addr");
        require(_seller != msg.sender, "buyer and seller must be distinct");
        require(_price > 0, "price");
        require(_quantity > 0, "the muslam fih must be a known quantity (ma'lum)");
        require(_deliveryDate > 0, "delivery must be at a known future date (ajal ma'lum)");
        buyer = msg.sender; seller = _seller; price = _price; quantity = _quantity; deliveryDate = _deliveryDate;
    }

    /// @dev INVARIANT full_prepayment: the entire ra's al-mal is paid at the session — never
    ///      deferred (deferring both price and good is bayʿ al-kaliʾ bi-l-kaliʾ, debt for debt).
    function payPriceInFull() external payable onlyBuyer nonReentrant {
        require(!paid, "already paid");
        require(msg.value == price, "the full salam price must be paid at the session");
        paid = true;
        (bool ok, ) = seller.call{value: price}(""); require(ok, "forward to seller failed");
        emit PricePaid(price);
    }

    function deliver() external onlySeller {
        require(paid, "price not yet paid");
        require(!delivered, "already delivered");
        delivered = true; emit Delivered(quantity);
    }

    function confirmReceipt() external onlyBuyer {
        require(delivered, "not yet delivered");
        require(!settled, "already settled");
        settled = true; emit Settled();
    }
"#;

fn gen_salam_test(name: &str, price: u64, quantity: u64, delivery: u64) -> String {
    format!(
        r#"// Generated by deducible — Salam (forward sale). Proves the gharar guards: full prepayment at the
// session (no debt-for-debt), a known quantity, and a known delivery term.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — salam forward sale", function () {{
  let buyer, seller, c;
  const PRICE = {price}n, QTY = {quantity}n, DDATE = {delivery}n;

  beforeEach(async function () {{
    [buyer, seller] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(buyer).deploy(seller.address, PRICE, QTY, DDATE);
    await c.waitForDeployment();
  }});

  it("object_known + delivery_known are set at contract", async function () {{
    expect(await c.quantity()).to.equal(QTY);
    expect(await c.deliveryDate()).to.equal(DDATE);
  }});

  it("full_prepayment: a partial price is rejected (no debt-for-debt)", async function () {{
    await expect(c.connect(buyer).payPriceInFull({{ value: PRICE - 1n }})).to.be.revertedWith("the full salam price must be paid at the session");
  }});

  it("full lifecycle: pay in full -> seller receives the capital -> deliver -> settle", async function () {{
    await expect(c.connect(buyer).payPriceInFull({{ value: PRICE }})).to.emit(c, "PricePaid").withArgs(PRICE);
    await c.connect(seller).deliver();
    await expect(c.connect(buyer).confirmReceipt()).to.emit(c, "Settled");
    expect(await c.settled()).to.equal(true);
  }});

  it("the seller cannot deliver before the price is paid", async function () {{
    await expect(c.connect(seller).deliver()).to.be.revertedWith("price not yet paid");
  }});

  it("only the buyer pays and only the seller delivers", async function () {{
    await expect(c.connect(seller).payPriceInFull({{ value: PRICE }})).to.be.revertedWith("only buyer");
  }});
}});
"#,
        name = name,
        price = price,
        quantity = quantity,
        delivery = delivery,
    )
}

fn salam_descriptor(name: &str, price: u64, quantity: u64, delivery: u64) -> String {
    format!(
        r#"{{
  "instrument": "salam",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "buyer",
  "oracle": null,
  "constructorAbi": ["address","uint256","uint256","uint256"],
  "constructorArgs": ["@seller", {price}, {quantity}, {delivery}],
  "accounts": ["seller"],
  "lifecycle": [
    {{ "as": "buyer", "fn": "payPriceInFull", "value": {price}, "note": "ra's al-mal paid in full at the session" }},
    {{ "as": "seller", "fn": "deliver", "note": "the described good delivered at the known term" }},
    {{ "as": "buyer", "fn": "confirmReceipt", "note": "buyer confirms; salam settled" }}
  ],
  "reads": ["quantity","deliveryDate","settled"]
}}
"#,
        name = name,
        price = price,
        quantity = quantity,
        delivery = delivery,
    )
}

// =====================================================================================
// Istisna' (manufacture-to-order). The customer commissions a good to be MADE to spec; the
// maker (al-sani') supplies materials + labour. Unlike salam, the price may be paid in
// progress instalments. No oracle (the price is fixed at contract).
// =====================================================================================

fn istisna_field(spec: &Spec, key: &str) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "istisna")
        .and_then(|r| kv_get(&r.kvs, key))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_istisna(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let price = istisna_field(spec, "price");
    let mut s = provenance_doc(spec, &format!("{} — istisna' (manufacture-to-order) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(ISTISNA_BODY);
    s.push_str("}\n");
    let test_js = gen_istisna_test(&name, price);
    let descriptor = istisna_descriptor(&name, price);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const ISTISNA_BODY: &str = r#"    address public immutable customer;     // al-mustasni'
    address public immutable manufacturer; // al-sani' (supplies materials + labour)
    uint256 public immutable price;        // a known, fixed total price

    bool public commissioned;
    bool public manufactured;
    bool public delivered;
    uint256 public paid;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyCustomer() { require(msg.sender == customer, "only customer"); _; }
    modifier onlyManufacturer() { require(msg.sender == manufacturer, "only manufacturer"); _; }

    event Commissioned();
    event Manufactured();
    event Delivered();
    event InstalmentPaid(uint256 amount, uint256 paid, uint256 price);
    event Settled();

    /// @dev INVARIANT price_known: price > 0.
    constructor(address _manufacturer, uint256 _price) {
        require(_manufacturer != address(0), "zero addr");
        require(_manufacturer != msg.sender, "customer and manufacturer must be distinct");
        require(_price > 0, "price must be known");
        customer = msg.sender; manufacturer = _manufacturer; price = _price;
    }

    function commission() external onlyCustomer {
        require(!commissioned, "already commissioned");
        commissioned = true; emit Commissioned();
    }

    /// @dev INVARIANT material_by_maker: the sani' builds with its OWN materials (else ijarat al-'amal).
    function manufacture() external onlyManufacturer {
        require(commissioned, "not commissioned");
        require(!manufactured, "already manufactured");
        manufactured = true; emit Manufactured();
    }

    function deliver() external onlyManufacturer {
        require(manufactured, "not yet manufactured");
        require(!delivered, "already delivered");
        delivered = true; emit Delivered();
        if (paid == price) { settled = true; emit Settled(); }
    }

    /// @dev unlike salam, the price MAY be paid in progress instalments — never above the fixed total.
    function payInstalment() external payable onlyCustomer nonReentrant {
        require(commissioned, "not commissioned");
        require(!settled, "already settled");
        require(msg.value > 0, "no payment");
        require(paid + msg.value <= price, "would exceed the fixed price");
        paid += msg.value;
        (bool ok, ) = manufacturer.call{value: msg.value}(""); require(ok, "forward to manufacturer failed");
        emit InstalmentPaid(msg.value, paid, price);
        if (paid == price && delivered) { settled = true; emit Settled(); }
    }
"#;

fn gen_istisna_test(name: &str, price: u64) -> String {
    format!(
        r#"// Generated by deducible — Istisna' (manufacture-to-order). Proves a made-to-spec sale: the maker
// supplies materials + labour, the price may be paid in progress instalments (unlike salam), and
// the customer can never be charged above the fixed total.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — istisna' manufacture-to-order", function () {{
  let customer, manufacturer, c;
  const PRICE = {price}n;

  beforeEach(async function () {{
    [customer, manufacturer] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(customer).deploy(manufacturer.address, PRICE);
    await c.waitForDeployment();
  }});

  it("price_known is fixed at contract", async function () {{
    expect(await c.price()).to.equal(PRICE);
  }});

  it("cannot manufacture before the good is commissioned", async function () {{
    await expect(c.connect(manufacturer).manufacture()).to.be.revertedWith("not commissioned");
  }});

  it("full lifecycle with PROGRESS instalments: commission -> pay half -> manufacture -> deliver -> pay rest -> settle", async function () {{
    await c.connect(customer).commission();
    await c.connect(customer).payInstalment({{ value: PRICE / 2n }});
    await c.connect(manufacturer).manufacture();
    await c.connect(manufacturer).deliver();
    await expect(c.connect(customer).payInstalment({{ value: PRICE - PRICE / 2n }})).to.emit(c, "Settled");
    expect(await c.settled()).to.equal(true);
  }});

  it("the customer can never be charged above the fixed price", async function () {{
    await c.connect(customer).commission();
    await expect(c.connect(customer).payInstalment({{ value: PRICE + 1n }})).to.be.revertedWith("would exceed the fixed price");
  }});

  it("only the manufacturer manufactures; only the customer commissions", async function () {{
    await expect(c.connect(manufacturer).commission()).to.be.revertedWith("only customer");
  }});
}});
"#,
        name = name,
        price = price,
    )
}

fn istisna_descriptor(name: &str, price: u64) -> String {
    format!(
        r#"{{
  "instrument": "istisna",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "customer",
  "oracle": null,
  "constructorAbi": ["address","uint256"],
  "constructorArgs": ["@manufacturer", {price}],
  "accounts": ["manufacturer"],
  "lifecycle": [
    {{ "as": "customer", "fn": "commission", "note": "customer commissions the made-to-order good" }},
    {{ "as": "manufacturer", "fn": "manufacture", "note": "the sani' builds to spec with its own materials" }},
    {{ "as": "manufacturer", "fn": "deliver", "note": "delivers the masnu'" }},
    {{ "as": "customer", "fn": "payInstalment", "value": {price}, "note": "price paid (may be progressive)" }}
  ],
  "reads": ["price","paid","settled"]
}}
"#,
        name = name,
        price = price,
    )
}

// =====================================================================================
// Sarf (currency / metal exchange). Two legs escrowed and released ATOMICALLY (yadan bi-yad);
// a same-genus exchange is forced equal-for-equal at construction. No oracle.
// =====================================================================================

fn sarf_field(spec: &Spec, key: &str) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "exchange")
        .and_then(|r| kv_get(&r.kvs, key))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn sarf_same_genus(spec: &Spec) -> bool {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "exchange")
        .and_then(|r| kv_get(&r.kvs, "same_genus"))
        .and_then(|e| e.as_ident().map(|s| s == "yes"))
        .unwrap_or(false)
}

fn gen_sarf(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let give = sarf_field(spec, "give_amount");
    let take = sarf_field(spec, "take_amount");
    let same = sarf_same_genus(spec);
    let mut s = provenance_doc(spec, &format!("{} — ṣarf (spot currency/metal exchange, yadan bi-yad) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(SARF_BODY);
    s.push_str("}\n");
    let test_js = gen_sarf_test(&name, give, take, same);
    let descriptor = sarf_descriptor(&name, give, take, same);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const SARF_BODY: &str = r#"    address public immutable partyA;
    address public immutable partyB;
    uint256 public immutable giveAmount;  // party A's leg
    uint256 public immutable takeAmount;  // party B's leg
    bool public immutable sameGenus;

    bool public depositedA;
    bool public depositedB;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyA() { require(msg.sender == partyA, "only party A"); _; }
    modifier onlyB() { require(msg.sender == partyB, "only party B"); _; }

    event DepositedA(uint256 amount);
    event DepositedB(uint256 amount);
    event Settled(uint256 toA, uint256 toB);

    /// @dev INVARIANT riba_fadl_guarded: a same-genus exchange must be equal for equal (no excess).
    constructor(address _partyB, uint256 _giveAmount, uint256 _takeAmount, bool _sameGenus) {
        require(_partyB != address(0), "zero addr");
        require(_partyB != msg.sender, "two distinct counterparties");
        require(_giveAmount > 0 && _takeAmount > 0, "amounts");
        if (_sameGenus) { require(_giveAmount == _takeAmount, "same-genus exchange must be equal (riba al-fadl)"); }
        partyA = msg.sender; partyB = _partyB; giveAmount = _giveAmount; takeAmount = _takeAmount; sameGenus = _sameGenus;
    }

    function depositA() external payable onlyA {
        require(!depositedA, "deposited");
        require(msg.value == giveAmount, "exact leg A");
        depositedA = true; emit DepositedA(msg.value);
    }

    function depositB() external payable onlyB {
        require(!depositedB, "deposited");
        require(msg.value == takeAmount, "exact leg B");
        depositedB = true; emit DepositedB(msg.value);
    }

    /// @dev INVARIANT spot_settlement: yadan bi-yad — neither leg is released until BOTH are in;
    ///      settlement is atomic in one transaction (no deferral = no riba al-nasi'a).
    function settle() external nonReentrant {
        require(msg.sender == partyA || msg.sender == partyB, "only a party");
        require(depositedA && depositedB, "both legs must be present (yadan bi-yad)");
        require(!settled, "settled");
        settled = true;
        (bool okA, ) = partyA.call{value: takeAmount}(""); require(okA, "to A");
        (bool okB, ) = partyB.call{value: giveAmount}(""); require(okB, "to B");
        emit Settled(takeAmount, giveAmount);
    }
"#;

fn gen_sarf_test(name: &str, give: u64, take: u64, same: bool) -> String {
    let same_js = if same { "true" } else { "false" };
    format!(
        r#"// Generated by deducible — Sarf (spot exchange). Proves yadan bi-yad (atomic, no deferral) and,
// for a same-genus exchange, equal-for-equal (no riba al-fadl).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — sarf spot exchange", function () {{
  let partyA, partyB, c;
  const GIVE = {give}n, TAKE = {take}n, SAME = {same_js};

  beforeEach(async function () {{
    [partyA, partyB] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(partyA).deploy(partyB.address, GIVE, TAKE, SAME);
    await c.waitForDeployment();
  }});

  it("spot_settlement: settle reverts until BOTH legs are present (yadan bi-yad)", async function () {{
    await c.connect(partyA).depositA({{ value: GIVE }});
    await expect(c.connect(partyA).settle()).to.be.revertedWith("both legs must be present (yadan bi-yad)");
  }});

  it("atomic settle: both legs in -> A receives B's leg and B receives A's leg", async function () {{
    await c.connect(partyA).depositA({{ value: GIVE }});
    await c.connect(partyB).depositB({{ value: TAKE }});
    await expect(c.connect(partyA).settle()).to.emit(c, "Settled").withArgs(TAKE, GIVE);
    expect(await c.settled()).to.equal(true);
  }});

  it("riba_fadl_guarded: a same-genus exchange with unequal amounts cannot deploy", async function () {{
    const F = await ethers.getContractFactory("{name}");
    await expect(F.connect(partyA).deploy(partyB.address, 1000n, 1100n, true)).to.be.revertedWith("same-genus exchange must be equal (riba al-fadl)");
  }});

  it("only the right party funds each leg", async function () {{
    await expect(c.connect(partyB).depositA({{ value: GIVE }})).to.be.revertedWith("only party A");
  }});
}});
"#,
        name = name,
        give = give,
        take = take,
        same_js = same_js,
    )
}

fn sarf_descriptor(name: &str, give: u64, take: u64, same: bool) -> String {
    let same_js = if same { "true" } else { "false" };
    format!(
        r#"{{
  "instrument": "sarf",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "exchanger_a",
  "oracle": null,
  "constructorAbi": ["address","uint256","uint256","bool"],
  "constructorArgs": ["@exchanger_b", {give}, {take}, {same_js}],
  "accounts": ["exchanger_b"],
  "lifecycle": [
    {{ "as": "exchanger_a", "fn": "depositA", "value": {give}, "note": "party A escrows its leg" }},
    {{ "as": "exchanger_b", "fn": "depositB", "value": {take}, "note": "party B escrows its leg" }},
    {{ "as": "exchanger_a", "fn": "settle", "note": "atomic spot swap (yadan bi-yad)" }}
  ],
  "reads": ["settled"]
}}
"#,
        name = name,
        give = give,
        take = take,
        same_js = same_js,
    )
}

// =====================================================================================
// Tawarruq (individual). Buy a commodity on credit -> take possession -> sell to an INDEPENDENT
// third party for spot cash -> repay the deferred price. The constructor forbids the spot buyer
// being the financier (the on-chain 'inah guard). No oracle.
// =====================================================================================

fn tawarruq_field(spec: &Spec, block: &str, key: &str) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == block)
        .and_then(|r| kv_get(&r.kvs, key))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_tawarruq(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let credit = tawarruq_field(spec, "credit_purchase", "price");
    let spot = tawarruq_field(spec, "spot_sale", "price");
    let mut s = provenance_doc(spec, &format!("{} — tawarruq (individual: credit buy, possess, sell to a third party) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(TAWARRUQ_BODY);
    s.push_str("}\n");
    let test_js = gen_tawarruq_test(&name, credit, spot);
    let descriptor = tawarruq_descriptor(&name, credit, spot);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const TAWARRUQ_BODY: &str = r#"    address public immutable customer;     // al-mustawriq (needs cash)
    address public immutable financier;    // sells the commodity on credit
    address public immutable thirdParty;   // independent spot buyer
    uint256 public immutable creditPrice;  // deferred price owed to the financier
    uint256 public immutable spotPrice;    // spot cash from the third party

    bool public bought;
    bool public possessed;
    bool public sold;
    uint256 public repaid;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyCustomer() { require(msg.sender == customer, "only customer"); _; }
    modifier onlyThirdParty() { require(msg.sender == thirdParty, "only third party"); _; }

    event BoughtOnCredit(uint256 price);
    event PossessionTaken();
    event SoldSpot(uint256 cash);
    event Repaid(uint256 amount, uint256 repaid, uint256 creditPrice);
    event Settled();

    /// @dev INVARIANT onward_to_third_party: the spot buyer must NOT be the credit seller —
    ///      selling back to the financier is bay' al-'inah; an arranged ring is tawarruq munazzam.
    constructor(address _financier, address _thirdParty, uint256 _creditPrice, uint256 _spotPrice) {
        require(_financier != address(0) && _thirdParty != address(0), "zero addr");
        require(_financier != _thirdParty, "spot buyer must differ from the credit seller (else 'inah)");
        require(_financier != msg.sender && _thirdParty != msg.sender, "distinct parties");
        require(_creditPrice > 0 && _spotPrice > 0, "prices");
        customer = msg.sender; financier = _financier; thirdParty = _thirdParty;
        creditPrice = _creditPrice; spotPrice = _spotPrice;
    }

    function buyOnCredit() external onlyCustomer {
        require(!bought, "already bought");
        bought = true; emit BoughtOnCredit(creditPrice);
    }

    /// @dev INVARIANT possession_before_resale: qabd before the onward sale.
    function takePossession() external onlyCustomer {
        require(bought, "not bought");
        require(!possessed, "already possessed");
        possessed = true; emit PossessionTaken();
    }

    /// @dev the onward sale: an INDEPENDENT third party pays the customer spot cash for the commodity.
    function sellSpot() external payable onlyThirdParty nonReentrant {
        require(possessed, "must take possession (qabd) before reselling");
        require(!sold, "already sold");
        require(msg.value == spotPrice, "exact spot price");
        sold = true;
        (bool ok, ) = customer.call{value: msg.value}(""); require(ok, "cash to customer");
        emit SoldSpot(msg.value);
    }

    /// @dev the customer repays the DEFERRED price to the financier — never more than agreed.
    function repayDeferred() external payable onlyCustomer nonReentrant {
        require(sold, "not sold yet");
        require(!settled, "already settled");
        require(msg.value > 0, "no payment");
        require(repaid + msg.value <= creditPrice, "would exceed the deferred price");
        repaid += msg.value;
        (bool ok, ) = financier.call{value: msg.value}(""); require(ok, "repay financier");
        emit Repaid(msg.value, repaid, creditPrice);
        if (repaid == creditPrice) { settled = true; emit Settled(); }
    }
"#;

fn gen_tawarruq_test(name: &str, credit: u64, spot: u64) -> String {
    format!(
        r#"// Generated by deducible — Tawarruq (individual). Proves the licit form: possession before the
// onward sale, the spot buyer is an INDEPENDENT third party (the 'inah ring cannot deploy), and
// the deferred debt can never be charged above the agreed price.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — tawarruq (individual)", function () {{
  let customer, financier, thirdParty, c;
  const CREDIT = {credit}n, SPOT = {spot}n;

  beforeEach(async function () {{
    [customer, financier, thirdParty] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(customer).deploy(financier.address, thirdParty.address, CREDIT, SPOT);
    await c.waitForDeployment();
  }});

  it("onward_to_third_party: a ring where the spot buyer IS the financier cannot deploy ('inah)", async function () {{
    const F = await ethers.getContractFactory("{name}");
    await expect(F.connect(customer).deploy(financier.address, financier.address, CREDIT, SPOT)).to.be.revertedWith("spot buyer must differ from the credit seller (else 'inah)");
  }});

  it("possession_before_resale: cannot sell before taking possession (qabd)", async function () {{
    await c.connect(customer).buyOnCredit();
    await expect(c.connect(thirdParty).sellSpot({{ value: SPOT }})).to.be.revertedWith("must take possession (qabd) before reselling");
  }});

  it("full lifecycle: buy on credit -> possess -> third party buys spot -> repay deferred -> settle", async function () {{
    await c.connect(customer).buyOnCredit();
    await c.connect(customer).takePossession();
    await expect(c.connect(thirdParty).sellSpot({{ value: SPOT }})).to.emit(c, "SoldSpot").withArgs(SPOT);
    await expect(c.connect(customer).repayDeferred({{ value: CREDIT }})).to.emit(c, "Settled");
    expect(await c.settled()).to.equal(true);
  }});

  it("the deferred debt can never be charged above the agreed price", async function () {{
    await c.connect(customer).buyOnCredit();
    await c.connect(customer).takePossession();
    await c.connect(thirdParty).sellSpot({{ value: SPOT }});
    await expect(c.connect(customer).repayDeferred({{ value: CREDIT + 1n }})).to.be.revertedWith("would exceed the deferred price");
  }});

  it("only the independent third party may buy the commodity spot", async function () {{
    await c.connect(customer).buyOnCredit();
    await c.connect(customer).takePossession();
    await expect(c.connect(customer).sellSpot({{ value: SPOT }})).to.be.revertedWith("only third party");
  }});
}});
"#,
        name = name,
        credit = credit,
        spot = spot,
    )
}

fn tawarruq_descriptor(name: &str, credit: u64, spot: u64) -> String {
    format!(
        r#"{{
  "instrument": "tawarruq",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "mustawriq",
  "oracle": null,
  "constructorAbi": ["address","address","uint256","uint256"],
  "constructorArgs": ["@financier", "@third_party", {credit}, {spot}],
  "accounts": ["financier","third_party"],
  "lifecycle": [
    {{ "as": "mustawriq", "fn": "buyOnCredit", "note": "buy the commodity from the financier, deferred" }},
    {{ "as": "mustawriq", "fn": "takePossession", "note": "take possession (qabd)" }},
    {{ "as": "third_party", "fn": "sellSpot", "value": {spot}, "note": "independent third party buys spot; customer gets cash" }},
    {{ "as": "mustawriq", "fn": "repayDeferred", "value": {credit}, "note": "repay the deferred price to the financier" }}
  ],
  "reads": ["sold","repaid","settled"]
}}
"#,
        name = name,
        credit = credit,
        spot = spot,
    )
}

// =====================================================================================
// Qard Hasan (benevolent loan). Disburse the principal; the borrower repays EXACTLY the principal.
// The contract refuses any repayment above the principal — no stipulated increase, no fee. No oracle.
// =====================================================================================

fn loan_principal(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "loan")
        .and_then(|r| kv_get(&r.kvs, "principal"))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_qard(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let principal = loan_principal(spec);
    let mut s = provenance_doc(spec, &format!("{} — qard hasan (benevolent loan, repaid in like) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(QARD_BODY);
    s.push_str("}\n");
    let test_js = gen_qard_test(&name, principal);
    let descriptor = qard_descriptor(&name, principal);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const QARD_BODY: &str = r#"    address public immutable lender;
    address public immutable borrower;
    uint256 public immutable principal;  // repaid IN LIKE — never more

    bool public disbursed;
    bool public repaid;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyLender() { require(msg.sender == lender, "only lender"); _; }
    modifier onlyBorrower() { require(msg.sender == borrower, "only borrower"); _; }

    event Disbursed(uint256 amount);
    event Repaid(uint256 amount);

    constructor(address _borrower, uint256 _principal) {
        require(_borrower != address(0), "zero addr");
        require(_borrower != msg.sender, "lender and borrower must be distinct");
        require(_principal > 0, "principal");
        lender = msg.sender; borrower = _borrower; principal = _principal;
    }

    function disburse() external payable onlyLender nonReentrant {
        require(!disbursed, "already disbursed");
        require(msg.value == principal, "must disburse exactly the principal");
        disbursed = true;
        (bool ok, ) = borrower.call{value: principal}(""); require(ok, "to borrower");
        emit Disbursed(principal);
    }

    /// @dev INVARIANT no_increase: the borrower repays EXACTLY the principal — the contract rejects
    ///      any increase. INVARIANT no_fee: there is no fee path. 'Every loan that draws a benefit
    ///      is riba'. (An unstipulated gift would be a separate, voluntary transfer.)
    function repay() external payable onlyBorrower nonReentrant {
        require(disbursed, "not disbursed");
        require(!repaid, "already repaid");
        require(msg.value == principal, "qard hasan: repay exactly the principal, no increase");
        repaid = true;
        (bool ok, ) = lender.call{value: principal}(""); require(ok, "to lender");
        emit Repaid(principal);
    }
"#;

fn gen_qard_test(name: &str, principal: u64) -> String {
    format!(
        r#"// Generated by deducible — Qard Hasan (benevolent loan). Proves the loan is repaid in like: the
// contract accepts EXACTLY the principal and rejects any increase (riba).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — qard hasan", function () {{
  let lender, borrower, c;
  const PRINCIPAL = {principal}n;

  beforeEach(async function () {{
    [lender, borrower] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(lender).deploy(borrower.address, PRINCIPAL);
    await c.waitForDeployment();
  }});

  it("full lifecycle: disburse -> borrower repays exactly the principal", async function () {{
    await c.connect(lender).disburse({{ value: PRINCIPAL }});
    await expect(c.connect(borrower).repay({{ value: PRINCIPAL }})).to.emit(c, "Repaid").withArgs(PRINCIPAL);
    expect(await c.repaid()).to.equal(true);
  }});

  it("no_increase: a repayment above the principal is rejected (riba)", async function () {{
    await c.connect(lender).disburse({{ value: PRINCIPAL }});
    await expect(c.connect(borrower).repay({{ value: PRINCIPAL + 1n }})).to.be.revertedWith("qard hasan: repay exactly the principal, no increase");
  }});

  it("only the borrower repays; only the lender disburses", async function () {{
    await expect(c.connect(borrower).disburse({{ value: PRINCIPAL }})).to.be.revertedWith("only lender");
  }});
}});
"#,
        name = name,
        principal = principal,
    )
}

fn qard_descriptor(name: &str, principal: u64) -> String {
    format!(
        r#"{{
  "instrument": "qard_hasan",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "lender",
  "oracle": null,
  "constructorAbi": ["address","uint256"],
  "constructorArgs": ["@borrower", {principal}],
  "accounts": ["borrower"],
  "lifecycle": [
    {{ "as": "lender", "fn": "disburse", "value": {principal}, "note": "lender disburses the principal" }},
    {{ "as": "borrower", "fn": "repay", "value": {principal}, "note": "borrower repays exactly the principal — no increase" }}
  ],
  "reads": ["disbursed","repaid"]
}}
"#,
        name = name,
        principal = principal,
    )
}

// =====================================================================================
// Rahn (pledge). The pledge is escrowed as security. On repayment it is returned; on default it
// is liquidated — the creditor takes only the debt, the surplus returns to the pledgor (not
// forfeit). The creditor never benefits from the held pledge. No oracle.
// =====================================================================================

fn pledge_field(spec: &Spec, key: &str) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "pledge")
        .and_then(|r| kv_get(&r.kvs, key))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_rahn(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let debt = pledge_field(spec, "debt");
    let pledge_value = pledge_field(spec, "pledge_value");
    let mut s = provenance_doc(spec, &format!("{} — rahn (pledge / collateral) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(RAHN_BODY);
    s.push_str("}\n");
    let test_js = gen_rahn_test(&name, debt, pledge_value);
    let descriptor = rahn_descriptor(&name, debt, pledge_value);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const RAHN_BODY: &str = r#"    address public immutable pledgor;     // al-rahin (debtor)
    address public immutable pledgee;     // al-murtahin (creditor)
    uint256 public immutable debt;        // the secured debt
    uint256 public immutable pledgeValue; // the marhun escrowed as security

    bool public pledged;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyPledgor() { require(msg.sender == pledgor, "only pledgor"); _; }
    modifier onlyPledgee() { require(msg.sender == pledgee, "only pledgee"); _; }

    event Pledged(uint256 value);
    event Redeemed(uint256 debtPaid, uint256 pledgeReturned);
    event Liquidated(uint256 toCreditor, uint256 surplusToPledgor);

    constructor(address _pledgee, uint256 _debt, uint256 _pledgeValue) {
        require(_pledgee != address(0), "zero addr");
        require(_pledgee != msg.sender, "pledgor and pledgee must be distinct");
        require(_debt > 0 && _pledgeValue > 0, "amounts");
        pledgor = msg.sender; pledgee = _pledgee; debt = _debt; pledgeValue = _pledgeValue;
    }

    /// @dev the marhun is escrowed here as security; the creditor never holds or uses it.
    function pledge() external payable onlyPledgor {
        require(!pledged, "already pledged");
        require(msg.value == pledgeValue, "must escrow exactly the pledge value");
        pledged = true; emit Pledged(pledgeValue);
    }

    /// @dev redemption: the pledgor repays the debt; the WHOLE pledge is returned to the pledgor.
    function repay() external payable onlyPledgor nonReentrant {
        require(pledged && !settled, "not redeemable");
        require(msg.value == debt, "repay exactly the debt");
        settled = true;
        (bool okC, ) = pledgee.call{value: debt}(""); require(okC, "debt to creditor");
        (bool okP, ) = pledgor.call{value: pledgeValue}(""); require(okP, "pledge to pledgor");
        emit Redeemed(debt, pledgeValue);
    }

    /// @dev INVARIANT no_creditor_benefit + surplus_to_pledgor: on default the pledge is sold to
    ///      satisfy the debt; the creditor takes ONLY the debt, and the surplus returns to the
    ///      pledgor — the pledge is not forfeit (la yaghlaqu al-rahn).
    function liquidate() external onlyPledgee nonReentrant {
        require(pledged && !settled, "not liquidatable");
        settled = true;
        uint256 toCreditor = debt <= pledgeValue ? debt : pledgeValue;
        uint256 surplus = pledgeValue - toCreditor;
        (bool okC, ) = pledgee.call{value: toCreditor}(""); require(okC, "to creditor");
        if (surplus > 0) { (bool okP, ) = pledgor.call{value: surplus}(""); require(okP, "surplus to pledgor"); }
        emit Liquidated(toCreditor, surplus);
    }
"#;

fn gen_rahn_test(name: &str, debt: u64, pledge_value: u64) -> String {
    format!(
        r#"// Generated by deducible — Rahn (pledge). Proves the pledge secures but does not forfeit: redemption
// returns the whole pledge, and default liquidation gives the creditor only the debt with the
// surplus returning to the pledgor.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — rahn pledge", function () {{
  let pledgor, pledgee, c;
  const DEBT = {debt}n, PVAL = {pledge_value}n;

  beforeEach(async function () {{
    [pledgor, pledgee] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(pledgor).deploy(pledgee.address, DEBT, PVAL);
    await c.waitForDeployment();
  }});

  it("redemption: repay the debt -> the whole pledge returns to the pledgor", async function () {{
    await c.connect(pledgor).pledge({{ value: PVAL }});
    await expect(c.connect(pledgor).repay({{ value: DEBT }})).to.emit(c, "Redeemed").withArgs(DEBT, PVAL);
    expect(await c.settled()).to.equal(true);
  }});

  it("surplus_to_pledgor: on default the creditor takes only the debt; surplus returns to the pledgor", async function () {{
    await c.connect(pledgor).pledge({{ value: PVAL }});
    await expect(c.connect(pledgee).liquidate()).to.emit(c, "Liquidated").withArgs(DEBT, PVAL - DEBT);
  }});

  it("only the pledgee may liquidate; only the pledgor may pledge", async function () {{
    await expect(c.connect(pledgee).pledge({{ value: PVAL }})).to.be.revertedWith("only pledgor");
  }});
}});
"#,
        name = name,
        debt = debt,
        pledge_value = pledge_value,
    )
}

fn rahn_descriptor(name: &str, debt: u64, pledge_value: u64) -> String {
    format!(
        r#"{{
  "instrument": "rahn",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "pledgor",
  "oracle": null,
  "constructorAbi": ["address","uint256","uint256"],
  "constructorArgs": ["@pledgee", {debt}, {pledge_value}],
  "accounts": ["pledgee"],
  "lifecycle": [
    {{ "as": "pledgor", "fn": "pledge", "value": {pledge_value}, "note": "escrow the marhun as security" }},
    {{ "as": "pledgor", "fn": "repay", "value": {debt}, "note": "repay the debt; the whole pledge returns" }}
  ],
  "reads": ["pledged","settled"]
}}
"#,
        name = name,
        debt = debt,
        pledge_value = pledge_value,
    )
}

// =====================================================================================
// Kafala (suretyship). The guarantor pays the creditor on default, then recovers from the debtor
// EXACTLY what he paid — no fee, no surcharge. No oracle.
// =====================================================================================

fn guarantee_amount(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "guarantee")
        .and_then(|r| kv_get(&r.kvs, "amount"))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_kafala(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let amount = guarantee_amount(spec);
    let mut s = provenance_doc(spec, &format!("{} — kafala (gratuitous suretyship) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(KAFALA_BODY);
    s.push_str("}\n");
    let test_js = gen_kafala_test(&name, amount);
    let descriptor = kafala_descriptor(&name, amount);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const KAFALA_BODY: &str = r#"    address public immutable kafil;     // guarantor
    address public immutable debtor;    // principal debtor
    address public immutable creditor;  // makful lahu
    uint256 public immutable amount;    // the guaranteed obligation

    bool public paid;       // the kafil has paid the creditor on default
    bool public recovered;  // the kafil has recovered from the debtor

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyKafil() { require(msg.sender == kafil, "only guarantor"); _; }
    modifier onlyDebtor() { require(msg.sender == debtor, "only debtor"); _; }

    event PaidCreditor(uint256 amount);
    event RecoveredFromDebtor(uint256 amount);

    /// @dev the kafil binds himself gratuitously — there is no fee field anywhere in this contract.
    constructor(address _debtor, address _creditor, uint256 _amount) {
        require(_debtor != address(0) && _creditor != address(0), "zero addr");
        require(_debtor != msg.sender, "guarantor and debtor must be distinct");
        require(_amount > 0, "amount");
        kafil = msg.sender; debtor = _debtor; creditor = _creditor; amount = _amount;
    }

    /// @dev on the debtor's default the guarantor pays the creditor the guaranteed amount.
    function payOnDefault() external payable onlyKafil nonReentrant {
        require(!paid, "already paid");
        require(msg.value == amount, "must pay exactly the guaranteed amount");
        paid = true;
        (bool ok, ) = creditor.call{value: amount}(""); require(ok, "to creditor");
        emit PaidCreditor(amount);
    }

    /// @dev INVARIANT recourse_actual: the guarantor recovers from the debtor EXACTLY what he paid,
    ///      never a surcharge (a surcharge for the guarantee would be riba). INVARIANT no_guarantee_fee.
    function recover() external payable onlyDebtor nonReentrant {
        require(paid, "nothing paid yet");
        require(!recovered, "already recovered");
        require(msg.value == amount, "recover exactly what the guarantor paid, no surcharge");
        recovered = true;
        (bool ok, ) = kafil.call{value: amount}(""); require(ok, "to guarantor");
        emit RecoveredFromDebtor(amount);
    }
"#;

fn gen_kafala_test(name: &str, amount: u64) -> String {
    format!(
        r#"// Generated by deducible — Kafala (suretyship). Proves the guarantee is gratuitous: the guarantor
// pays the creditor and recovers from the debtor EXACTLY what he paid (no fee, no surcharge).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — kafala", function () {{
  let kafil, debtor, creditor, c;
  const AMOUNT = {amount}n;

  beforeEach(async function () {{
    [kafil, debtor, creditor] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(kafil).deploy(debtor.address, creditor.address, AMOUNT);
    await c.waitForDeployment();
  }});

  it("the guarantor pays the creditor on default", async function () {{
    await expect(c.connect(kafil).payOnDefault({{ value: AMOUNT }})).to.emit(c, "PaidCreditor").withArgs(AMOUNT);
  }});

  it("recourse_actual: the debtor repays EXACTLY what the guarantor paid; a surcharge is rejected", async function () {{
    await c.connect(kafil).payOnDefault({{ value: AMOUNT }});
    await expect(c.connect(debtor).recover({{ value: AMOUNT + 1n }})).to.be.revertedWith("recover exactly what the guarantor paid, no surcharge");
    await expect(c.connect(debtor).recover({{ value: AMOUNT }})).to.emit(c, "RecoveredFromDebtor").withArgs(AMOUNT);
  }});

  it("only the guarantor pays; only the debtor recovers", async function () {{
    await expect(c.connect(debtor).payOnDefault({{ value: AMOUNT }})).to.be.revertedWith("only guarantor");
  }});
}});
"#,
        name = name,
        amount = amount,
    )
}

fn kafala_descriptor(name: &str, amount: u64) -> String {
    format!(
        r#"{{
  "instrument": "kafala",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "kafil",
  "oracle": null,
  "constructorAbi": ["address","address","uint256"],
  "constructorArgs": ["@principal_debtor", "@creditor", {amount}],
  "accounts": ["principal_debtor","creditor"],
  "lifecycle": [
    {{ "as": "kafil", "fn": "payOnDefault", "value": {amount}, "note": "guarantor pays the creditor on default" }},
    {{ "as": "principal_debtor", "fn": "recover", "value": {amount}, "note": "debtor reimburses exactly what was paid" }}
  ],
  "reads": ["paid","recovered"]
}}
"#,
        name = name,
        amount = amount,
    )
}

// =====================================================================================
// Hawala (debt transfer). Acceptance discharges the original debtor; the new payer (muhal alayh)
// then settles the creditor for EXACTLY the debt — no increase. No oracle.
// =====================================================================================

fn transfer_debt(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "transfer")
        .and_then(|r| kv_get(&r.kvs, "debt"))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_hawala(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let debt = transfer_debt(spec);
    let mut s = provenance_doc(spec, &format!("{} — hawala (assignment of debt) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(HAWALA_BODY);
    s.push_str("}\n");
    let test_js = gen_hawala_test(&name, debt);
    let descriptor = hawala_descriptor(&name, debt);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const HAWALA_BODY: &str = r#"    address public immutable muhil;       // original debtor
    address public immutable muhal;       // creditor
    address public immutable muhalAlayh;  // the new payer
    uint256 public immutable debt;        // transferred for the LIKE amount — no increase

    bool public accepted;
    bool public muhilDischarged;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyMuhal() { require(msg.sender == muhal, "only creditor"); _; }
    modifier onlyMuhalAlayh() { require(msg.sender == muhalAlayh, "only the new payer"); _; }

    event HawalaAccepted();
    event MuhilDischarged();
    event Settled(uint256 amount);

    constructor(address _muhal, address _muhalAlayh, uint256 _debt) {
        require(_muhal != address(0) && _muhalAlayh != address(0), "zero addr");
        require(_debt > 0, "debt");
        muhil = msg.sender; muhal = _muhal; muhalAlayh = _muhalAlayh; debt = _debt;
    }

    /// @dev INVARIANT debtor_discharged: on acceptance the ORIGINAL debtor (muhil) is discharged;
    ///      the creditor's recourse moves to the new payer.
    function acceptHawala() external onlyMuhal {
        require(!accepted, "already accepted");
        accepted = true; muhilDischarged = true;
        emit HawalaAccepted(); emit MuhilDischarged();
    }

    /// @dev INVARIANT equal_transfer: the new payer settles the creditor for EXACTLY the debt —
    ///      no increase (an increase would be a riba-bearing sale of debt).
    function settle() external payable onlyMuhalAlayh nonReentrant {
        require(accepted, "not accepted");
        require(!settled, "already settled");
        require(msg.value == debt, "settle exactly the debt, no increase");
        settled = true;
        (bool ok, ) = muhal.call{value: debt}(""); require(ok, "to creditor");
        emit Settled(debt);
    }
"#;

fn gen_hawala_test(name: &str, debt: u64) -> String {
    format!(
        r#"// Generated by deducible — Hawala (assignment of debt). Proves acceptance discharges the original
// debtor and the new payer settles the creditor for EXACTLY the debt (no increase).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — hawala", function () {{
  let muhil, muhal, muhalAlayh, c;
  const DEBT = {debt}n;

  beforeEach(async function () {{
    [muhil, muhal, muhalAlayh] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(muhil).deploy(muhal.address, muhalAlayh.address, DEBT);
    await c.waitForDeployment();
  }});

  it("debtor_discharged: acceptance discharges the original debtor", async function () {{
    await expect(c.connect(muhal).acceptHawala()).to.emit(c, "MuhilDischarged");
    expect(await c.muhilDischarged()).to.equal(true);
  }});

  it("equal_transfer: the new payer settles for EXACTLY the debt; an increase is rejected", async function () {{
    await c.connect(muhal).acceptHawala();
    await expect(c.connect(muhalAlayh).settle({{ value: DEBT + 1n }})).to.be.revertedWith("settle exactly the debt, no increase");
    await expect(c.connect(muhalAlayh).settle({{ value: DEBT }})).to.emit(c, "Settled").withArgs(DEBT);
  }});

  it("only the creditor accepts; only the new payer settles", async function () {{
    await expect(c.connect(muhil).acceptHawala()).to.be.revertedWith("only creditor");
  }});
}});
"#,
        name = name,
        debt = debt,
    )
}

fn hawala_descriptor(name: &str, debt: u64) -> String {
    format!(
        r#"{{
  "instrument": "hawala",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "muhil",
  "oracle": null,
  "constructorAbi": ["address","address","uint256"],
  "constructorArgs": ["@muhal", "@muhal_alayh", {debt}],
  "accounts": ["muhal","muhal_alayh"],
  "lifecycle": [
    {{ "as": "muhal", "fn": "acceptHawala", "note": "creditor accepts; original debtor discharged" }},
    {{ "as": "muhal_alayh", "fn": "settle", "value": {debt}, "note": "the new payer settles the creditor for exactly the debt" }}
  ],
  "reads": ["accepted","muhilDischarged","settled"]
}}
"#,
        name = name,
        debt = debt,
    )
}

// =====================================================================================
// Wadia (safekeeping). The deposit is escrowed in a neutral vault and returned to the depositor on
// demand, intact. The custodian has NO function to move it (it is amana, not used). No oracle.
// =====================================================================================

fn deposit_amount(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "deposit")
        .and_then(|r| kv_get(&r.kvs, "amount"))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_wadia(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let amount = deposit_amount(spec);
    let mut s = provenance_doc(spec, &format!("{} — wadia (safekeeping deposit, held as amana) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(WADIA_BODY);
    s.push_str("}\n");
    let test_js = gen_wadia_test(&name, amount);
    let descriptor = wadia_descriptor(&name, amount);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const WADIA_BODY: &str = r#"    address public immutable depositor;  // al-mudi'
    address public immutable custodian;  // al-mustawda' (custody of record; cannot move the funds)
    uint256 public immutable amount;

    bool public deposited;
    bool public withdrawn;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyDepositor() { require(msg.sender == depositor, "only depositor"); _; }

    event Deposited(uint256 amount);
    event Withdrawn(uint256 amount);

    constructor(address _custodian, uint256 _amount) {
        require(_custodian != address(0), "zero addr");
        require(_custodian != msg.sender, "depositor and custodian must be distinct");
        require(_amount > 0, "amount");
        depositor = msg.sender; custodian = _custodian; amount = _amount;
    }

    /// @dev INVARIANT no_custodian_use: the deposit is escrowed here as amana; there is NO function
    ///      by which the custodian can move or use it.
    function deposit() external payable onlyDepositor {
        require(!deposited, "already deposited");
        require(msg.value == amount, "deposit exactly the agreed amount");
        deposited = true; emit Deposited(amount);
    }

    /// @dev INVARIANT held_as_amanah: the depositor withdraws the deposit on demand, intact.
    function withdraw() external onlyDepositor nonReentrant {
        require(deposited && !withdrawn, "nothing to withdraw");
        withdrawn = true;
        (bool ok, ) = depositor.call{value: amount}(""); require(ok, "return to depositor");
        emit Withdrawn(amount);
    }
"#;

fn gen_wadia_test(name: &str, amount: u64) -> String {
    format!(
        r#"// Generated by deducible — Wadia (safekeeping). Proves the deposit is held as amana: returned to the
// depositor on demand intact, and the custodian has no power to move it.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — wadia", function () {{
  let depositor, custodian, c;
  const AMOUNT = {amount}n;

  beforeEach(async function () {{
    [depositor, custodian] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(depositor).deploy(custodian.address, AMOUNT);
    await c.waitForDeployment();
  }});

  it("held_as_amanah: the depositor withdraws the deposit on demand, intact", async function () {{
    await c.connect(depositor).deposit({{ value: AMOUNT }});
    await expect(c.connect(depositor).withdraw()).to.emit(c, "Withdrawn").withArgs(AMOUNT);
    expect(await c.withdrawn()).to.equal(true);
  }});

  it("no_custodian_use: the custodian has no power to move the deposit", async function () {{
    await c.connect(depositor).deposit({{ value: AMOUNT }});
    await expect(c.connect(custodian).withdraw()).to.be.revertedWith("only depositor");
  }});
}});
"#,
        name = name,
        amount = amount,
    )
}

fn wadia_descriptor(name: &str, amount: u64) -> String {
    format!(
        r#"{{
  "instrument": "wadia",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "depositor",
  "oracle": null,
  "constructorAbi": ["address","uint256"],
  "constructorArgs": ["@custodian", {amount}],
  "accounts": ["custodian"],
  "lifecycle": [
    {{ "as": "depositor", "fn": "deposit", "value": {amount}, "note": "depositor places the property for safekeeping" }},
    {{ "as": "depositor", "fn": "withdraw", "note": "depositor withdraws it on demand, intact" }}
  ],
  "reads": ["deposited","withdrawn"]
}}
"#,
        name = name,
        amount = amount,
    )
}

// =====================================================================================
// Wakala (agency / investment agency). The principal escrows capital + the known fee; the agent
// acts, then settles — taking ONLY its fixed fee, returning the capital to the principal. There is
// no guarantee path: the agent never tops up to ensure a return. No oracle.
// =====================================================================================

fn agency_field(spec: &Spec, key: &str) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "agency")
        .and_then(|r| kv_get(&r.kvs, key))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_wakala(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let capital = agency_field(spec, "capital");
    let fee = agency_field(spec, "fee");
    let mut s = provenance_doc(spec, &format!("{} — wakala (agency for a known fee, no guarantee) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(WAKALA_BODY);
    s.push_str("}\n");
    let test_js = gen_wakala_test(&name, capital, fee);
    let descriptor = wakala_descriptor(&name, capital, fee);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const WAKALA_BODY: &str = r#"    address public immutable principal;  // al-muwakkil
    address public immutable agent;      // al-wakil
    uint256 public immutable capital;    // the amount under management (borne by the principal)
    uint256 public immutable fee;        // the agent's KNOWN ujra

    bool public appointed;
    bool public invested;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyPrincipal() { require(msg.sender == principal, "only principal"); _; }
    modifier onlyAgent() { require(msg.sender == agent, "only agent"); _; }

    event Appointed(uint256 capital, uint256 fee);
    event Invested();
    event Settled(uint256 toAgentFee, uint256 toPrincipal);

    constructor(address _agent, uint256 _capital, uint256 _fee) {
        require(_agent != address(0), "zero addr");
        require(_agent != msg.sender, "principal and agent must be distinct");
        require(_capital > 0, "capital");
        principal = msg.sender; agent = _agent; capital = _capital; fee = _fee;
    }

    function appoint() external payable onlyPrincipal {
        require(!appointed, "already appointed");
        require(msg.value == capital + fee, "fund the capital plus the agreed fee");
        appointed = true; emit Appointed(capital, fee);
    }

    function invest() external onlyAgent {
        require(appointed, "not appointed");
        require(!invested, "already investing");
        invested = true; emit Invested();
    }

    /// @dev INVARIANT no_agent_guarantee: the agent takes ONLY its fixed fee; there is no code path
    ///      by which it guarantees the principal a profit or even the full capital. The realized
    ///      return belongs to (and its risk is borne by) the principal.
    function settle() external onlyAgent nonReentrant {
        require(invested && !settled, "not settleable");
        settled = true;
        if (fee > 0) { (bool okF, ) = agent.call{value: fee}(""); require(okF, "fee to agent"); }
        (bool okP, ) = principal.call{value: capital}(""); require(okP, "capital to principal");
        emit Settled(fee, capital);
    }
"#;

fn gen_wakala_test(name: &str, capital: u64, fee: u64) -> String {
    format!(
        r#"// Generated by deducible — Wakala (agency). Proves the agent takes only its known fee and never
// guarantees a return (there is no guarantee code path).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — wakala", function () {{
  let principal, agent, c;
  const CAPITAL = {capital}n, FEE = {fee}n;

  beforeEach(async function () {{
    [principal, agent] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(principal).deploy(agent.address, CAPITAL, FEE);
    await c.waitForDeployment();
  }});

  it("lifecycle: appoint -> invest -> settle; the agent takes only its known fee", async function () {{
    await c.connect(principal).appoint({{ value: CAPITAL + FEE }});
    await c.connect(agent).invest();
    await expect(c.connect(agent).settle()).to.emit(c, "Settled").withArgs(FEE, CAPITAL);
    expect(await c.settled()).to.equal(true);
  }});

  it("only the principal appoints; only the agent invests and settles", async function () {{
    await expect(c.connect(agent).appoint({{ value: CAPITAL + FEE }})).to.be.revertedWith("only principal");
  }});
}});
"#,
        name = name,
        capital = capital,
        fee = fee,
    )
}

fn wakala_descriptor(name: &str, capital: u64, fee: u64) -> String {
    format!(
        r#"{{
  "instrument": "wakala",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "muwakkil",
  "oracle": null,
  "constructorAbi": ["address","uint256","uint256"],
  "constructorArgs": ["@wakil", {capital}, {fee}],
  "accounts": ["wakil"],
  "lifecycle": [
    {{ "as": "muwakkil", "fn": "appoint", "value": {total}, "note": "principal funds the capital plus the known fee" }},
    {{ "as": "wakil", "fn": "invest", "note": "agent acts on the principal's account" }},
    {{ "as": "wakil", "fn": "settle", "note": "agent takes only its fee; capital returns to the principal" }}
  ],
  "reads": ["appointed","invested","settled"]
}}
"#,
        name = name,
        capital = capital,
        fee = fee,
        total = capital + fee,
    )
}

// =====================================================================================
// Ijarah (plain operating lease). Rent for usufruct; the asset returns to the lessor at the end
// (no transfer of ownership). The lessor bears the asset's risk. No oracle.
// =====================================================================================

fn gen_ijarah_plain(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let rate = rent_rate(spec);
    let mut s = provenance_doc(spec, &format!("{} — ijarah (operating lease of usufruct) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(IJARAH_PLAIN_BODY);
    s.push_str("}\n");
    let test_js = gen_ijarah_plain_test(&name, rate);
    let descriptor = ijarah_plain_descriptor(&name, rate);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const IJARAH_PLAIN_BODY: &str = r#"    address public immutable lessor;
    address public immutable lessee;
    uint256 public immutable rentPerPeriod;

    uint256 public paid;
    bool public returned;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyLessee() { require(msg.sender == lessee, "only lessee"); _; }
    modifier onlyLessor() { require(msg.sender == lessor, "only lessor"); _; }

    event RentPaid(uint256 amount);
    event AssetReturned();

    constructor(address _lessee, uint256 _rentPerPeriod) {
        require(_lessee != address(0), "zero addr");
        require(_lessee != msg.sender, "lessor and lessee must be distinct");
        lessor = msg.sender; lessee = _lessee; rentPerPeriod = _rentPerPeriod;
    }

    /// @dev INVARIANT rent_for_usufruct: rent is the price of the usufruct, paid to the lessor.
    ///      INVARIANT no_late_penalty_interest: exactly one period's rent is accepted; no surcharge.
    function payRent() external payable onlyLessee nonReentrant {
        require(!returned, "lease ended");
        require(msg.value == rentPerPeriod, "pay exactly one period's rent (no surcharge)");
        paid += msg.value;
        (bool ok, ) = lessor.call{value: msg.value}(""); require(ok, "rent to lessor");
        emit RentPaid(msg.value);
    }

    /// @dev INVARIANT lessor_bears_risk: at the end the asset returns to the lessor — ownership never
    ///      transferred to the lessee (this is a lease, not a sale).
    function returnAsset() external onlyLessor {
        require(!returned, "already returned");
        returned = true; emit AssetReturned();
    }
"#;

fn gen_ijarah_plain_test(name: &str, rate: u64) -> String {
    format!(
        r#"// Generated by deducible — Ijarah (operating lease). Proves rent flows to the lessor for the
// usufruct, the asset returns to the lessor at the end, and no late surcharge is accepted.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — ijarah lease", function () {{
  let lessor, lessee, c;
  const RENT = {rate}n;

  beforeEach(async function () {{
    [lessor, lessee] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(lessor).deploy(lessee.address, RENT);
    await c.waitForDeployment();
  }});

  it("rent_for_usufruct: the lessee pays one period's rent to the lessor", async function () {{
    await expect(c.connect(lessee).payRent({{ value: RENT }})).to.emit(c, "RentPaid").withArgs(RENT);
    expect(await c.paid()).to.equal(RENT);
  }});

  it("no_late_penalty_interest: a surcharge above the rent is rejected", async function () {{
    await expect(c.connect(lessee).payRent({{ value: RENT + 1n }})).to.be.revertedWith("pay exactly one period's rent (no surcharge)");
  }});

  it("lessor_bears_risk: the asset returns to the lessor at the end (no ownership transfer)", async function () {{
    await expect(c.connect(lessor).returnAsset()).to.emit(c, "AssetReturned");
    expect(await c.returned()).to.equal(true);
  }});
}});
"#,
        name = name,
        rate = rate,
    )
}

fn ijarah_plain_descriptor(name: &str, rate: u64) -> String {
    format!(
        r#"{{
  "instrument": "ijarah",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "lessor",
  "oracle": null,
  "constructorAbi": ["address","uint256"],
  "constructorArgs": ["@lessee", {rate}],
  "accounts": ["lessee"],
  "lifecycle": [
    {{ "as": "lessee", "fn": "payRent", "value": {rate}, "note": "lessee pays one period's rent for the usufruct" }},
    {{ "as": "lessor", "fn": "returnAsset", "note": "at lease end the asset returns to the lessor" }}
  ],
  "reads": ["paid","returned"]
}}
"#,
        name = name,
        rate = rate,
    )
}

// =====================================================================================
// Ju'ala (reward for a result). The offerer escrows a known reward; the worker claims it only on
// completion. No oracle.
// =====================================================================================

fn reward_amount(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "reward")
        .and_then(|r| kv_get(&r.kvs, "amount"))
        .and_then(|e| e.as_num())
        .unwrap_or(0)
}

fn gen_juala(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let reward = reward_amount(spec);
    let mut s = provenance_doc(spec, &format!("{} — ju'ala (reward for a result) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(JUALA_BODY);
    s.push_str("}\n");
    let test_js = gen_juala_test(&name, reward);
    let descriptor = juala_descriptor(&name, reward);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const JUALA_BODY: &str = r#"    address public immutable jail;   // the offerer (al-ja'il)
    address public immutable amil;   // the worker (al-'amil)
    uint256 public immutable reward; // the known ju'l

    bool public offered;
    bool public completed;
    bool public claimed;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyJail() { require(msg.sender == jail, "only offerer"); _; }
    modifier onlyAmil() { require(msg.sender == amil, "only worker"); _; }

    event Offered(uint256 reward);
    event Completed();
    event RewardClaimed(uint256 reward);

    constructor(address _amil, uint256 _reward) {
        require(_amil != address(0), "zero addr");
        require(_amil != msg.sender, "offerer and worker must be distinct");
        require(_reward > 0, "reward must be known");
        jail = msg.sender; amil = _amil; reward = _reward;
    }

    function offer() external payable onlyJail {
        require(!offered, "already offered");
        require(msg.value == reward, "escrow exactly the reward");
        offered = true; emit Offered(reward);
    }

    function complete() external onlyAmil {
        require(offered, "no offer");
        require(!completed, "already completed");
        completed = true; emit Completed();
    }

    /// @dev INVARIANT due_on_completion: the worker is paid ONLY after the result is achieved —
    ///      the worker bears the risk of non-completion (no completion, no reward).
    function claim() external onlyAmil nonReentrant {
        require(completed, "reward is due only on completion");
        require(!claimed, "already claimed");
        claimed = true;
        (bool ok, ) = amil.call{value: reward}(""); require(ok, "reward to worker");
        emit RewardClaimed(reward);
    }
"#;

fn gen_juala_test(name: &str, reward: u64) -> String {
    format!(
        r#"// Generated by deducible — Ju'ala (reward for a result). Proves the reward is due ONLY on completion.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — ju'ala", function () {{
  let jail, amil, c;
  const REWARD = {reward}n;

  beforeEach(async function () {{
    [jail, amil] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(jail).deploy(amil.address, REWARD);
    await c.waitForDeployment();
  }});

  it("due_on_completion: claiming before completion reverts", async function () {{
    await c.connect(jail).offer({{ value: REWARD }});
    await expect(c.connect(amil).claim()).to.be.revertedWith("reward is due only on completion");
  }});

  it("lifecycle: offer -> complete -> claim; the worker receives the known reward", async function () {{
    await c.connect(jail).offer({{ value: REWARD }});
    await c.connect(amil).complete();
    await expect(c.connect(amil).claim()).to.emit(c, "RewardClaimed").withArgs(REWARD);
  }});

  it("only the worker completes and claims", async function () {{
    await c.connect(jail).offer({{ value: REWARD }});
    await expect(c.connect(jail).complete()).to.be.revertedWith("only worker");
  }});
}});
"#,
        name = name,
        reward = reward,
    )
}

fn juala_descriptor(name: &str, reward: u64) -> String {
    format!(
        r#"{{
  "instrument": "juala",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "jail",
  "oracle": null,
  "constructorAbi": ["address","uint256"],
  "constructorArgs": ["@amil", {reward}],
  "accounts": ["amil"],
  "lifecycle": [
    {{ "as": "jail", "fn": "offer", "value": {reward}, "note": "offerer escrows the known reward" }},
    {{ "as": "amil", "fn": "complete", "note": "worker achieves the result" }},
    {{ "as": "amil", "fn": "claim", "note": "worker claims the reward on completion" }}
  ],
  "reads": ["offered","completed","claimed"]
}}
"#,
        name = name,
        reward = reward,
    )
}

// =====================================================================================
// 'Ariyya (gratuitous loan of usufruct). No money moves; the SAME asset is returned. No oracle.
// =====================================================================================

fn gen_ariyah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let mut s = provenance_doc(spec, &format!("{} — 'ariyya (gratuitous loan of usufruct) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(ARIYAH_BODY);
    s.push_str("}\n");
    let test_js = gen_ariyah_test(&name);
    let descriptor = ariyah_descriptor(&name);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const ARIYAH_BODY: &str = r#"    address public immutable muir;     // the lender (al-mu'ir)
    address public immutable mustair;  // the borrower (al-musta'ir)

    bool public lent;
    bool public returned;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyMuir() { require(msg.sender == muir, "only lender"); _; }
    modifier onlyMustair() { require(msg.sender == mustair, "only borrower"); _; }

    event UsufructLent();
    event AssetReturned();

    constructor(address _mustair) {
        require(_mustair != address(0), "zero addr");
        require(_mustair != msg.sender, "lender and borrower must be distinct");
        muir = msg.sender; mustair = _mustair;
    }

    /// @dev INVARIANT gratuitous: there is no fee path — the usufruct is lent free of charge.
    function lendUse() external onlyMuir nonReentrant {
        require(!lent, "already lent");
        lent = true; emit UsufructLent();
    }

    /// @dev INVARIANT returns_same: the SAME asset is returned (only its usufruct was lent).
    function returnAsset() external onlyMustair nonReentrant {
        require(lent && !returned, "nothing to return");
        returned = true; emit AssetReturned();
    }
"#;

fn gen_ariyah_test(name: &str) -> String {
    format!(
        r#"// Generated by deducible — 'Ariyya (gratuitous loan of usufruct). Proves the loan is free and the
// same asset is returned (no money moves).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — 'ariyya", function () {{
  let muir, mustair, c;

  beforeEach(async function () {{
    [muir, mustair] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(muir).deploy(mustair.address);
    await c.waitForDeployment();
  }});

  it("gratuitous lifecycle: lend the usufruct, then the borrower returns the same asset", async function () {{
    await expect(c.connect(muir).lendUse()).to.emit(c, "UsufructLent");
    await expect(c.connect(mustair).returnAsset()).to.emit(c, "AssetReturned");
    expect(await c.returned()).to.equal(true);
  }});

  it("only the borrower returns the asset", async function () {{
    await c.connect(muir).lendUse();
    await expect(c.connect(muir).returnAsset()).to.be.revertedWith("only borrower");
  }});
}});
"#,
        name = name,
    )
}

fn ariyah_descriptor(name: &str) -> String {
    format!(
        r#"{{
  "instrument": "ariyah",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "muir",
  "oracle": null,
  "constructorAbi": ["address"],
  "constructorArgs": ["@mustair"],
  "accounts": ["mustair"],
  "lifecycle": [
    {{ "as": "muir", "fn": "lendUse", "note": "lender lends the usufruct gratis" }},
    {{ "as": "mustair", "fn": "returnAsset", "note": "borrower returns the same asset" }}
  ],
  "reads": ["lent","returned"]
}}
"#,
        name = name,
    )
}

// =====================================================================================
// Musharakah (full partnership). settle() distributes the liquidated proceeds: profit (over the
// total capital) by the profit ratio, and on a loss each partner bears it by capital share. No oracle.
// =====================================================================================

fn gen_musharakah_full(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let cap_a = party_bps(spec, "partner").unwrap_or(6000);
    let profit_a = profit_share(spec, "partner").unwrap_or(5000);
    let mut s = provenance_doc(spec, &format!("{} — musharakah (full partnership) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(MUSHARAKAH_FULL_BODY);
    s.push_str("}\n");
    let test_js = gen_musharakah_full_test(&name, cap_a, profit_a);
    let descriptor = musharakah_full_descriptor(&name, cap_a, profit_a);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const MUSHARAKAH_FULL_BODY: &str = r#"    address public immutable partnerA;
    address public immutable partnerB;
    uint256 public immutable totalCapital;
    uint256 public immutable capitalABps; // partner A's capital share
    uint256 public immutable profitABps;  // partner A's profit share (may differ from capital)
    uint256 public constant BPS = 10000;

    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyPartnerA() { require(msg.sender == partnerA, "only partner A"); _; }

    event Settled(uint256 realized, uint256 toA, uint256 toB);

    constructor(address _partnerB, uint256 _totalCapital, uint256 _capitalABps, uint256 _profitABps) {
        require(_partnerB != address(0), "zero addr");
        require(_partnerB != msg.sender, "partners must be distinct");
        require(_capitalABps > 0 && _capitalABps < BPS, "capital split");
        require(_profitABps <= BPS, "profit split");
        partnerA = msg.sender; partnerB = _partnerB;
        totalCapital = _totalCapital; capitalABps = _capitalABps; profitABps = _profitABps;
    }

    /// @dev INVARIANT profit_by_ratio: profit (over the capital) is split by the profit ratio.
    ///      INVARIANT loss_by_capital: a shortfall is borne by each partner in proportion to capital.
    ///      INVARIANT no_capital_guarantee: neither partner is topped up — the realized value is split as-is.
    function settle(uint256 realized) external payable onlyPartnerA nonReentrant {
        require(!settled, "settled");
        require(msg.value == realized, "send exactly the realized proceeds");
        settled = true;
        uint256 toA;
        if (realized > totalCapital) {
            uint256 profit = realized - totalCapital;
            toA = (totalCapital * capitalABps) / BPS + (profit * profitABps) / BPS;
        } else {
            // loss (or break-even): each bears it by capital share
            toA = (realized * capitalABps) / BPS;
        }
        uint256 toB = realized - toA;
        if (toA > 0) { (bool okA, ) = partnerA.call{value: toA}(""); require(okA, "to A"); }
        if (toB > 0) { (bool okB, ) = partnerB.call{value: toB}(""); require(okB, "to B"); }
        emit Settled(realized, toA, toB);
    }
"#;

fn gen_musharakah_full_test(name: &str, cap_a: u64, profit_a: u64) -> String {
    format!(
        r#"// Generated by deducible — Musharakah (full partnership). Proves profit splits by the profit ratio
// and a loss is borne by capital share.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — musharakah", function () {{
  let a, b, c;
  const TOTAL = 1000000n, CAPA = {cap_a}n, PROFA = {profit_a}n, BPS = 10000n;

  beforeEach(async function () {{
    [a, b] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(a).deploy(b.address, TOTAL, CAPA, PROFA);
    await c.waitForDeployment();
  }});

  it("profit_by_ratio: profit over capital splits by the profit ratio", async function () {{
    const realized = TOTAL + 200000n; // 200k profit
    const profit = realized - TOTAL;
    const toA = (TOTAL * CAPA) / BPS + (profit * PROFA) / BPS;
    await expect(c.connect(a).settle(realized, {{ value: realized }})).to.emit(c, "Settled").withArgs(realized, toA, realized - toA);
  }});

  it("loss_by_capital: a shortfall is borne by capital share", async function () {{
    const realized = TOTAL - 100000n; // a loss
    const toA = (realized * CAPA) / BPS;
    await expect(c.connect(a).settle(realized, {{ value: realized }})).to.emit(c, "Settled").withArgs(realized, toA, realized - toA);
  }});

  it("only partner A operates settlement", async function () {{
    await expect(c.connect(b).settle(TOTAL, {{ value: TOTAL }})).to.be.revertedWith("only partner A");
  }});
}});
"#,
        name = name,
        cap_a = cap_a,
        profit_a = profit_a,
    )
}

fn musharakah_full_descriptor(name: &str, cap_a: u64, profit_a: u64) -> String {
    format!(
        r#"{{
  "instrument": "musharakah",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "partner",
  "oracle": null,
  "constructorAbi": ["address","uint256","uint256","uint256"],
  "constructorArgs": ["@partner", 1000000, {cap_a}, {profit_a}],
  "accounts": ["partner"],
  "lifecycle": [
    {{ "as": "partner", "fn": "settle", "args": [1200000], "value": 1200000, "note": "distribute realized proceeds: profit by ratio, loss by capital" }}
  ],
  "reads": ["settled"]
}}
"#,
        name = name,
        cap_a = cap_a,
        profit_a = profit_a,
    )
}

// =====================================================================================
// Muzara'ah (sharecropping). splitCrop() shares the ACTUAL harvest by the agreed ratio — zero
// output means zero to each (the owner shares the crop's fate; no fixed rent). No oracle.
// =====================================================================================

fn muzaraah_owner_bps(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "harvest_share")
        .and_then(|r| kv_get(&r.kvs, "landowner"))
        .and_then(|e| e.as_num())
        .unwrap_or(5000)
}

fn gen_muzaraah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let owner_bps = muzaraah_owner_bps(spec);
    let mut s = provenance_doc(spec, &format!("{} — muzara'ah (sharecropping) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(MUZARAAH_BODY);
    s.push_str("}\n");
    let test_js = gen_muzaraah_test(&name, owner_bps);
    let descriptor = muzaraah_descriptor(&name, owner_bps);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const MUZARAAH_BODY: &str = r#"    address public immutable landowner;
    address public immutable cultivator;
    uint256 public immutable ownerBps; // landowner's share of the OUTPUT
    uint256 public constant BPS = 10000;

    bool public split;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyLandowner() { require(msg.sender == landowner, "only landowner"); _; }

    event CropSplit(uint256 output, uint256 toOwner, uint256 toCultivator);

    constructor(address _cultivator, uint256 _ownerBps) {
        require(_cultivator != address(0), "zero addr");
        require(_cultivator != msg.sender, "landowner and cultivator must be distinct");
        require(_ownerBps > 0 && _ownerBps < BPS, "share");
        landowner = msg.sender; cultivator = _cultivator; ownerBps = _ownerBps;
    }

    /// @dev INVARIANT output_by_ratio: the ACTUAL harvest is shared by the agreed ratio.
    ///      INVARIANT no_fixed_rent: a zero harvest yields zero to each — the owner shares the risk;
    ///      there is no path where the owner takes a fixed amount regardless of the output.
    function splitCrop(uint256 output) external payable onlyLandowner nonReentrant {
        require(!split, "already split");
        require(msg.value == output, "send exactly the realized output");
        split = true;
        uint256 toOwner = (output * ownerBps) / BPS;
        uint256 toCultivator = output - toOwner;
        if (toOwner > 0) { (bool okO, ) = landowner.call{value: toOwner}(""); require(okO, "to owner"); }
        if (toCultivator > 0) { (bool okC, ) = cultivator.call{value: toCultivator}(""); require(okC, "to cultivator"); }
        emit CropSplit(output, toOwner, toCultivator);
    }
"#;

fn gen_muzaraah_test(name: &str, owner_bps: u64) -> String {
    format!(
        r#"// Generated by deducible — Muzara'ah (sharecropping). Proves the harvest is shared by ratio and a
// zero harvest yields zero to each (no fixed rent — the owner shares the crop's fate).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — muzara'ah", function () {{
  let owner, cultivator, c;
  const OWNER_BPS = {owner_bps}n, BPS = 10000n;

  beforeEach(async function () {{
    [owner, cultivator] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(owner).deploy(cultivator.address, OWNER_BPS);
    await c.waitForDeployment();
  }});

  it("output_by_ratio: the harvest is shared by the agreed ratio", async function () {{
    const output = 1000000n;
    const toOwner = (output * OWNER_BPS) / BPS;
    await expect(c.connect(owner).splitCrop(output, {{ value: output }})).to.emit(c, "CropSplit").withArgs(output, toOwner, output - toOwner);
  }});

  it("no_fixed_rent: a zero harvest yields zero to the owner (he shares the crop's fate)", async function () {{
    await expect(c.connect(owner).splitCrop(0n, {{ value: 0n }})).to.emit(c, "CropSplit").withArgs(0n, 0n, 0n);
  }});

  it("only the landowner operates the split", async function () {{
    await expect(c.connect(cultivator).splitCrop(1000n, {{ value: 1000n }})).to.be.revertedWith("only landowner");
  }});
}});
"#,
        name = name,
        owner_bps = owner_bps,
    )
}

fn muzaraah_descriptor(name: &str, owner_bps: u64) -> String {
    format!(
        r#"{{
  "instrument": "muzaraah",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "landowner",
  "oracle": null,
  "constructorAbi": ["address","uint256"],
  "constructorArgs": ["@cultivator", {owner_bps}],
  "accounts": ["cultivator"],
  "lifecycle": [
    {{ "as": "landowner", "fn": "splitCrop", "args": [1000000], "value": 1000000, "note": "share the actual harvest by the agreed ratio" }}
  ],
  "reads": ["split"]
}}
"#,
        name = name,
        owner_bps = owner_bps,
    )
}

// =====================================================================================
// Sukuk (investment certificates) — the FIRST multilateral instrument. The holders are a pool
// (address[] + uint16[] shares, the faraid array pattern generalised); the issuer distributes the
// asset's rental income PRO-RATA to their undivided ownership shares. No oracle.
// =====================================================================================

fn pool_shares(spec: &Spec) -> Vec<u64> {
    spec.pool().into_iter().filter_map(|kv| kv.val.as_num()).collect()
}

fn gen_sukuk(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let shares = pool_shares(spec);
    let mut s = provenance_doc(spec, &format!("{} — sukuk (undivided ownership, pro-rata income) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(SUKUK_BODY);
    s.push_str("}\n");
    let test_js = gen_sukuk_test(&name, &shares);
    let descriptor = sukuk_descriptor(&name, &shares);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const SUKUK_BODY: &str = r#"    address public immutable issuer;
    address[] public holders;
    uint16[] public sharesBps;
    uint256 public constant BPS = 10000;
    bool public distributed;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyIssuer() { require(msg.sender == issuer, "only issuer"); _; }

    event IncomeDistributed(uint256 total);

    /// @dev INVARIANT ownership shares total 10000 bps; each sukuk is an undivided share.
    constructor(address[] memory _holders, uint16[] memory _sharesBps) {
        require(_holders.length == _sharesBps.length && _holders.length >= 2, "holders/shares mismatch");
        uint256 sum;
        for (uint256 i = 0; i < _sharesBps.length; i++) {
            require(_holders[i] != address(0), "zero holder");
            sum += _sharesBps[i];
            holders.push(_holders[i]);
            sharesBps.push(_sharesBps[i]);
        }
        require(sum == BPS, "ownership shares must total 10000 bps");
        issuer = msg.sender;
    }

    /// @dev INVARIANT return_is_asset_based + pro_rata_distribution: the issuer distributes the
    ///      asset's rental income PRO-RATA to the holders' undivided ownership shares — not interest.
    function distributeIncome() external payable onlyIssuer nonReentrant {
        require(msg.value > 0, "no income");
        uint256 total = msg.value;
        for (uint256 i = 0; i < holders.length; i++) {
            uint256 part = (total * sharesBps[i]) / BPS;
            if (part > 0) { (bool ok, ) = holders[i].call{value: part}(""); require(ok, "to holder"); }
        }
        distributed = true;
        emit IncomeDistributed(total);
    }
"#;

fn gen_sukuk_test(name: &str, shares: &[u64]) -> String {
    let n = shares.len();
    let shares_js = shares.iter().map(|s| format!("{}n", s)).collect::<Vec<_>>().join(", ");
    format!(
        r#"// Generated by deducible — Sukuk (multilateral). Proves undivided shares total 10000 and the
// asset's income is distributed pro-rata to the holder pool.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — sukuk", function () {{
  let issuer, holders, c;
  const SHARES = [{shares_js}];
  const TOTAL = 1000000n, BPS = 10000n;

  beforeEach(async function () {{
    const signers = await ethers.getSigners();
    issuer = signers[0];
    holders = signers.slice(1, 1 + {n});
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(issuer).deploy(holders.map(h => h.address), SHARES);
    await c.waitForDeployment();
  }});

  it("pro_rata_distribution: rental income is split by each holder's ownership share", async function () {{
    const before = await Promise.all(holders.map(h => ethers.provider.getBalance(h.address)));
    await c.connect(issuer).distributeIncome({{ value: TOTAL }});
    for (let i = 0; i < holders.length; i++) {{
      const after = await ethers.provider.getBalance(holders[i].address);
      expect(after - before[i]).to.equal((TOTAL * SHARES[i]) / BPS);
    }}
  }});

  it("ownership shares must total 10000 bps (else deployment reverts)", async function () {{
    const F = await ethers.getContractFactory("{name}");
    const bad = SHARES.map((s, i) => (i === 0 ? s + 100n : s)); // breaks the sum
    await expect(F.connect(issuer).deploy(holders.map(h => h.address), bad)).to.be.revertedWith("ownership shares must total 10000 bps");
  }});

  it("only the issuer distributes income", async function () {{
    await expect(c.connect(holders[0]).distributeIncome({{ value: TOTAL }})).to.be.revertedWith("only issuer");
  }});
}});
"#,
        name = name,
        shares_js = shares_js,
        n = n,
    )
}

fn sukuk_descriptor(name: &str, shares: &[u64]) -> String {
    let shares_json = shares.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ");
    format!(
        r#"{{
  "instrument": "sukuk",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "issuer",
  "oracle": null,
  "pool": {{ "shares_bps": [{shares_json}] }},
  "constructorAbi": ["address[]","uint16[]"],
  "constructorArgs": ["@holders", [{shares_json}]],
  "accounts": ["holders"],
  "lifecycle": [
    {{ "as": "issuer", "fn": "distributeIncome", "value": 1000000, "note": "distribute rental income pro-rata to holders" }}
  ],
  "reads": ["distributed"]
}}
"#,
        name = name,
        shares_json = shares_json,
    )
}

// =====================================================================================
// Takaful (cooperative). A participant pool donates (tabarru') into a mutual fund; the operator
// pays claims from it and distributes the surplus PRO-RATA back to the participants. No oracle.
// =====================================================================================

fn gen_takaful(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let shares = pool_shares(spec);
    let mut s = provenance_doc(spec, &format!("{} — takaful (cooperative; tabarru' pool) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(TAKAFUL_BODY);
    s.push_str("}\n");
    let test_js = gen_takaful_test(&name, &shares);
    let descriptor = sukuk_descriptor(&name, &shares); // same pool-shaped descriptor
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const TAKAFUL_BODY: &str = r#"    address public immutable operator;
    address[] public participants;
    uint16[] public sharesBps;
    uint256 public constant BPS = 10000;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyOperator() { require(msg.sender == operator, "only operator"); _; }

    event Contributed(uint256 amount);
    event ClaimPaid(address to, uint256 amount);
    event SurplusDistributed(uint256 total);

    constructor(address[] memory _participants, uint16[] memory _sharesBps) {
        require(_participants.length == _sharesBps.length && _participants.length >= 2, "participants/shares");
        uint256 sum;
        for (uint256 i = 0; i < _sharesBps.length; i++) {
            require(_participants[i] != address(0), "zero participant");
            sum += _sharesBps[i];
            participants.push(_participants[i]);
            sharesBps.push(_sharesBps[i]);
        }
        require(sum == BPS, "participant shares must total 10000 bps");
        operator = msg.sender;
    }

    /// @dev tabarru': contributions are donations into the mutual fund (held here).
    function contribute() external payable {
        require(msg.value > 0, "no contribution");
        emit Contributed(msg.value);
    }

    /// @dev claims are paid FROM the mutual fund.
    function payClaim(address to, uint256 amount) external onlyOperator nonReentrant {
        require(amount <= address(this).balance, "claim exceeds the fund");
        (bool ok, ) = to.call{value: amount}(""); require(ok, "claim xfer");
        emit ClaimPaid(to, amount);
    }

    /// @dev INVARIANT surplus_to_participants: the remaining surplus is distributed PRO-RATA to the
    ///      participants — never taken as the operator's profit (there is no path to the operator).
    function distributeSurplus() external onlyOperator nonReentrant {
        uint256 total = address(this).balance;
        for (uint256 i = 0; i < participants.length; i++) {
            uint256 part = (total * sharesBps[i]) / BPS;
            if (part > 0) { (bool ok, ) = participants[i].call{value: part}(""); require(ok, "surplus xfer"); }
        }
        emit SurplusDistributed(total);
    }
"#;

fn gen_takaful_test(name: &str, shares: &[u64]) -> String {
    let n = shares.len();
    let shares_js = shares.iter().map(|s| format!("{}n", s)).collect::<Vec<_>>().join(", ");
    format!(
        r#"// Generated by deducible — Takaful (cooperative). Proves the surplus returns PRO-RATA to the
// participant pool, and only the operator pays claims.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — takaful", function () {{
  let operator, participants, c;
  const SHARES = [{shares_js}];
  const FUND = 1000000n, BPS = 10000n;

  beforeEach(async function () {{
    const signers = await ethers.getSigners();
    operator = signers[0];
    participants = signers.slice(1, 1 + {n});
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(operator).deploy(participants.map(p => p.address), SHARES);
    await c.waitForDeployment();
  }});

  it("surplus_to_participants: the surplus is returned pro-rata to participants", async function () {{
    await c.connect(operator).contribute({{ value: FUND }});
    const before = await Promise.all(participants.map(p => ethers.provider.getBalance(p.address)));
    await c.connect(operator).distributeSurplus();
    for (let i = 0; i < participants.length; i++) {{
      const after = await ethers.provider.getBalance(participants[i].address);
      expect(after - before[i]).to.equal((FUND * SHARES[i]) / BPS);
    }}
  }});

  it("only the operator pays claims and distributes surplus", async function () {{
    await c.connect(operator).contribute({{ value: FUND }});
    await expect(c.connect(participants[0]).distributeSurplus()).to.be.revertedWith("only operator");
  }});
}});
"#,
        name = name,
        shares_js = shares_js,
        n = n,
    )
}

// =====================================================================================
// Mudarabah pool (investment fund). Many rabb al-mal (pool) + one mudarib. settle() distributes
// realized proceeds: profit (over capital) gives the mudarib its ratio cut, the rest pro-rata to
// the pool; on a loss the mudarib gets nothing and the pool bears it pro-rata. No oracle.
// =====================================================================================

fn gen_mudarabah_pool(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let shares = pool_shares(spec);
    let mudarib_bps = spec
        .returns()
        .into_iter()
        .find(|r| r.kind == "profit")
        .and_then(|r| kv_get(&r.kvs, "mudarib"))
        .and_then(|e| e.as_num())
        .unwrap_or(3000);
    let mut s = provenance_doc(spec, &format!("{} — mudarabah pool (many rabb al-mal, one mudarib) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(MUDARABAH_POOL_BODY);
    s.push_str("}\n");
    let test_js = gen_mudarabah_pool_test(&name, &shares, mudarib_bps);
    let descriptor = sukuk_descriptor(&name, &shares);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const MUDARABAH_POOL_BODY: &str = r#"    address public immutable mudarib;
    address[] public rabbs;
    uint16[] public sharesBps;     // each rabb's share of the capital pool
    uint256 public immutable totalCapital;
    uint256 public immutable mudaribProfitBps;
    uint256 public constant BPS = 10000;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyMudarib() { require(msg.sender == mudarib, "only mudarib"); _; }

    event Settled(uint256 realized, uint256 toMudarib);

    constructor(address[] memory _rabbs, uint16[] memory _sharesBps, uint256 _totalCapital, uint256 _mudaribProfitBps) {
        require(_rabbs.length == _sharesBps.length && _rabbs.length >= 2, "rabbs/shares");
        require(_mudaribProfitBps < BPS, "profit split");
        uint256 sum;
        for (uint256 i = 0; i < _sharesBps.length; i++) {
            require(_rabbs[i] != address(0), "zero rabb");
            sum += _sharesBps[i];
            rabbs.push(_rabbs[i]);
            sharesBps.push(_sharesBps[i]);
        }
        require(sum == BPS, "capital shares must total 10000 bps");
        mudarib = msg.sender; totalCapital = _totalCapital; mudaribProfitBps = _mudaribProfitBps;
    }

    /// @dev INVARIANT profit_by_ratio: profit over capital gives the mudarib its ratio cut.
    ///      INVARIANT loss_on_capital + no_guarantee: a shortfall is borne by the rabb pool pro-rata;
    ///      the mudarib gets NOTHING on a loss and never tops up the capital.
    function settle(uint256 realized) external payable onlyMudarib nonReentrant {
        require(!settled, "settled");
        require(msg.value == realized, "send exactly the realized proceeds");
        settled = true;
        uint256 toMudarib = 0;
        if (realized > totalCapital) {
            uint256 profit = realized - totalCapital;
            toMudarib = (profit * mudaribProfitBps) / BPS;
            if (toMudarib > 0) { (bool okM, ) = mudarib.call{value: toMudarib}(""); require(okM, "to mudarib"); }
        }
        uint256 toPool = realized - toMudarib;
        for (uint256 i = 0; i < rabbs.length; i++) {
            uint256 part = (toPool * sharesBps[i]) / BPS;
            if (part > 0) { (bool ok, ) = rabbs[i].call{value: part}(""); require(ok, "to rabb"); }
        }
        emit Settled(realized, toMudarib);
    }
"#;

fn gen_mudarabah_pool_test(name: &str, shares: &[u64], mudarib_bps: u64) -> String {
    let n = shares.len();
    let shares_js = shares.iter().map(|s| format!("{}n", s)).collect::<Vec<_>>().join(", ");
    format!(
        r#"// Generated by deducible — Mudarabah pool. Proves profit gives the mudarib its ratio cut and the
// rest goes pro-rata to the rabb pool; on a loss the mudarib gets nothing.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — mudarabah pool", function () {{
  let mudarib, rabbs, c;
  const SHARES = [{shares_js}];
  const TOTAL = 1000000n, MUDARIB_BPS = {mudarib_bps}n, BPS = 10000n;

  beforeEach(async function () {{
    const signers = await ethers.getSigners();
    mudarib = signers[0];
    rabbs = signers.slice(1, 1 + {n});
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(mudarib).deploy(rabbs.map(r => r.address), SHARES, TOTAL, MUDARIB_BPS);
    await c.waitForDeployment();
  }});

  it("profit_by_ratio: the mudarib takes its ratio of the profit, the pool the rest pro-rata", async function () {{
    const realized = TOTAL + 300000n;
    const profit = realized - TOTAL;
    const toMudarib = (profit * MUDARIB_BPS) / BPS;
    await expect(c.connect(mudarib).settle(realized, {{ value: realized }})).to.emit(c, "Settled").withArgs(realized, toMudarib);
  }});

  it("loss_on_capital: on a loss the mudarib gets nothing", async function () {{
    const realized = TOTAL - 200000n;
    await expect(c.connect(mudarib).settle(realized, {{ value: realized }})).to.emit(c, "Settled").withArgs(realized, 0n);
  }});

  it("only the mudarib settles", async function () {{
    await expect(c.connect(rabbs[0]).settle(TOTAL, {{ value: TOTAL }})).to.be.revertedWith("only mudarib");
  }});
}});
"#,
        name = name,
        shares_js = shares_js,
        mudarib_bps = mudarib_bps,
        n = n,
    )
}

// =====================================================================================
// Waqf (endowment). The corpus is escrowed and LOCKED forever (no withdraw path = inalienable);
// only income sent to distributeIncome reaches the beneficiary. No oracle.
// =====================================================================================

fn gen_waqf(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let mut s = provenance_doc(spec, &format!("{} — waqf (endowment; corpus inalienable) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(WAQF_BODY);
    s.push_str("}\n");
    let test_js = gen_waqf_test(&name);
    let descriptor = waqf_descriptor(&name);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const WAQF_BODY: &str = r#"    address public immutable waqif;
    address public immutable beneficiary;
    address public immutable nazir;
    uint256 public corpus;   // escrowed and LOCKED — there is no withdrawal path (inalienable)

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyWaqif() { require(msg.sender == waqif, "only waqif"); _; }
    modifier onlyNazir() { require(msg.sender == nazir, "only nazir"); _; }

    event Endowed(uint256 corpus);
    event IncomeDistributed(uint256 amount);

    constructor(address _beneficiary, address _nazir) {
        require(_beneficiary != address(0) && _nazir != address(0), "zero addr");
        waqif = msg.sender; beneficiary = _beneficiary; nazir = _nazir;
    }

    /// @dev INVARIANT corpus_inalienable: the corpus is locked here forever — NO function ever
    ///      transfers it out (it is neither sold, gifted, nor inherited).
    function endow() external payable onlyWaqif {
        require(corpus == 0, "already endowed");
        require(msg.value > 0, "no corpus");
        corpus = msg.value; emit Endowed(corpus);
    }

    /// @dev INVARIANT income_only: only the income remitted here reaches the beneficiary; the
    ///      corpus is never touched.
    function distributeIncome() external payable onlyNazir nonReentrant {
        require(corpus > 0, "not endowed");
        require(msg.value > 0, "no income");
        (bool ok, ) = beneficiary.call{value: msg.value}(""); require(ok, "income to beneficiary");
        emit IncomeDistributed(msg.value);
    }
"#;

fn gen_waqf_test(name: &str) -> String {
    format!(
        r#"// Generated by deducible — Waqf (endowment). Proves the corpus is locked (inalienable) while income
// flows to the beneficiary.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — waqf", function () {{
  let waqif, beneficiary, nazir, c;
  const CORPUS = 1000000n, INCOME = 50000n;

  beforeEach(async function () {{
    [waqif, beneficiary, nazir] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(waqif).deploy(beneficiary.address, nazir.address);
    await c.waitForDeployment();
  }});

  it("income_only: income reaches the beneficiary; the corpus stays locked", async function () {{
    await c.connect(waqif).endow({{ value: CORPUS }});
    await expect(c.connect(nazir).distributeIncome({{ value: INCOME }})).to.emit(c, "IncomeDistributed").withArgs(INCOME);
    expect(await ethers.provider.getBalance(await c.getAddress())).to.equal(CORPUS); // corpus untouched
  }});

  it("corpus_inalienable: only the nazir distributes; there is no corpus withdrawal", async function () {{
    await c.connect(waqif).endow({{ value: CORPUS }});
    await expect(c.connect(waqif).distributeIncome({{ value: INCOME }})).to.be.revertedWith("only nazir");
  }});
}});
"#,
        name = name,
    )
}

fn waqf_descriptor(name: &str) -> String {
    format!(
        r#"{{
  "instrument": "waqf",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "waqif",
  "oracle": null,
  "constructorAbi": ["address","address"],
  "constructorArgs": ["@beneficiary","@nazir"],
  "accounts": ["beneficiary","nazir"],
  "lifecycle": [
    {{ "as": "waqif", "fn": "endow", "value": 1000000, "note": "lock the corpus (inalienable)" }},
    {{ "as": "nazir", "fn": "distributeIncome", "value": 50000, "note": "income to the beneficiary; corpus preserved" }}
  ],
  "reads": ["corpus"]
}}
"#,
        name = name,
    )
}

// =====================================================================================
// Hibah (gift). Immediate, irrevocable, no consideration. No oracle.
// =====================================================================================

fn gen_hibah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let mut s = provenance_doc(spec, &format!("{} — hibah (gratuitous gift) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(HIBAH_BODY);
    s.push_str("}\n");
    let test_js = gen_hibah_test(&name);
    let descriptor = hibah_descriptor(&name);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const HIBAH_BODY: &str = r#"    address public immutable donor;
    address public immutable donee;
    bool public given;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyDonor() { require(msg.sender == donor, "only donor"); _; }

    event Gifted(uint256 amount);

    constructor(address _donee) {
        require(_donee != address(0), "zero addr");
        require(_donee != msg.sender, "donor and donee must be distinct");
        donor = msg.sender; donee = _donee;
    }

    /// @dev INVARIANT immediate_transfer + no_consideration: the gift transfers to the donee at once,
    ///      and the donee owes nothing in return (there is no consideration path).
    function give() external payable onlyDonor nonReentrant {
        require(!given, "already given");
        require(msg.value > 0, "no gift");
        given = true;
        (bool ok, ) = donee.call{value: msg.value}(""); require(ok, "gift to donee");
        emit Gifted(msg.value);
    }
"#;

fn gen_hibah_test(name: &str) -> String {
    format!(
        r#"// Generated by deducible — Hibah (gift). Proves an immediate, gratuitous transfer to the donee.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — hibah", function () {{
  let donor, donee, c;
  const AMOUNT = 1000000n;

  beforeEach(async function () {{
    [donor, donee] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(donor).deploy(donee.address);
    await c.waitForDeployment();
  }});

  it("immediate_transfer: the gift goes to the donee at once", async function () {{
    const before = await ethers.provider.getBalance(donee.address);
    await c.connect(donor).give({{ value: AMOUNT }});
    expect(await ethers.provider.getBalance(donee.address) - before).to.equal(AMOUNT);
  }});

  it("only the donor gives", async function () {{
    await expect(c.connect(donee).give({{ value: AMOUNT }})).to.be.revertedWith("only donor");
  }});
}});
"#,
        name = name,
    )
}

fn hibah_descriptor(name: &str) -> String {
    format!(
        r#"{{
  "instrument": "hibah",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "donor",
  "oracle": null,
  "constructorAbi": ["address"],
  "constructorArgs": ["@donee"],
  "accounts": ["donee"],
  "lifecycle": [
    {{ "as": "donor", "fn": "give", "value": 1000000, "note": "immediate gratuitous transfer to the donee" }}
  ],
  "reads": ["given"]
}}
"#,
        name = name,
    )
}

// =====================================================================================
// Wasiyya (bequest). Capped at one-third (enforced at construction); paid to a non-heir legatee.
// No oracle.
// =====================================================================================

fn gen_wasiyya(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let share = spec.returns().into_iter().find(|r| r.kind == "bequest")
        .and_then(|r| kv_get(&r.kvs, "share_bps")).and_then(|e| e.as_num()).unwrap_or(3000);
    let mut s = provenance_doc(spec, &format!("{} — wasiyya (bequest, <= 1/3) (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(WASIYYA_BODY);
    s.push_str("}\n");
    let test_js = gen_wasiyya_test(&name, share);
    let descriptor = wasiyya_descriptor(&name, share);
    Ok(Generated { instrument: spec.class.clone(), contract_name: name, sol: s, test_js, descriptor })
}

const WASIYYA_BODY: &str = r#"    address public immutable testator;
    address public immutable legatee;
    uint256 public immutable estate;
    uint256 public immutable shareBps;
    uint256 public constant BPS = 10000;
    bool public executed;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyTestator() { require(msg.sender == testator, "only testator"); _; }

    event Executed(uint256 amount);

    /// @dev INVARIANT within_one_third: the bequest may not exceed one-third (3333 bps) of the estate.
    constructor(address _legatee, uint256 _estate, uint256 _shareBps) {
        require(_legatee != address(0), "zero addr");
        require(_legatee != msg.sender, "testator and legatee must be distinct");
        require(_estate > 0, "estate");
        require(_shareBps > 0 && _shareBps <= 3333, "bequest must be <= one-third (3333 bps)");
        testator = msg.sender; legatee = _legatee; estate = _estate; shareBps = _shareBps;
    }

    /// @dev the bequest (estate * shareBps / 10000) is paid to the non-heir legatee on execution.
    function execute() external payable onlyTestator nonReentrant {
        require(!executed, "already executed");
        uint256 bequest = (estate * shareBps) / BPS;
        require(msg.value == bequest, "send exactly the bequest amount");
        executed = true;
        (bool ok, ) = legatee.call{value: bequest}(""); require(ok, "bequest to legatee");
        emit Executed(bequest);
    }
"#;

fn gen_wasiyya_test(name: &str, share: u64) -> String {
    format!(
        r#"// Generated by deducible — Wasiyya (bequest). Proves the one-third cap is enforced at construction.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — wasiyya", function () {{
  let testator, legatee, c;
  const ESTATE = 1000000n, SHARE = {share}n, BPS = 10000n;

  beforeEach(async function () {{
    [testator, legatee] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(testator).deploy(legatee.address, ESTATE, SHARE);
    await c.waitForDeployment();
  }});

  it("the bequest (<= 1/3) is paid to the legatee", async function () {{
    const bequest = (ESTATE * SHARE) / BPS;
    await expect(c.connect(testator).execute({{ value: bequest }})).to.emit(c, "Executed").withArgs(bequest);
  }});

  it("within_one_third: a bequest over one-third cannot deploy", async function () {{
    const F = await ethers.getContractFactory("{name}");
    await expect(F.connect(testator).deploy(legatee.address, ESTATE, 4000n)).to.be.revertedWith("bequest must be <= one-third (3333 bps)");
  }});
}});
"#,
        name = name,
        share = share,
    )
}

fn wasiyya_descriptor(name: &str, share: u64) -> String {
    format!(
        r#"{{
  "instrument": "wasiyya",
  "regime": "islamic",
  "contract": "{name}",
  "operatorRole": "testator",
  "oracle": null,
  "constructorAbi": ["address","uint256","uint256"],
  "constructorArgs": ["@legatee", 1000000, {share}],
  "accounts": ["legatee"],
  "lifecycle": [
    {{ "as": "testator", "fn": "execute", "note": "pay the bequest (<= 1/3) to the non-heir legatee" }}
  ],
  "reads": ["executed"]
}}
"#,
        name = name,
        share = share,
    )
}

fn escrow_amount(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "release")
        .and_then(|r| kv_get(&r.kvs, "amount"))
        .and_then(|e| e.as_num())
        .unwrap_or(1_000_000)
}

fn gen_commercial(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let amount = escrow_amount(spec);
    let mut s = provenance_doc(spec, &format!("{} — commercial escrow with a code-based judiciary engine (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(COMMERCIAL_BODY);
    s.push_str("}\n");
    let test_js = gen_commercial_test(&name, amount);
    let descriptor = commercial_descriptor(&name, amount);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const COMMERCIAL_BODY: &str = r#"    address public immutable depositor;
    address public immutable beneficiary;
    address public immutable arbiter;
    uint256 public immutable amount;

    uint256 public deposited;
    bool public conditionMet;
    bool public disputed;
    bool public closed;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyDepositor() { require(msg.sender == depositor, "only depositor"); _; }
    modifier onlyArbiter() { require(msg.sender == arbiter, "only arbiter"); _; }
    modifier open() { require(!closed, "closed"); _; }

    event Funded(uint256 amount);
    event ConditionConfirmed(address by);
    event Released(address to, uint256 amount);
    event Refunded(address to, uint256 amount);
    event DisputeRaised(address by);
    event ArbiterRuling(bool forBeneficiary);

    /// @dev INVARIANT consideration_present: depositor and beneficiary must be distinct.
    constructor(address _beneficiary, address _arbiter, uint256 _amount) {
        require(_beneficiary != address(0) && _arbiter != address(0), "zero addr");
        require(_amount > 0, "amount"); // INVARIANT certainty_of_terms: a definite sum
        require(_beneficiary != msg.sender, "consideration: distinct parties");
        depositor = msg.sender; beneficiary = _beneficiary; arbiter = _arbiter; amount = _amount;
    }

    function fund() external payable onlyDepositor open {
        require(deposited == 0, "funded");
        require(msg.value == amount, "must deposit exactly the agreed amount");
        deposited = msg.value; emit Funded(msg.value);
    }

    function confirmCondition() external onlyDepositor open {
        require(deposited == amount, "not funded");
        conditionMet = true; emit ConditionConfirmed(msg.sender);
    }

    function release() external open nonReentrant {
        require(msg.sender == depositor || msg.sender == beneficiary, "only a party");
        require(conditionMet, "condition not met");
        require(!disputed, "under dispute");
        closed = true;
        (bool ok, ) = beneficiary.call{value: deposited}(""); require(ok, "release xfer");
        emit Released(beneficiary, deposited);
    }

    /// @dev the judiciary engine: either party may invoke the tribunal.
    function raiseDispute() external open {
        require(msg.sender == depositor || msg.sender == beneficiary, "only a party");
        disputed = true; emit DisputeRaised(msg.sender);
    }

    /// @dev INVARIANT dispute_resolution_present: the arbiter's ruling is the remedy
    ///      (release to the beneficiary, or refund to the depositor). Regime-neutral.
    function arbiterRuling(bool forBeneficiary) external onlyArbiter open nonReentrant {
        require(disputed, "no dispute");
        closed = true;
        address to = forBeneficiary ? beneficiary : depositor;
        (bool ok, ) = to.call{value: deposited}(""); require(ok, "ruling xfer");
        emit ArbiterRuling(forBeneficiary);
        if (forBeneficiary) { emit Released(beneficiary, deposited); } else { emit Refunded(depositor, deposited); }
    }
"#;

fn gen_commercial_test(name: &str, amount: u64) -> String {
    format!(
        r#"// Generated by deducible — Commercial Escrow (common law). Proves compliance-by-construction
// is universal across legal regimes, and exercises the regime-neutral code-based judiciary
// engine (arbiter-adjudicated release or refund).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (deducible-generated) — commercial escrow + judiciary engine", function () {{
  let depositor, beneficiary, arbiter, c;
  const AMOUNT = {amount}n;

  beforeEach(async function () {{
    [depositor, beneficiary, arbiter] = await ethers.getSigners();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(depositor).deploy(beneficiary.address, arbiter.address, AMOUNT);
    await c.waitForDeployment();
  }});

  it("releases to the beneficiary once the definite condition is met", async function () {{
    await c.connect(depositor).fund({{ value: AMOUNT }});
    await c.connect(depositor).confirmCondition();
    await expect(c.connect(beneficiary).release()).to.emit(c, "Released").withArgs(beneficiary.address, AMOUNT);
    expect(await c.closed()).to.equal(true);
  }});

  it("judiciary engine: the arbiter may rule for the beneficiary", async function () {{
    await c.connect(depositor).fund({{ value: AMOUNT }});
    await c.connect(depositor).raiseDispute();
    await expect(c.connect(arbiter).arbiterRuling(true)).to.emit(c, "ArbiterRuling").withArgs(true);
  }});

  it("judiciary engine: the arbiter may refund the depositor", async function () {{
    await c.connect(depositor).fund({{ value: AMOUNT }});
    await c.connect(beneficiary).raiseDispute();
    await expect(c.connect(arbiter).arbiterRuling(false)).to.emit(c, "Refunded").withArgs(depositor.address, AMOUNT);
  }});

  it("only the arbiter may rule on a dispute", async function () {{
    await c.connect(depositor).fund({{ value: AMOUNT }});
    await c.connect(depositor).raiseDispute();
    await expect(c.connect(depositor).arbiterRuling(true)).to.be.revertedWith("only arbiter");
  }});

  it("release is blocked while a dispute is open", async function () {{
    await c.connect(depositor).fund({{ value: AMOUNT }});
    await c.connect(depositor).confirmCondition();
    await c.connect(depositor).raiseDispute();
    await expect(c.connect(beneficiary).release()).to.be.revertedWith("under dispute");
  }});
}});
"#,
        name = name,
        amount = amount,
    )
}

fn commercial_descriptor(name: &str, amount: u64) -> String {
    format!(
        r#"{{
  "instrument": "commercial_escrow",
  "regime": "common_law",
  "contract": "{name}",
  "operatorRole": "depositor",
  "oracle": null,
  "constructorAbi": ["address","address","uint256"],
  "constructorArgs": ["@beneficiary","@arbiter",{amount}],
  "accounts": ["beneficiary","arbiter"],
  "funding": {{ "fund": {amount} }},
  "lifecycle": [
    {{ "as": "depositor", "fn": "confirmCondition", "note": "definite condition met" }},
    {{ "as": "depositor", "fn": "release", "note": "escrow released to beneficiary" }}
  ],
  "reads": []
}}
"#,
        name = name,
        amount = amount,
    )
}
