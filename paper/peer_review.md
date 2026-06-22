# Peer-Review Report — "Compliance by Construction"

Simulated 5-reviewer panel (full mode). Read-only: this report does not modify the manuscript.

## Phase 0 — Field analysis & panel configuration

- **Primary field:** Islamic finance / *fiqh al-mu'amalat* applied to financial technology.
- **Secondary field:** blockchain systems / smart-contract engineering (design science).
- **Paradigm:** Design Science Research (artifact + evaluation).
- **Maturity:** early-stage proof-of-concept with a real on-chain demonstration.
- **Plausible venue tier:** mid-tier interdisciplinary fintech/Islamic-finance journal; a top-tier blockchain-systems venue would demand far stronger evaluation.

Panel:
- **EIC** — interdisciplinary fintech editor; judges fit, novelty, significance.
- **R1 Methodology** — empirical software-engineering / smart-contract security.
- **R2 Domain** — Islamic finance scholar with *fiqh al-mu'amalat* grounding.
- **R3 Perspective** — RegTech / law-and-economics of asset finance.
- **DA** — Devil's Advocate against the central claim.

---

## Phase 1 — Independent reviews

### R0 — Editor-in-Chief

**Summary.** A genuinely fresh thesis — recasting *Shari'ah* compliance from ex-post attestation to ex-ante protocol enforcement ("compliance by construction"), demonstrated with a real, deployed *Musharakah Mutanaqisah* and measured cost. The framing inversion (the *Shari'ah* economy as the proposal, blockchain as the substrate) is rhetorically strong and intellectually serious. The writing is well above average.

**Concerns (fit & apparatus).**
1. **No Abstract, no keywords, no References section, no figure call-outs in text.** As submitted this is not a complete manuscript. (Loc: whole document.) *Fix:* add structured abstract (+ Arabic/bilingual if venue expects), 5–7 keywords, a real reference list, and reference Figures 1–2 from §4.
2. **Title over-promises.** "Enforcing Shari'ah Substance" asserts a contested *fiqh* outcome as achieved. *Fix:* soften to "toward enforcing…" or scope to "enforcing the contractual conditions of…".
3. **Significance vs. evidence mismatch.** The claims are field-level ("the Shari'ah economy is the revolutionary proposal"); the evidence is one toy lifecycle. *Fix:* either narrow the claims or strengthen the evaluation (see R1, DA).

**Scores (/100):** Originality 82 · Rigor 42 · Evidence 45 · Coherence 64 · Writing 70.

---

### R1 — Methodology Reviewer (smart-contract / DSR rigor)

**Strengths.** Artifact is real and reproducible: contracts, tests, deploy/interaction scripts, recorded on-chain IDs and a machine-readable run. The honest §4.4 (units bug → finding) is good scientific conduct.

**Major issues.**
1. **The evaluation is N=1 and happy-path.** A single lifecycle execution is a *demonstration*, not the rigorous evaluation DSR requires (Hevner's evaluate). No comparison baseline (a conventional MMP, or an existing "smart sukuk"), no stress/adversarial runs, no negative-path on-chain tests (only the 5 local unit tests). (Loc: §3.4, §4.1–4.3.) *Fix:* add adversarial on-chain cases, multiple parameterizations, and a comparison.
2. **The independence properties were never actually exercised.** §4.2 and the artifact run bank, client, **and** valuer from one operator account. Invariant I4 (no partner self-reports; independent valuer) is therefore *untested in the demonstration* — the very anti-self-dealing claim is asserted, not shown. (Loc: §3.5 "operator account", §4.2.) *Fix:* re-run with three distinct funded accounts; show a partner-submitted valuation reverting on-chain.
3. **No security audit.** `payRent`/`buyShare` perform external value transfers via low-level `.call` with no reentrancy guard and no test for a failing `bank.call`. For a contract custodying value this is disqualifying without a Slither/Mythril pass + reentrancy tests. (Loc: contract methods.) *Fix:* add static analysis, reentrancy guards/tests, and report results.
4. **Loss-sharing is emitted, not settled (see DA — this is also a rigor failure).** `syncValuation` computes write-downs and emits an event but moves no funds and adjusts no claimable balance. The evaluation cannot support "the financier cannot exit whole" because no settlement path exists or is tested. (Loc: §4.2 second transition; §4.5.) *Fix:* implement capital custody + dissolution settlement, then test that the financier's recoverable amount actually falls.

**Scores:** Originality 75 · Rigor 38 · Evidence 40 · Coherence 60 · Writing 68.

---

### R2 — Domain Reviewer (Islamic finance / fiqh)

**Strengths.** Correctly identifies the form-over-substance and PLS-underuse fault lines; chooses *Musharakah Mutanaqisah* well as the contested instrument; treats *gharar* and the oracle honestly.

**Major issues.**
1. **Every citation is a placeholder.** `[verify]` on al-Baqarah 2:275, al-Hashr 59:7, Sahih Muslim, AAOIFI No. 12, OIC Res. 179, El-Gamal, Usmani, Hevner, Peffers. A *fiqh* paper cannot assert rulings without verified primary sources; mis-attributing to scripture is a grave error. (Loc: throughout.) *Fix:* verify each against primary sources; confirm whether diminishing *musharakah* falls under AAOIFI SS No. 12 or another standard.
2. **Compliance is asserted, not adjudicated.** The paper repeatedly implies the artifact *is* Shari'ah-compliant. Compliance is a scholarly ruling; an engineering artifact cannot self-certify. (Loc: title, §3.3, §5.) *Fix:* reframe as "encodes conditions that a Shari'ah board could audit," and state explicitly that no fatwa is claimed; ideally obtain a scholar's review.
3. **MMP's own contestation is omitted.** Some scholars regard diminishing *musharakah* (especially with binding purchase undertakings and fixed rentals) as itself near-debt. Engaging only the *defense* of MMP is one-sided. (Loc: §2.) *Fix:* present the *khilaf* (scholarly disagreement) on MMP itself.
4. **The literature review names no actual works on blockchain + Islamic finance.** (Loc: §2 "Blockchain and Islamic finance".) *Fix:* engage real prior art (e.g., smart-sukuk efforts, existing tokenized-waqf and Islamic-DeFi scholarship).

**Scores:** Originality 78 · Rigor 45 · Evidence 40 · Coherence 66 · Writing 72.

---

### R3 — Perspective Reviewer (RegTech / law & economics)

**Strengths.** The "Shari'ah board as protocol auditor" reframing is a real contribution for standard-setters; the cost framing is a useful opening.

**Major issues.**
1. **The valuation cost — the dominant real cost — is excluded.** §4.3 counts only on-chain gas. For real assets, the *valuer* (periodic professional appraisal) dominates total cost and is the actual oracle. Claiming "industrial viability" while omitting it is misleading. (Loc: §4.3.) *Fix:* model end-to-end cost including valuation/oracle operations and their cadence.
2. **On-chain "ownership" has no legal force.** The contract tracks `bps`, not title; real property ownership lives in an off-chain registry. Without a legal bridge, the enforced state is a private ledger, not ownership. (Loc: §3.3, §5.) *Fix:* discuss the legal-recognition gap and any registry-integration path; temper the enforcement claim accordingly.
3. **No real-asset binding.** There is no asset token, escrow, or link to a real underlying; "co-ownership" is an integer. The form-over-substance cure thus depends entirely on the very off-chain link the paper flags as the limitation. (Loc: §3.3.) *Fix:* show how a tokenized real asset (or registry hook) would attach; acknowledge this is unbuilt.
4. **Generalization overreach.** One instrument cannot carry claims about "the Shari'ah economy." (Loc: §1, §6.) *Fix:* scope claims to MMP; mark the rest as a research programme.

**Scores:** Originality 80 · Rigor 44 · Evidence 46 · Coherence 62 · Writing 70.

---

### DA — Devil's Advocate

**Strongest counter-argument.** The paper's central contribution — moving compliance from *attestation* to *enforcement* — is, at this stage, largely illusory, for three compounding reasons. (1) Every binding compliance fact (that a real asset exists and is co-owned; its fair value; that a loss occurred) is **off-chain** and enters only through a trusted valuer/oracle. The contract therefore enforces *arithmetic consistency among numbers a trusted party supplies*, not Shari'ah substance — trust was relocated into Solidity, not removed. (2) The flagship risk-sharing property is **not economically enforced**: `syncValuation` *emits* a `LossShared` event but custodies no capital and executes no settlement, so "the financier cannot exit whole" is unsupported at the level of money — it is a printed number. (3) The on-chain demonstration ran bank, client, and valuer from **one account**, so the anti-self-dealing independence the paper sells was never exercised adversarially. Net: the artifact demonstrates an internally consistent bookkeeping contract; it does not yet demonstrate *enforced Shari'ah substance*. "Compliance by construction" is, today, compliance-by-assertion re-expressed in code.

**Issue list.**
- **CRITICAL** — Loss-sharing emitted, not settled; no capital custody → core claim unsupported. (§4.2, §4.5, contract `syncValuation`.)
- **CRITICAL** — Independence/anti-self-dealing untested (single account for all roles). (§3.5, §4.2.)
- **MAJOR** — Enforcement reduces to trusting the oracle for all real-world facts; the "enforcement" surface is internal arithmetic. (§3.3, §5.)
- **MAJOR** — Real-asset binding absent; ownership is an integer with no legal/title link. (§3.3.)
- **MINOR** — "Revolutionary proposal" rhetoric outruns a single PoC. (§1, §6.)

**Ignored alternatives.** That a well-designed *off-chain* registry + audit could achieve the same continuous verification without a ledger; the paper does not argue why the chain is necessary rather than convenient.

**Missing stakeholders.** The actual valuer/appraiser (cost, liability, capture); the client's legal recourse if on-chain and registry state diverge.

**"So what?" test.** Survives *as a vision and a credible first artifact*; does **not** yet survive *as a demonstration of enforced compliance*.

---

## Phase 2 — Editorial decision & revision roadmap

**Decision: MAJOR REVISION.** (Per IRON RULE #4, the DA CRITICAL findings preclude Accept. The contribution is novel and the artifact real, so the work is salvageable rather than reject — but a top-tier systems venue would likely reject pending the capital-settlement and independence fixes.)

**Consensus across ≥4 reviewers:** (i) central claim outruns the evidence; (ii) citations must be made real; (iii) claims must be scoped from "the Shari'ah economy" to "MMP".

**Gating items (must fix to clear the CRITICALs):**
1. **Implement capital custody + settlement** so a downward `syncValuation` actually reduces the financier's recoverable capital; test that on-chain. (R1, DA)
2. **Re-run with three distinct accounts** (bank, client, independent valuer); show a partner-submitted valuation reverting and an independent valuation accepted on-chain. (R1, DA)
3. **Reframe the compliance claim** — encode *conditions a board audits*, claim no fatwa; verify every `[verify]` citation; present the *khilaf* on MMP. (R2)
4. **Security pass** — static analysis + reentrancy guards/tests for the value-transferring methods. (R1)

**High-priority (strengthen significance):**
5. End-to-end cost including valuation cadence, not gas alone. (R3)
6. Discuss the legal-title / registry gap and real-asset binding. (R3)
7. Add Abstract, keywords, References, and Figure 1–2 call-outs; engage real blockchain-Islamic-finance prior art. (EIC, R2)

**Lower-priority:** temper "revolutionary" rhetoric; argue why a ledger is *necessary*, not merely convenient (DA).

**Overall weighted score:** ~57/100 (Originality 80 · Rigor 42 · Evidence 44 · Coherence 63 · Writing 70). Verdict: a strong idea and an honest first artifact, one substantial engineering revision away from supporting its own central claim.
