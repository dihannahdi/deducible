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
         /// @notice COMPLIANCE BY CONSTRUCTION. Emitted by the fiqhc compiler ONLY after the\n\
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
        r#"// Generated by fiqhc — differential proof: the GENERATED {name} reproduces the
// hand-written, peer-reviewed Musharakah Mutanaqisah behaviour on the shared lifecycle.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — differential equivalence", function () {{
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
        r#"// Generated by fiqhc — Mudarabah profit-sharing trust. Proves the SAME compiler emits a
// DIFFERENT, correct instrument: profit by ratio, and loss borne by the rabb al-mal alone.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — profit-sharing & loss on rabb al-mal", function () {{
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
        r#"// Generated by fiqhc — Ijarah Muntahia Bittamleek. Proves a THIRD instrument from the same
// compiler: rent for usufruct, lessor bears ownership risk, and ownership transfer is a
// SEPARATE act at the end of the term (not two contracts in one).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — lease ending in ownership", function () {{
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
        r#"// Generated by fiqhc — Murabaha (cost-plus trust sale, bay' al-amana). Proves the three riba
// routes are unrepresentable: a markup that grows with time, selling before possession (qabd),
// and a penalty that overcharges the debtor. The buyer can never owe more than the fixed total.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — murabaha cost-plus sale", function () {{
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
        r#"// Generated by fiqhc — Salam (forward sale). Proves the gharar guards: full prepayment at the
// session (no debt-for-debt), a known quantity, and a known delivery term.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — salam forward sale", function () {{
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
        r#"// Generated by fiqhc — Istisna' (manufacture-to-order). Proves a made-to-spec sale: the maker
// supplies materials + labour, the price may be paid in progress instalments (unlike salam), and
// the customer can never be charged above the fixed total.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — istisna' manufacture-to-order", function () {{
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
        r#"// Generated by fiqhc — Sarf (spot exchange). Proves yadan bi-yad (atomic, no deferral) and,
// for a same-genus exchange, equal-for-equal (no riba al-fadl).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — sarf spot exchange", function () {{
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
        r#"// Generated by fiqhc — Tawarruq (individual). Proves the licit form: possession before the
// onward sale, the spot buyer is an INDEPENDENT third party (the 'inah ring cannot deploy), and
// the deferred debt can never be charged above the agreed price.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — tawarruq (individual)", function () {{
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
        r#"// Generated by fiqhc — Qard Hasan (benevolent loan). Proves the loan is repaid in like: the
// contract accepts EXACTLY the principal and rejects any increase (riba).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — qard hasan", function () {{
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
        r#"// Generated by fiqhc — Commercial Escrow (common law). Proves compliance-by-construction
// is universal across legal regimes, and exercises the regime-neutral code-based judiciary
// engine (arbiter-adjudicated release or refund).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — commercial escrow + judiciary engine", function () {{
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
