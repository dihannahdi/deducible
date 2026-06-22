# Compliance by Construction: Enforcing the Contractual Conditions of a Shari'ah-Compliant Diminishing Partnership on a Public Ledger

**Abstract**

Contemporary Islamic finance is recurrently criticised for *form over substance*: an instrument is certified Shari'ah-compliant once, by a supervisory board, after which no party can continuously verify that its economic substance still matches its certified form. We propose **compliance by construction** — encoding the contractual conditions that distinguish a genuine Shari'ah-compliant instrument from disguised debt as invariants a smart contract enforces *at execution time*, so that a violating transaction simply does not execute. We instantiate the principle on *Musharakah Mutanaqisah* (the diminishing partnership), the instrument that sits at the intersection of the field's deepest fault lines, and develop it as a family of smart contracts on Hedera. The artifact escrows partner capital and, on dissolution, pays each partner its share of the asset's *current* value — so a loss provably reduces the financier's recoverable capital by transfer rather than by assertion; it prices buyouts from an independent valuation oracle; it represents ownership as a real on-ledger token whose units move atomically with payment; and it encodes lawful rescission — *khiyar al-shart*, *iqalah*, *khiyar al-'ayb* (adjudicated by an agreed arbiter), and an authoritative *faskh* path. We evaluate the artifact with a unit-test suite (20/20 passing), an adversarial multi-account lifecycle on the Hedera testnet, static analysis (solhint and Slither, zero security findings), and per-operation cost measurement. On-chain, after a 10% asset write-down the financier recovered 72,000,000 of the 80,000,000 tinybar it had funded — bearing its proportional loss as enforced by the protocol — while role-violating transactions reverted. We argue the deeper contribution is a reframing of Shari'ah governance: the supervisory board moves from *certifier of an instrument* to *auditor of a protocol*. We are explicit about the boundary of the claim. This is an engineering result, not a juristic one: binding real-world facts still enter through a trusted valuer, on-ledger ownership is not yet legal title, and whether the encodings satisfy the *fiqh* is a ruling reserved for qualified scholars.

**Keywords:** Islamic finance; *Musharakah Mutanaqisah*; smart contracts; design science; Shari'ah governance; risk-sharing; blockchain; tokenization.

---

## 1. Introduction

For half a century, Islamic finance has lived with a quiet contradiction. Its normative ideal is participation — the sharing of profit and of loss, the binding of finance to the real economy, and the refusal of a guaranteed return on money lent (Qur'an, al-Baqarah 2:275 [scholar-verify]). Its dominant practice, however, leans on debt-like instruments — *murabaha*, and especially organised *tawarruq* — whose economic substance reproduces the very interest-bearing loan they were meant to displace. The critique is neither marginal nor recent. The OIC International Islamic Fiqh Academy declared organised *tawarruq* impermissible (Resolution 179 (19/5), 19th session, Sharjah, April 2009); and Usmani (2007), writing from within the tradition as a senior jurist, argued that the great majority of the *sukuk* then in circulation did not, in substance, meet the requirements of the Shari'ah. The charge that recurs under many names is the same: *form over substance* — a contract certified once, in form, whose substance is free to drift thereafter.

This paper begins from an inversion. The prevailing literature on blockchain for Islamic finance casts the technology as protagonist and the Shari'ah as a use-case: it asks what a ledger can do *for* Islamic finance. We reverse the order. The Shari'ah economy — its insistence on real ownership, on shared risk, and on the just circulation of wealth "so that it does not circulate only among the rich among you" (al-Hashr 59:7 [scholar-verify]) — is the substantive proposal. A public ledger is merely the first execution substrate adequate to run that proposal *faithfully*, because it can make economic substance continuously verifiable and contractually enforced rather than periodically attested.

From this inversion we isolate a single root cause beneath the field's recurring difficulties. In contemporary practice — and, we argue, in the existing blockchain-for-Islamic-finance literature — Shari'ah compliance is an **attestation**: a board certifies an instrument, and afterward no party can continuously verify that its substance still holds. Four fault lines pressed against Islamic finance — *form over substance*, the *under-use of risk-sharing*, the tension between *immutability and the fiqh's flexibility*, and the *uncertainty (gharar)* at the boundary with the real world — are not four diseases but symptoms of one: **compliance is asserted, not enforced**.

Our research question follows directly. *Can the contractual conditions that distinguish a genuine Shari'ah-compliant partnership from disguised debt be enforced by a contract's own execution, rather than attested after the fact — and can this be done at industrially viable cost?* We answer constructively, and deliberately for a single instrument: we build a *Musharakah Mutanaqisah* (diminishing partnership) as a family of smart contracts on Hedera, encode the conditions as invariants the protocol enforces, and demonstrate the instrument through an adversarial multi-account lifecycle on a public test network. We name the principle **compliance by construction**.

This paper makes four contributions:

1. **A principle.** *Compliance by construction* — re-casting Shari'ah compliance from an ex-post human attestation into an ex-ante, protocol-enforced property of a financial instrument.
2. **An artifact.** A capital-custodying *Musharakah Mutanaqisah* contract family that enforces rent on the financier's living share, oracle-priced buyouts, proportional loss-sharing by transfer, tokenized ownership moved atomically with payment, and a suite of lawful rescission rights.
3. **Evidence.** Functional, adversarial-on-chain, cost, and static-analysis results — including a settlement in which a loss provably reduced the financier's recovery — establishing that the principle is buildable and, for the properties a protocol can verify, correct.
4. **A reframing.** Shari'ah governance reconceived as *protocol audit* rather than *instrument certification*, with the boundary of the engineering claim stated plainly.

The remainder proceeds as follows. Section 2 reviews the normative ideal and its erosion, the form-over-substance critique, prior blockchain work, and the design-science foundation. Section 3 develops the four-fault-lines analysis and the compliance-by-construction thesis. Section 4 sets out the method. Section 5 describes the artifact. Section 6 reports the evaluation. Section 7 discusses meaning and limits; Section 8 treats Shari'ah governance and the boundary of an engineering claim; Section 9 concludes.

## 2. Background and Related Work

### 2.1 The normative ideal and its erosion

The prohibition of *riba* is settled ground in Islamic commercial law (al-Baqarah 2:275 [scholar-verify]), and from it the jurists derive a preference for participatory finance — *musharakah* (partnership) and *mudarabah* (trustee finance) — over lending at interest. Yet a substantial empirical literature documents that these profit-and-loss-sharing (PLS) instruments remain marginal on the balance sheets of Islamic banks, which concentrate instead in *murabaha* and related mark-up sales. The explanation most often advanced is one of agency: under asymmetric information, the cost of verifying a partner's true profit, together with the moral hazard it invites, makes PLS expensive to police, so institutions retreat to debt-like structures whose returns are contractually fixed and cheaply monitored. The normative ideal and the operational reality thus pull in opposite directions, and have done so for decades.

### 2.2 The form-over-substance critique

A second body of work interrogates whether the prevailing debt-like instruments are Shari'ah-compliant in substance or only in form. El-Gamal (2006) analyses *Shari'ah arbitrage* and the use of *hiyal* (legal stratagems) to reproduce conventional economic outcomes within nominally compliant contracts. Usmani (2007) critiques *sukuk* structures that promise a fixed return while wearing the garment of partnership. The standard-setters respond by tightening conditions: the Accounting and Auditing Organization for Islamic Financial Institutions (AAOIFI), in its Shari'ah Standard No. 12, *Sharikah (Musharakah) and Modern Corporations*, governs the diminishing partnership and requires, among other conditions, that rent attach to a genuinely co-owned asset and that a partner's purchase undertaking be at fair or market value rather than at a pre-fixed price that would guarantee the financier's capital.

In fairness, the diminishing partnership is itself contested. Some scholars regard *Musharakah Mutanaqisah* with a binding purchase undertaking and fixed rentals as close to debt in substance, and the *khilaf* (scholarly disagreement) on this point is live rather than settled. What unites these critiques is decisive for our argument: the conditions that separate genuine partnership from disguised debt are exactly the conditions an *attestation* regime cannot *continuously* guarantee. A board can certify, at a moment, that an instrument is structured correctly; it cannot ride along inside every subsequent transaction to ensure the substance is preserved.

### 2.3 Blockchain and Islamic finance

A third, rapidly growing literature applies distributed ledgers to Islamic finance. The bulk of it treats blockchain as a *transparency* and *tokenisation* layer — "smart *sukuk*," asset provenance, and the tracking of *zakat* and *waqf* flows. This work is valuable: it improves auditability and reduces certain frictions. But, with few exceptions, it leaves the compliance *decision* precisely where it has always sat — with a human board, certifying ex post. The ledger records; it does not enforce. We find no body of work that re-casts compliance as a property *enforced by the instrument's own execution*, such that the conditions distinguishing genuine partnership from disguised debt cannot be violated without the transaction failing. That re-casting is the gap this paper addresses.

### 2.4 Design science

Methodologically, the contribution is the construction and rigorous evaluation of a novel artifact rather than the observation of an existing phenomenon. We therefore situate the work in Design Science Research (DSR), as articulated by Hevner, March, Park, and Ram (2004) and operationalised by Peffers, Tuunanen, Rothenberger, and Chatterjee (2007). DSR is the appropriate paradigm because our central claim is not a statement *about* the world but a demonstration that a *new kind of instrument can exist*: a financial contract whose compliance conditions are enforced by its own execution. We follow Hevner's three cycles — relevance (the field's unresolved tension), design (build and test the artifact), and rigor (ground the design in both *fiqh al-mu'amalat* and software-verification practice).

## 3. The Four Fault Lines and the Compliance-by-Construction Thesis

We organise the field's difficulties into four fault lines and claim a single root for all four.

- **F1 — Form over substance.** Compliance is certified once and may drift; the gap between certified form and unobserved substance is where *Shari'ah arbitrage* operates.
- **F2 — The risk-sharing paradox.** The normative ideal (PLS) is marginal because verifying and policing a partner's real position is costly; institutions default to debt-like instruments, which reintroduces F1.
- **F3 — Immutability versus flexibility.** Classical *fiqh al-mu'amalat* preserves flexibility through options to rescind (*khiyar*) and mutual cancellation (*iqalah*); a ledger's defining virtue is finality, which is in apparent tension with these.
- **F4 — The oracle / *gharar* boundary.** A contract cannot see the real world; whatever bridges the on-ledger record to off-ledger fact reintroduces both trust and potential *gharar* (excessive uncertainty), which the Shari'ah prohibits (the Prophet ﷺ forbade *bay' al-gharar*; Sahih Muslim, *Kitab al-Buyu'* [scholar-verify]).

**The thesis.** These are not independent. Their common root is that **compliance is asserted, not enforced** — it lives as a claim checked at a moment by humans, rather than as a property maintained by the instrument continuously. *Compliance by construction* attacks the root: encode each condition as an invariant the contract enforces at execution, so that the only transactions the instrument will perform are compliant ones. F1 is then met not by argument but by construction — the substance *is* the form, because the form will not execute unless the substance holds. F2 is eased because the verifiable components of a partner's position — ownership, rent, loss apportionment, settlement — become machine-checked state transitions rather than audited claims, collapsing the monitoring cost that drove institutions away from PLS. F3 is met by encoding the rescission rights themselves as contract mechanisms. F4 is the honest residue: it cannot be abolished, only relocated to an explicit, accountable boundary and minimised.

This is the paper's "single mechanism" in the spirit of a general principle rather than a one-off product: not a new Islamic-finance instrument, but a way of *holding* such instruments so that their compliance is enforced rather than promised.

## 4. Methodology

### 4.1 Design objective

Move the enforcement of a diminishing partnership's compliance conditions from ex-post human attestation to ex-ante, protocol-enforced execution, and test it on *Musharakah Mutanaqisah* — the instrument underlying much real Islamic asset and home financing, and the one sitting precisely at the intersection of F1 and F2.

### 4.2 Evaluation strategy

We evaluate the artifact at four levels of increasing externality and adversariality:

1. **Functional correctness.** A unit-test suite on a local EVM exercises each invariant, including adversarial cases (mispriced buyout, role violation, rescission after performance, terminal acquisition).
2. **On-chain demonstration.** The contracts are deployed to the Hedera testnet and driven through full lifecycles — funding, rent, buyout, revaluation, settlement, dissolution, and rescission — with state read back from the network after each step, including an *adversarial three-account* configuration in which the roles of financier, client, and valuer are held by genuinely distinct keys.
3. **Cost.** Gas consumed and fees charged are recorded per operation, to assess industrial viability.
4. **Static analysis.** The contracts are passed through solhint and Slither.

### 4.3 Implementation environment

Contracts are written in Solidity 0.8.24 and compiled with Hardhat; deployment and interaction use the Hedera JavaScript SDK (`@hashgraph/sdk`). The contracts follow checks-effects-interactions and carry `nonReentrant` guards on value-moving functions. One practical finding shaped the implementation: on this network the value visible to a contract as `msg.value` is denominated in **tinybar** (1 ℏ = 10⁸ tinybar), not in the Ethereum-style weibar; monetary quantities were denominated accordingly. The Hedera Token Service (HTS) is reached from Solidity through the system-contract precompile at address `0x167`; because that precompile executes only on Hedera, the token-bearing layer of the artifact is verified on testnet rather than on a local EVM.

## 5. The Artifact

The artifact is a family of Solidity contracts developed in five increments, each closing a specific gap; together they constitute the complete instrument.

### 5.1 Core invariants (V1)

The base contract co-owns an asset between a financier ("bank") and a client and enforces four conditions as invariants:

| # | Enforced invariant | *Fiqh* basis |
|---|--------------------|--------------|
| I1 | Rent (*ijarah*) is charged only on the financier's **current** ownership share, so it falls automatically as ownership transfers | rent must attach to a genuinely co-owned, usufruct-bearing asset (AAOIFI SS No. 12) |
| I2 | A buyout is priced from an **independent attested valuation**, never a pre-fixed schedule; the buyer cannot name their own price | prohibition of a guaranteed-capital return disguised as partnership (al-Baqarah 2:275 [scholar-verify]) |
| I3 | A fall in asset value is apportioned by the **current ownership ratio** | risk-sharing is the essence of *musharakah*; *al-ghunm bil-ghurm* and *al-kharaj bil-daman* |
| I4 | Roles are enforced; neither partner self-reports value | separation of the lease and sale undertakings |

The valuation enters through an external, access-controlled oracle interface, deliberately isolating F4 at one explicit point.

### 5.2 Capital custody and settlement (V2)

The first design recorded loss-sharing only as an emitted figure. The corrected design **escrows both partners' capital** into the contract, which then represents the asset in trust, and adds a `settle()` that, on dissolution, pays each partner its share of the oracle's *current* value from the escrowed pool. A fall in value therefore reduces the financier's transfer: loss-sharing becomes an enforced movement of money, not a printed number. This is the mechanism that gives F2 its teeth.

### 5.3 Tokenized ownership and HTS-native atomic buyout (V3)

Ownership is represented as a real HTS fungible token (a fixed supply of units equal to 100% of the asset), so a partner's stake is a transferable token with on-ledger provenance rather than a bare counter. A custodian contract demonstrates that the enforcement contract can itself hold and move these units through the HTS precompile. Building on this, an HTS-native partnership contract performs an **atomic buyout**: in a single transaction, ownership units move from the contract's escrow to the client *and* the hbar price moves to the financier. Ownership transfer and payment are thereby inseparable — there is no off-ledger interval in which one has occurred without the other.

### 5.4 Lawful rescission and authoritative *faskh* (V4–V5)

To reconcile finality with the flexibility *fiqh* preserves (F3), the contract encodes a graded suite of rescission rights:

- ***Khiyar al-shart*** — within a stipulated window after activation, either partner may unilaterally rescind (before performance), with capital refunded.
- ***Iqalah*** — mutual cancellation that completes only with the counter-party's consent.
- ***Khiyar al-'ayb*** — either partner may raise a defect; an **agreed arbiter** (a *qadi* / arbitration proxy fixed at contract creation) adjudicates it.
- **Authoritative *faskh*** — the agreed arbiter may rescind to resolve a dispute, so lawful redress is never structurally foreclosed.

The final increment also addresses the disposition of the realised-loss residue: rather than stranding the impaired remainder in the contract (which would risk *idha'at al-mal*, the wasting of wealth), it is directed to an agreed *maslahah* / *waqf* fund.

Each of these encodings is a *mechanism*; whether a given mechanism *satisfies* the *fiqh* is a question we explicitly reserve for scholars (Section 8).

## 6. Evaluation

### 6.1 Functional correctness

All twenty unit tests pass on a local EVM, spanning the core invariants, capital-custody settlement, tokenized buyout logic, the full rescission suite, and the *maslahah* disposition — including adversarial cases (mispriced buyout rejected; role violations reverted; rescission refused once performance has begun).

### 6.2 On-chain demonstration

The contracts were deployed and exercised on the Hedera testnet across several configurations. Two results carry the thesis.

**Capital-custody settlement under loss (three distinct accounts).** With financier, client, and valuer held by separate funded accounts, role-violating transactions reverted on-chain (a partner could not attest value; a non-partner could not settle). After the independent valuer attested a 10% write-down, settlement paid the financier exactly **72,000,000** of the **80,000,000** tinybar it had funded — it bore 8,000,000, its 80% share of the 10,000,000 loss, enforced by transfer between separate parties on a public network.

**Atomic buyout and clean dissolution.** A single `buyShare` transaction moved 2,000 ownership units from the contract's escrow to the client (escrow 8,000 → 6,000) and the hbar price to the financier, atomically. Across a full step-down, ownership transferred from 8,000 to 0 basis points and rent fell from 80,000 to 0 tinybar in lockstep, terminating in full client ownership (Figure 1). Under a downward revaluation the loss was borne through *lower buyout proceeds* — the price for a tranche fell from 20,000,000 to 18,000,000 tinybar after a 10% write-down — and a dissolution cleanly returned the financier's remaining units.

*Figure 1* shows ownership transferring and rent falling in lockstep to zero across the buy-down. *Figure 2* shows the loss-share crossing from financier to client as ownership diminishes, anchored by the on-chain measured point.

### 6.3 Cost

Per-operation testnet fees are fractions of one ℏ:

| Operation | Gas used | Fee (ℏ) |
|-----------|---------:|--------:|
| `payRent` | 34,546 | 0.03696 |
| `buyShare` | 49,599 | 0.05307 |
| `attest` (oracle) | 27,908 | 0.02986 |
| `syncValuation` | 62,800 | 0.06720 |
| **Full lifecycle** | **174,853** | **0.18709** |

The complete lifecycle cost **0.187 ℏ**. Because Hedera fees are USD-pegged and predictable by design, each state-changing operation costs on the order of a few US cents [verify against current HBAR price]. Against an instrument whose conventional analogue carries recurring servicing, custody, and audit overheads, the on-chain mechanics are firmly within industrial reach — with the important caveat in Section 7 that valuation cost, excluded here, dominates the real total.

### 6.4 Security

The contracts pass both static gates with no security defects: solhint returns 0 errors (only style and gas-optimisation warnings), and Slither (63 detectors over the contract set) returns 0 findings after a hardening pass that made set-once state variables immutable and removed vestigial state. No reentrancy, arbitrary-send, or unchecked-call vulnerabilities were reported. Deeper formal verification remains optional future hardening.

### 6.5 Comparison

The contribution is clarified by comparison with the two prevailing alternatives — conventional, board-attested *Musharakah Mutanaqisah*, and the dominant "transparency-layer" use of blockchain in Islamic finance:

| Dimension | Conventional MMP (board-attested) | Transparency-layer blockchain-IF | This work (compliance by construction) |
|---|---|---|---|
| Compliance decision | ex-post human attestation | ledger records; board still attests | **enforced by execution (invariants)** |
| Loss-sharing | contractual; manually settled | recorded, not enforced | **enforced by transfer at settlement** |
| Monitoring cost | high (periodic human audit) | medium | low (machine-checked state transitions) |
| On-chain rescission rights | n/a (off-chain legal process) | typically none | *khiyar* / *iqalah* / defect / authoritative *faskh* |
| Asset representation | registry only | token (provenance) | token **custodied and moved by the contract** |
| Auditability | periodic snapshots | continuous (read-only) | continuous **and enforced** |
| Residual trust locus | board + counterparties | board + oracle + counterparties | **the valuer (explicit, access-controlled)** |

The decisive column is the first row: where the alternatives place the compliance decision in human hands after the fact, the present artifact makes it a property of execution.

## 7. Discussion

**What the results establish.** For the properties a protocol can verify — ownership, rent, buyout pricing, loss apportionment, settlement, and the rescission rights — the conditions of a diminishing partnership can be enforced by the contract itself, demonstrated on a public network between distinct parties. The form-over-substance critique (F1) is met by construction; the risk-sharing monitoring cost (F2) is collapsed for the verifiable components; the immutability–flexibility tension (F3) is answered by encoding the rescission rights and an authoritative *faskh* path.

**Dialogue with the literature.** Where the agency-cost literature explains the retreat from PLS by the prohibitive cost of verifying a partner's position, our results lower that cost toward zero for the machine-checkable components. Where El-Gamal (2006) and Usmani (2007) indict *Shari'ah arbitrage*, compliance by construction narrows the space in which it operates — the gap between certified form and observed substance — for everything the protocol can see.

**Theoretical implication: Shari'ah as protocol.** The deeper implication is a shift in *where compliance lives*: from a periodic, human, ex-post judgement to a continuous, computational, ex-ante property. This reframes the Shari'ah supervisory board from *certifier of an instrument* to *auditor of a protocol*, with consequences for standard-setters such as AAOIFI, whose standards might in time be expressed partly as verifiable contract specifications rather than prose alone.

**Limitations, stated plainly.** First, the **oracle / valuation (F4)**: the contract enforces that no contracting partner sets the value, but the independent valuer's integrity remains an external assumption — the relocated, explicit locus of *gharar* — and in reality professional valuation is periodic and costly, a cost excluded from the on-chain figures of Section 6.3 and one that would dominate the true total. Second, **legal title**: on-ledger ownership is a token, but legal title to a real asset lives in a state registry; without a recognised bridge, the enforced state is a private ledger, not ownership. Third, **post-performance rescission**: the encoded *khiyar* and *iqalah* unwind only before performance; unwinding after partial buyouts, which must apportion ownership and payments already made, is future work. Fourth, **scope**: a single instrument, on a test network; production deployment, multi-instrument generalisation, and a field study with real valuation data remain ahead.

## 8. Shari'ah Governance and the Boundary of an Engineering Claim

We are deliberate about what this work does and does not assert. It does **not** claim the artifact is Shari'ah-compliant; that is a scholarly ruling, not an engineering output. It claims only that the artifact *encodes conditions a board can audit*, and that — as Section 2.2 noted — the diminishing partnership itself remains subject to *khilaf*. No fatwa is claimed.

This boundary has a sharp methodological consequence in an age of AI-assisted scholarship. Candidate evidences (*ibarat*) for the encodings — whether assembled by a human or by a language model — must be verified (*takhrij*) against primary sources by a qualified person before any reliance. Confident, well-formatted citation is not authenticity; and misattribution to the Prophet ﷺ or to scripture is a grave matter. Accordingly, the appropriate next step is not a stronger claim but a genuine *istifta* — a structured request for a ruling — submitted to a qualified board. We have prepared such a set of questions, spanning the partnership structure (the nature of the purchase undertaking; the absence of a capital guarantee; loss strictly by capital ratio; the legitimacy of the rent; price determinacy at execution) and the novel ledger-specific issues (whether a token constitutes *milk* and its transfer *qabd*; whether oracle-attested valuation introduces *gharar*; whether code execution constitutes valid offer and acceptance and consent; whether the contract may be regarded as a *wakil*; whether the encoded rescission and *faskh* suffice to avoid foreclosing lawful redress; the permissibility of the network's native token as a fee medium; and the disposition of the loss residue). The ruling, and the *takhrij* of any evidence, belong to those firmer in knowledge.

## 9. Conclusion and Future Work

We asked whether the conditions distinguishing a genuine Shari'ah-compliant partnership from disguised debt could be enforced by a contract's own execution, at viable cost. Building a capital-custodying *Musharakah Mutanaqisah* on Hedera and exercising it adversarially across distinct accounts, we answered yes for the properties a protocol can verify: rent that diminishes with ownership; a buyout priced only from an independent valuation; ownership held as a real token moved atomically with payment; a loss the financier provably bears by transfer; and lawful rescission, including an authoritative *faskh*, encoded rather than foreclosed — all at a cost of fractions of one ℏ, with no security findings.

The contribution is a principle, *compliance by construction*, and a demonstration that it is buildable and, for the reviewed objections, correct. The larger claim is the one we began with: the Shari'ah economy is the substantive proposal, and the ledger an instrument that can run part of it faithfully. What remains — encoding post-performance unwinding, binding to a tokenized real asset with legal recognition, decentralising the valuer, generalising beyond a single instrument, and submitting all of it to qualified scholarly review — measures how much is still to do, not whether the road is real. *Allahu a'lam.*

## Acknowledgements

The artifact, its tests, and the on-chain records are openly available for independent reproduction and verification.

## References

> Academic and standards references are verified. Scriptural and hadith references are given by surah:ayah and collection and marked [scholar-verify]; a qualified scholar must confirm exact wording, hadith grading, and all points of ruling. No fatwa is claimed.

- AAOIFI. *Shari'ah Standard No. 12: Sharikah (Musharakah) and Modern Corporations.* Accounting and Auditing Organization for Islamic Financial Institutions, Bahrain.
- El-Gamal, M. A. (2006). *Islamic Finance: Law, Economics, and Practice.* Cambridge University Press.
- Hevner, A. R., March, S. T., Park, J., & Ram, S. (2004). Design Science in Information Systems Research. *MIS Quarterly, 28*(1), 75–105.
- OIC International Islamic Fiqh Academy. (2009). *Resolution 179 (19/5) on Tawarruq.* 19th session, Sharjah, UAE, 26–30 April 2009. [Organised *tawarruq* ruled impermissible.]
- Peffers, K., Tuunanen, T., Rothenberger, M. A., & Chatterjee, S. (2007). A Design Science Research Methodology for Information Systems Research. *Journal of Management Information Systems, 24*(3), 45–77. https://doi.org/10.2753/MIS0742-1222240302
- Usmani, M. T. (2007). *Sukuk and their Contemporary Applications.* [as circulated via the AAOIFI Shari'ah Board] [scholar-verify exact edition].
- Qur'an, al-Baqarah 2:275; al-Hashr 59:7. [scholar-verify]
- Sahih Muslim, *Kitab al-Buyu'* (prohibition of *bay' al-gharar*). [scholar-verify exact hadith number]
- Legal maxims cited: *al-ghunm bil-ghurm*; *al-kharaj bil-daman* (reported in the Sunan collections) [scholar-verify].

## Appendix A — Figures

**Figure 1.** Diminishing partnership on Hedera testnet: ownership transfers and rent falls in lockstep to zero.

![Figure 1](figures/fig1_ownership.png)

**Figure 2.** Proportional loss-sharing of a write-down by live ownership ratio; the on-chain measured point is annotated.

![Figure 2](figures/fig2_loss.png)
