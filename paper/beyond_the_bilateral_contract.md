---
title: "Beyond the Bilateral Contract: Compliance-by-Construction for the Surrounding Fiqh of Muʿāmalāt"
subtitle: "Composition, Capacity, Charge, Contingency, and Concealment in the `fiqhc` Compiler"
author: "SyariahChain — design-science extension (Paper III)"
date: "2026-06-24"
---

## Abstract

Two prior stages of this research established *compliance by construction* — the idea that the
conditions distinguishing a genuine Shari'ah-compliant instrument from disguised debt can be
**enforced** by execution rather than **attested** after the fact — first for a single hand-written
*Musharakah Mutanaqisah* contract, then as a property of a *language*, the `fiqhc` compiler, in
which a specification that disguises *riba* as a partnership fails to compile. Both stages,
however, reasoned about the **bilateral ʿaqd**: one contract, between two parties, at one moment.
But the classical *fiqh al-muʿāmalāt* does not end at the bilateral contract. It governs the
*composition* of contracts (where a ruse hides between impeccable legs), the *capacity* of the
contracting parties (*ahliyyah*), the obligatory *charge* upon accumulated wealth (*zakāt*), the
*contingencies* that abate or dissolve an obligation (calamity and death), and the legitimate
*concealment* of private quantity against a public ledger's transparency. A compliant contract is
not yet a compliant economy. This paper reports five extensions to `fiqhc`, each closing one of
these gaps and each mapping to an established juristic doctrine: (1) a graph-based checker that
detects *riba*-by-composition — *bayʿ al-ʿīnah* and organised *tawarruq* — as a **topological**
property of an asset-flow graph (a cycle), not a property of any single leg; (2) an *ahliyyah*
middleware that verifies each party's *ahliyyat al-adāʾ* through decentralised credentials before
a compiled contract may execute; (3) an algorithmic *zakāt al-tijārah* layer that computes
*rubʿ al-ʿushr* (2.5%) at the abstract-syntax level and routes it, non-bypassably, to a *maṣlaḥah*
fund; (4) lifecycle off-ramps that encode *waḍʿ al-jawāʾiḥ* (calamity abatement that may never add
interest) and a deterministic *farāʾiḍ* engine that dissolves a partnership by the fixed Qur'anic
shares on a partner's death; and (5) a zero-knowledge protocol that proves loss-sharing is
proportional **without revealing the amounts**, reconciling the transparency the protocol needs
with the privacy a commercial party may keep. Each is implemented in Rust, verified by `cargo
test`, and — where it touches a contract — proven on a local-EVM Hardhat run; all five are opt-in
and leave the existing instruments byte-unchanged. The epistemic boundary is unchanged and
load-bearing: the engine proves *consistency with a declared, human-authored, citation-bearing
rule-base*; it issues **no fatwa**. *Allahu aʿlam.*

**Keywords:** Islamic finance; *fiqh al-muʿāmalāt*; compliance by construction; *sadd
al-dharāʾiʿ*; *ahliyyah*; *zakāt*; *farāʾiḍ*; zero-knowledge proof; design science; smart
contracts.

---

## 1. Introduction

A compiler that refuses to lower a disguised loan is a real result, but it is a result about a
**single contract** seen in isolation. The two earlier stages of this work — a capital-custodying
*Musharakah Mutanaqisah* on Hedera, and then `fiqhc`, a compiler in which non-compliance is
*unrepresentable* — both reasoned about the bilateral *ʿaqd*: its parties, its return mechanism,
its allocation of risk, checked at the moment of lowering. That altitude is necessary. It is not
sufficient.

The *fiqh al-muʿāmalāt* the jurists actually built reaches well past the two-party contract. It
asks whether a *combination* of individually licit contracts conceals an unlawful end (the doctrine
of *sadd al-dharāʾiʿ*, blocking the means). It asks *who* contracts — whether a party possesses the
*ahliyyah* (legal capacity) the law requires. It lays an obligatory *charge*, *zakāt*, upon wealth
that the contract accumulates. It provides emergency exits the contract did not negotiate —
abatement when calamity strikes, and a fixed distribution when a contracting party dies. And it
recognises that not everything must be exposed: a Muslim is not obliged to publish the figures of
his wealth, even where a public ledger would, by default, reveal them. None of these lives *inside*
the bilateral contract. Each lives in the lattice around it.

This paper's claim is therefore one of altitude rather than of a new mechanism: **compliance by
construction must reach the surrounding fiqh, or it secures the instrument while the economy
escapes it.** We report five extensions to `fiqhc`, organised under five doctrines — and, for the
reader's map, five words: **Composition, Capacity, Charge, Contingency, Concealment.** Each extension
is a place the prior proofs were blind, and each turns out to be enforceable at a *different layer*
of the system: the topology of a flow graph, the identity layer, the abstract-syntax tree, the
lifecycle state machine, and the cryptographic proof layer.

The contributions are:

1. **A reframing.** That *compliance by construction* is not a property of one contract or even one
   language of contracts, but a discipline that must be applied across the lattice of *fiqh* that
   surrounds the *ʿaqd* — and that this lattice decomposes cleanly into five enforceable layers.
2. **Five artifacts.** A composite-contract cycle detector; an *ahliyyah* credential middleware; an
   AST-level *zakāt* layer; *jawāʾiḥ*/*farāʾiḍ* lifecycle off-ramps with an exact-arithmetic
   inheritance engine; and a zero-knowledge proof of proportional loss-sharing.
3. **Evidence.** For each, a Rust implementation verified by unit and integration tests, and —
   where it touches a contract — generated Solidity proven on a local EVM; all opt-in, with the four
   base generators unchanged.
4. **A boundary, restated.** Each doctrine carries *khilāf*; the engine proves consistency with a
   declared rule-base and issues no ruling. The hardest cases — an unmodeled inheritance
   configuration, a judgment of a party's *rushd* — are referred *out* of the machine to a
   qualified person, by design.

Section 2 reviews what the single contract cannot see. Section 3 develops the five-doctrine thesis.
Section 4 states the method. Section 5 presents the five artifacts. Section 6 reports the
evaluation. Section 7 discusses meaning and limits; Section 8 treats the Shari'ah-governance
boundary; Section 9 concludes.

## 2. Background: what the single contract cannot see

The two prior stages of this research are the immediate background, and we build directly on their
machinery — the `.fiqh` DSL, the Rust semantic engine, the separation of a human-authored
rule-base from a mechanical enforcement engine, and the design-science discipline of Hevner et al.
(2004) and Peffers et al. (2007). What follows situates each of the five gaps in the juristic
literature and explains why an instrument-level or language-level check is structurally blind to
it.

**Composition.** The prohibition of *riba* can be evaded by *combining* contracts each of which is,
alone, unobjectionable. *Bayʿ al-ʿīnah* — selling a good on deferred terms and immediately buying
it back for less in cash — yields, in substance, a cash loan repaid with an increase, while wearing
the form of two sales. *Tawarruq*, in its *organised* (institutional) form, achieves the same
through a third party arranged by the financier. The jurists who forbid these do so not by faulting
any single sale but through *sadd al-dharāʾiʿ*, blocking a licit means to an illicit end — a source
of law emphasised in the Mālikī and Ḥanbalī schools and treated at length in al-Shāṭibī's
*al-Muwāfaqāt* [scholar-verify]. A checker that validates one contract at a time cannot, in
principle, see a ruse that exists only in the *relation between* contracts.

**Capacity.** Classical fiqh distinguishes *ahliyyat al-wujūb* (the capacity to bear obligations,
present from birth) from *ahliyyat al-adāʾ* (the capacity to execute a binding contract, requiring
*bulūgh*, *ʿaql*, and *rushd*). The law interdicts (*ḥajr*) the *safīh* (the prodigal lacking sound
judgment) and the bankrupt (*muflis*) to protect them and their creditors (al-Baqarah 2:282;
al-Nisāʾ 4:5 [scholar-verify]). A contract may be impeccable in its terms and still void for the
incapacity of a party — a fact about *identity and status*, which no analysis of the contract's
text can supply.

**Charge.** *Zakāt al-tijārah*, the *zakāt* due on trade wealth, is an obligation the majority of
the four schools attach to inventory and cash held above the *niṣāb* for a lunar year (al-Tawbah
9:103; AAOIFI Shari'ah Standard No. 35 [scholar-verify]). It is not a term the two contracting
parties negotiate; it is a charge the *Lawgiver* lays upon the wealth itself. A contract-level
compliance proof never raises it, because it is not part of the contract.

**Contingency.** The fiqh provides exits the parties did not write. *Waḍʿ al-jawāʾiḥ* — the
remission of an obligation when an uncontrollable calamity destroys the subject matter — is grounded
in the *ḥadīth* of Jābir (Sahih Muslim, *Kitāb al-Buyūʿ* [scholar-verify]) and is most developed in
the Mālikī school; crucially, it *abates*, and may never convert an obligation into a penalty. And
upon a partner's death, the partnership does not simply continue or freeze: the deceased's share
devolves by the fixed shares of *al-farāʾiḍ* (al-Nisāʾ 4:11–12, 4:176 [scholar-verify]). Both are
*lifecycle* events outside the contract's negotiated logic.

**Concealment.** The thesis of this whole research program is that a public, verifiable ledger lets
compliance be *proven, not asserted*. Yet verification by *disclosure* is in tension with a
legitimate privacy: a Muslim is not commanded to publish the figures of his wealth, and exposing
them can invite *ḥasad* and ostentation, which the tradition discourages. A global institution will
not place its real loss figures on a public chain. The question — can an invariant be *proven*
about quantities that remain *hidden*? — is one neither prior stage could pose, because both assumed
the figures were on-chain in the clear.

Across all five, the pattern is the same: the bilateral-contract proof secures the *form and
matter* of one *ʿaqd*, while the obligation that actually governs a Muslim economy lives one layer
out — in composition, in the parties, in the charge on wealth, in time, and in what may be kept
private.

## 3. The five doctrines beyond the ʿaqd

The validity of an *ʿaqd* in classical fiqh rests on its *arkān* (pillars) and *shurūṭ*
(conditions): the contracting parties (*al-ʿāqidān*), the form (*ṣīghah*), and the subject matter
(*al-maḥall*). A single-instrument compliance proof, and even a language of such proofs, checks the
*ṣīghah* and the *maḥall* of one contract. The five doctrines below are precisely the conditions
that fall *outside* that check — and our claim is that each is enforceable, at its own layer, with
the same discipline.

| # | Doctrine (the "C") | Classical basis | Where the single contract is blind | Enforcement layer |
|---|---|---|---|---|
| 1 | **Composition** | *al-ʿuqūd al-murakkabah*; *sadd al-dharāʾiʿ* | the ruse lives *between* legs, not in any one | the **topology** of an asset-flow graph |
| 2 | **Capacity** | *ahliyyat al-adāʾ*; *ḥajr* on *safīh*/*muflis* | a fact about the *party*, not the text | the **identity / credential** layer |
| 3 | **Charge** | *zakāt al-tijārah*; *rubʿ al-ʿushr* | an obligation of the *Lawgiver*, not a term | the **abstract-syntax tree** |
| 4 | **Contingency** | *waḍʿ al-jawāʾiḥ*; *al-farāʾiḍ* | events *outside* negotiated logic | the **lifecycle state machine** |
| 5 | **Concealment** | privacy of wealth vs. *bayyinah* | the figures need not be public to be proven | the **cryptographic proof** layer |

The deeper unity is methodological. In each row, an unlawful state is made *unrepresentable or
unexecutable* at the layer where it would otherwise hide: a riba cycle cannot be lowered; a party
without capacity cannot be authorised; the *zakāt* due cannot be bypassed; a calamity can only
abate and a death can only distribute by the fixed shares; and a disproportionate loss cannot
produce a valid proof. This is *compliance by construction* applied not to a contract but to the
fiqh that surrounds it.

## 4. Method

We follow the design-science paradigm already adopted in this program (Hevner et al., 2004; Peffers
et al., 2007): the contribution is the construction and rigorous evaluation of artifacts, and each
extension is built, tested, and — where it touches a contract — exercised on a ledger.

**Implementation.** Each vector is implemented in Rust as part of `fiqhc` (the contribution),
verified by `cargo test`. Where a vector emits or modifies a contract, the generated Solidity is
compiled with `solc 0.8.24` and exercised by generated Hardhat tests on a local EVM. The identity
and gateway layers are Node services. Every new construct is **opt-in** — a new DSL section, a new
build target, or a side service — so the four base generators (*Musharakah Mutanaqisah*,
*Mudarabah*, *Ijārah Muntahia Bittamlīk*, *commercial escrow*) remain byte-unchanged and the
codegen-safety properties established earlier continue to hold.

**Epistemic boundary.** Unchanged from the prior stages and central to this one: the engine proves
a specification *consistent or inconsistent with a declared, human-authored, citation-bearing
rule-base*. It issues **no fatwa**. Every fiqh citation is flagged `[scholar-verify]`; common-law
citations `[verify]`. Where a doctrine carries *khilāf*, the engine encodes one declared position
and says so; where a case exceeds what is safely modelled, the engine refuses to guess and refers it
to a qualified person. This referral is treated as a feature, not a failure (Section 8).

## 5. The artifacts

### 5.1 Composition — riba as a property of a graph (*al-ʿuqūd al-murakkabah*)

A single leg may be impeccable while the *composition* encodes a ruse. We add a top-level `bundle`
construct that declares *legs* — directed asset and cash flows between parties — and a checker,
`composite.rs`, that builds the directed **asset-flow graph**, enumerates its simple cycles by
depth-first search, and classifies them.

```
bundle InahDisguised {
  parties { bank: financier; customer: client; }
  legs {
    sale:    murabahah { from: bank;     to: customer; asset: widget; payment: deferred; price: 11000; }
    buyback: bay       { from: customer; to: bank;     asset: widget; payment: spot;     price: 10000; }
  }
}
```

| code | structure detected | citation |
|------|--------------------|-----------|
| `INAH-1` | **bayʿ al-ʿīnah** — a 2-cycle returning the same asset to its origin, with a deferred leg and a price differential (the time-and-price gap *is* the *riba*) | Abu Dawud (*ḥadīth* of *ʿīnah*); majority via *sadd al-dharāʾiʿ* `[scholar-verify]` |
| `INAH-2` | **organised *tawarruq*** — a ≥3-leg ring returning the asset to the financier, *and* the monetisation flip (deferred-in + spot-out of the same asset) | OIC Fiqh Academy Res. 179 (19/5), Sharjah 2009; AAOIFI SS No. 30 `[scholar-verify]` |
| `INAH-3` | (warning) a round-trip with no net economic transfer — a formalistic red flag (*ḥīlah*) | — |

The result that carries the section is a **contrast**: the *same* deferred markup that is *riba*
inside the *ʿīnah* cycle is licit in an *acyclic* `wakālah + murābaḥah` composite, where the
customer keeps the asset and no cycle returns it to the financier. The engine therefore forbids the
**cycle**, not the markup — which is exactly the logic of *sadd al-dharāʾiʿ*, where the same act is
permitted or blocked according to the end it serves. We are candid that *ʿīnah* itself is subject to
*khilāf* (the Shāfiʿī school, attending to the form of each separate sale, was more permissive than
the Mālikī and Ḥanbalī); the engine encodes the majority position and flags it. `fiqhc check|build`
routes bundles; `build` emits a composite manifest.

### 5.2 Capacity — *ahliyyah* before authorisation

Fiqh is not only about a contract's content but about *who* contracts. The invariant gateway gains
an *ahliyyah* layer (`services/ahliyyah.js`): it resolves each party's decentralised identifier
(DID) to a verifiable credential and checks *ahliyyat al-adāʾ* — the capacity for binding execution
— before a compiled contract may run.

```
POST /authorize { target, terms, parties: { financier: "did:fiqh:bank-alpha", acquirer: "did:fiqh:…" } }
  → allowed only if BOTH the invariant terms AND every party's capacity pass.
```

| code | refusal | citation |
|------|---------|-----------|
| `AHL-MINOR` | a minor (*ṣaghīr*) — lacks *bulūgh* | *ahliyyat al-adāʾ* requires *bulūgh* `[scholar-verify]` |
| `AHL-SAFIH` | the interdicted (*safīh*/*majnūn*) — lacks *rushd*/*ʿaql* | al-Baqarah 2:282; al-Nisāʾ 4:5 `[scholar-verify]` |
| `AHL-TAFLIS` | a bankrupt under *ḥajr* — estate reserved for creditors | Sahih Muslim (*ḥadīth* of the *muflis*) `[scholar-verify]` |
| `AHL-KYC` / `AHL-AML` | statutory overlay | — |

The compiler emits `ahliyyah.principals` — the contracting roles whose capacity the gateway must
require — into the manifest, so authorisation refuses on *either* a bad term or an incapable party.
The boundary is sharp and deliberate: an issuer *attests* a credential, and the engine *checks* it,
but the *ruling* that a given person is *safīh*, or that an interdiction is lawful, belongs to a
*qāḍī* or scholar. The engine verifies a status that a competent authority has already established;
it does not adjudicate capacity.

### 5.3 Charge — *zakāt al-tijārah* at the abstract-syntax tree

A `zakat { }` section lifts corporate *zakāt* from a year-end act of conscience into a property of
the contract. `zakat.rs` computes *rubʿ al-ʿushr* (one-fortieth = 2.5%) by exact, widened
integer arithmetic, and the generated contract routes it — non-bypassably, on-chain — to the
*maṣlaḥah*/*zakāt* fund.

```
zakat {
  rate_bps: 250;            // 1/40 — the only valid trade-goods rate
  nisab: 8500000 tinybar;   // the 85g-gold / 595g-silver equivalent
  haul: hijri_year;         // a lunar year — a solar haul under-collects
  beneficiary: maslahah;
}
```

`ZAKAT-1` the rate must be 250 bps · `ZAKAT-2` the *ḥawl* must be lunar (a solar year is ~11 days
longer and so under-collects) · `ZAKAT-3` a positive *niṣāb* · `ZAKAT-4` the beneficiary must
resolve to a party. The generated Solidity exposes `zakatDue(base)` (zero below the *niṣāb*) and
`payZakat(base)`, which moves exactly 2.5% to the existing `maslahahFund` — a one-line policy change
turns it from opt-in to mandatory, with no constructor change and so no destabilisation of the base
contract. *al-Tawbah 9:103; Abu Dawud (Samurah b. Jundub, on giving from what is prepared for sale);
AAOIFI SS No. 35 `[scholar-verify]`.* The honest scope: the *niṣāb* anchor (gold vs. silver) and
whether trade-goods *zakāt* is obligatory at all carry difference among the schools; the engine
computes a *due* and routes it, but the obligation and its *niyyah* remain the owner's, and the
parameters are a scholar's to ratify.

### 5.4 Contingency — *jawāʾiḥ* abatement and *farāʾiḍ* dissolution

A `contingency { }` section declares the emergency exits the fiqh itself provides.

```
contingency {
  jaaihah: reschedule;   // calamity abates the obligation — never adds interest (waḍʿ al-jawāʾiḥ)
  death:   faraid;       // dissolution distributes by the fixed Qur'anic shares
}
```

- `CONT-1` — a *jāʾiḥah* may only `reschedule`/`abate`, never trigger a penalty. The generated
  `declareJaaihah()` drops `effectiveRentDue()` to zero; `rescheduleWithoutRiba()` may only *extend*
  a deadline. This is the precise shape of *waḍʿ al-jawāʾiḥ*: calamity removes a burden; it cannot be
  the occasion to add one, which would be *riba* by the back door.
- `CONT-2` — a death may only invoke `faraid`. `faraid.rs` is a deterministic inheritance engine
  computing in **exact rational arithmetic** (fractions over 128-bit integers, never floating point):
  it handles spouse, parents, and children with the *ʿaṣaba* 2:1 apportionment, and the two classical
  corrections — *ʿawl* (proportional reduction when the fixed shares exceed unity, as ʿUmar ruled with
  the Companions) and *radd* (return of residue to the sharers, *excluding* the spouse, per the Ḥanafī
  and Ḥanbalī position). It explicitly guards the *ʿUmariyyatān* (*Gharrāwayn*) case, and where an
  estate exceeds what is modelled — distant *ʿaṣaba*, *ḥajb* configurations it does not encode — it
  **refuses to guess and refers the case to a *farāḍī***. The generated `dissolveByFaraid(heirs,
  shareBps)` distributes the deceased partner's escrowed capital by shares validated to total exactly
  10,000 bps, making the bank whole on its own share.

*Sahih Muslim (Jābir, on *waḍʿ al-jawāʾiḥ*); al-Nisāʾ 4:11–12, 4:176 `[scholar-verify]`.* We note
the *radd*-excluding-spouse choice is a madhhab position (the Mālikī and Shāfiʿī classically returned
residue to the *bayt al-māl*); the engine encodes one declared school and says which.

### 5.5 Concealment — proving proportional loss in zero knowledge

A public ledger makes compliance verifiable by making state visible; but a global institution will
not place its real loss figures in the clear, and the fiqh does not oblige a Muslim to publish the
quantities of his wealth. We resolve the tension cryptographically: prove the invariant, hide the
figures.

- `zk.rs` is a self-contained sigma-protocol proof-of-concept (its own SHA-256, a Miller–Rabin
  safe-prime search, Pedersen commitments, and a Schnorr/Fiat–Shamir transform) that proves the
  committed losses satisfy `lossBank · clientBps == lossClient · bankBps` — **[RISK-1]**, loss borne
  in proportion to ownership — **without revealing the amounts**. It reduces the relation to proving
  that `C_z = C_b^{clientBps} · C_c^{(q − bankBps)}` is a commitment to *zero* (a Schnorr proof of
  knowledge on the blinding base). *Honest scope: the protocol is real and tested; the ~61-bit
  modulus is illustrative, not production-grade.*
- `fiqhc build --target zk` is the **production** path: it emits a Circom circuit (private
  loss witnesses; the INV-1 and RISK-1 constraints as R1CS), a Groth16 zero-knowledge manifest for
  the snarkjs pipeline, and a Solidity `settleWithProof()` gate that admits a settlement only on a
  valid proof of proportional loss.

The fiqh resonance here is intentionally light: we do not claim the Shari'ah *mandates*
zero-knowledge, only that it permits a party to keep private what it may keep private while the
*invariant* the community cares about — that loss was shared, not shifted — is still *proven, not
asserted*. The foundations are standard (Pedersen, 1991; Schnorr, 1991; Fiat & Shamir, 1986; Groth,
2016). *AAOIFI SS No. 12 `[scholar-verify]`.*

## 6. Evaluation

Each vector is verified at the level its claim demands: Rust unit and integration tests for the
engine logic, generated Hardhat tests for the contract behaviour, and live service runs for the
identity/gateway layers.

| # | Doctrine | Artifact | Verification |
|---|---|---|---|
| 1 | Composition | `composite.rs`, `bundle` grammar | `tests/composite.rs` 5/5; the ʿīnah/tawarruq specs refused, the acyclic *wakālah+murābaḥah* admitted |
| 2 | Capacity | `ahliyyah.js`, `/authorize`, DID registry | module smoke 12/12; live `/authorize` 5/5 (capable allowed; minor/bankrupt/incapable refused) |
| 3 | Charge | `zakat.rs`, `zakat{}`, generated routing | `tests/zakat.rs` 5/5; generated `MusharakahZakatGen` 6/6 Hardhat — the *maṣlaḥah* balance rises by *exactly* 2.5%, and `zakatDue = 0` below the *niṣāb* |
| 4 | Contingency | `faraid.rs`, `contingency{}`, off-ramps | `tests/contingency.rs` 4/4; faraid 9/9 vs textbook estates; generated `MusharakahJaaihahGen` 7/7 Hardhat — rent abates to zero, capital distributes by validated shares |
| 5 | Concealment | `zk.rs`, `--target zk` | zk unit 8/8 (honest proof verifies; disproportionate and tampered proofs reject; amounts hidden); `tests/zk.rs` 3/3; circuit and gate emitted |

The full `cargo test` suite is green across all integration binaries; the fuzz harness from the
prior stage remains clean; and the four base generators are byte-unchanged, so the earlier
codegen-safety proofs (every external value-mover carries `nonReentrant`; role modifiers and pinned
pragmas present) continue to hold for the new generated contracts. Two evaluations are deliberately
*deferred* and we say so plainly: a *live testnet* deploy of a zakat-/jaaihah-/zk-gated contract
awaits an operator-account top-up (one contract-create costs ~12–35 ℏ at the depressed testnet
price), and the production ZK path emits a Circom circuit that has not been compiled in-container (a
*circom* toolchain is not installed; the protocol is proven instead by the Rust sigma-PoC). The
honesty of these deferrals is itself part of the result.

## 7. Discussion

**What the results establish.** *Compliance by construction* is not confined to the bilateral
contract; it spans layers. The same discipline that refuses a disguised loan at compile time also
refuses a riba *cycle* in the graph, an incapable *party* at the gateway, a bypassed *charge* at the
AST, a penalising *contingency* in the lifecycle, and a disproportionate *loss* at the proof layer.
The most interesting single finding is that *riba*-by-composition is **topological** — a property of
the shape of the flow graph, not of any node — which is why a leg-by-leg audit, human or
mechanical, structurally cannot catch it, and a cycle-aware one can.

**Dialogue with the literature.** Where the form-over-substance critique (El-Gamal, 2006; Usmani,
2007) indicts *ḥiyal* and *Shari'ah arbitrage*, the composition checker attacks the most
characteristic *ḥīlah* of all — the buy-back ring — at exactly the layer where it operates. Where
the agency-cost literature explains the retreat from profit-and-loss-sharing by the cost of
verifying a partner's position, the zero-knowledge layer shows that verification need not require
*disclosure*: an invariant can be proven over hidden figures, which lowers the monitoring cost
without forcing a party to expose its books.

**Limits, stated plainly.** Each vector carries an honest residue. *Composition*: the asset-flow
graph is only as faithful as the modelled legs; an off-graph side agreement remains invisible, and
*ʿīnah* itself is *khilāf*. *Capacity*: the engine verifies a credential but does not adjudicate
*rushd*; a fraudulent or stale credential defeats it, so the issuer is a trust locus. *Charge*: the
engine computes a due against declared parameters; valuing the trade base, anchoring the *niṣāb*,
and the very obligation carry difference, and *niyyah* is the owner's. *Contingency*: the *farāʾiḍ*
engine encodes one school's *radd* and a bounded set of heirs, referring the rest out — by design,
but a limit nonetheless. *Concealment*: the PoC modulus is illustrative, and the production Circom
circuit is emitted but not yet compiled. None of these is hidden; each marks the edge of what code
can honestly claim.

## 8. Shari'ah governance and the boundary of the claim

This paper's extensions sharpen, rather than soften, the boundary the program has held throughout.
The engine proves *consistency with a declared rule-base*; it issues **no fatwa**. Three features of
this work make the boundary load-bearing in a new way.

First, **referral is built in.** The *farāʾiḍ* engine does not approximate a hard estate; it detects
that the case exceeds its model and returns it to a *farāḍī*. A system that knows the edge of its own
competence and stops there is more trustworthy than one that always answers.

Second, **the doctrines are contested, and the engine says which side it took.** *ʿĪnah*'s
permissibility, the *radd*-excluding-spouse rule, the *niṣāb* anchor, the very obligation of
trade-goods *zakāt* — each is *khilāf*, and the engine encodes one declared, cited position rather
than presenting a single answer as *the* ruling. The choice among positions is governance, and
governance belongs to people.

Third, **citation is not authenticity.** Every scriptural and *fiqh* reference is flagged
`[scholar-verify]`; a confident, well-formatted attribution is not a verified one, and misattributing
a saying to the Prophet ﷺ or to scripture is a grave matter. The appropriate next step is not a
stronger claim but a genuine *istiftāʾ* to a qualified board — on the composition rules, the capacity
credentials, the *zakāt* parameters, the inheritance school, and the ledger-specific questions
(whether a DID credential suffices to establish *ahliyyah*; whether a zero-knowledge proof can stand
where a *bayyinah* is required; whether routed *zakāt* discharges the obligation). The *takhrīj* of
the evidences, and the rulings, belong to those firmer in knowledge.

## 9. Conclusion and future work

A compiler that refuses a disguised loan secures the contract; it does not yet secure the economy.
We asked whether *compliance by construction* could reach the *fiqh* that surrounds the bilateral
*ʿaqd* — its composition, the capacity of its parties, the charge upon its wealth, the contingencies
of calamity and death, and the privacy a party may keep — and we answered constructively, in five
extensions to `fiqhc`, each enforced at the layer where its doctrine actually lives. *Riba* by
composition became a cycle the compiler will not lower; capacity became a credential checked before
execution; *zakāt* became an exact charge routed at the syntax tree; calamity could only abate and
death could only distribute by the fixed shares; and proportional loss became a property provable
without disclosure.

What remains is the work no compiler can do for us: the live deployment that awaits only a faucet;
the production proof circuit that awaits a toolchain; and, above all, the scholar's ratification of
each contested rule-base, the law's recognition of tokenised title, and a community's adoption. The
machine has been carried as far toward the *whole* of *fiqh al-muʿāmalāt* as construction can carry
it — to the point where non-compliance is, layer by layer, unrepresentable. The rest is *istiftāʾ*,
and trust, and time. *Allahu aʿlam.*

## References

> Academic, standards, and cryptographic references are verified to author, year, and venue.
> Scriptural and *ḥadīth* references are given by *sūrah:āyah* and collection and marked
> `[scholar-verify]`; a qualified scholar must confirm exact wording, *ḥadīth* grading, and every
> point of ruling. No fatwa is claimed.

- AAOIFI. *Shari'ah Standard No. 12 (Sharikah/Musharakah)*, *No. 30 (Tawarruq)*, *No. 35 (Zakah)*.
  Accounting and Auditing Organization for Islamic Financial Institutions, Bahrain. `[scholar-verify]`
- El-Gamal, M. A. (2006). *Islamic Finance: Law, Economics, and Practice.* Cambridge University Press.
- Fiat, A., & Shamir, A. (1986). How to Prove Yourself: Practical Solutions to Identification and
  Signature Problems. In *Advances in Cryptology — CRYPTO '86*, LNCS 263, 186–194.
- Groth, J. (2016). On the Size of Pairing-Based Non-interactive Arguments. In *Advances in
  Cryptology — EUROCRYPT 2016*, LNCS 9666, 305–326.
- Hevner, A. R., March, S. T., Park, J., & Ram, S. (2004). Design Science in Information Systems
  Research. *MIS Quarterly, 28*(1), 75–105.
- OIC International Islamic Fiqh Academy. (2009). *Resolution 179 (19/5) on Tawarruq.* 19th session,
  Sharjah, UAE, 26–30 April 2009. [Organised *tawarruq* ruled impermissible.]
- Pedersen, T. P. (1991). Non-Interactive and Information-Theoretic Secure Verifiable Secret
  Sharing. In *Advances in Cryptology — CRYPTO '91*, LNCS 576, 129–140.
- Peffers, K., Tuunanen, T., Rothenberger, M. A., & Chatterjee, S. (2007). A Design Science Research
  Methodology for Information Systems Research. *Journal of Management Information Systems, 24*(3),
  45–77. https://doi.org/10.2753/MIS0742-1222240302
- Schnorr, C. P. (1991). Efficient Signature Generation by Smart Cards. *Journal of Cryptology,
  4*(3), 161–174.
- Usmani, M. T. (2007). *Sukuk and their Contemporary Applications.* [as circulated via the AAOIFI
  Shari'ah Board] `[scholar-verify exact edition]`.
- al-Shāṭibī, Ibrāhīm b. Mūsā. *al-Muwāfaqāt fī Uṣūl al-Sharīʿah* (on *sadd al-dharāʾiʿ* and
  *maqāṣid*). `[scholar-verify edition and page]`
- Qur'an: al-Baqarah 2:282; al-Nisāʾ 4:5, 4:11–12, 4:176; al-Tawbah 9:103. `[scholar-verify]`
- Sahih Muslim, *Kitāb al-Buyūʿ* (the *ḥadīth* of Jābir on *waḍʿ al-jawāʾiḥ*; the *ḥadīth* of the
  *muflis*); Sunan Abi Dawud (the *ḥadīth* of *ʿīnah*; Samurah b. Jundub on *zakāt* of trade goods).
  `[scholar-verify exact numbers and grading]`
