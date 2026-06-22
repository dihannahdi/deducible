---
title: "From an Artifact to a Primitive: A Compliance-by-Construction Compiler for Islamic-Finance Contracts"
subtitle: "fiqhc — toward an Algorithmic Jurisprudence"
author: "SyariahChain — design-science extension"
date: "2026-06-22"
---

## Abstract

An earlier stage of this research demonstrated a single hand-written family of smart
contracts — a *Musharakah Mutanaqisah* on Hedera — that made shariah compliance
*enforced* rather than merely *attested*. That artifact, however honest, was one instrument
on one network, written by hand. This paper reports a change in kind rather than degree.
We present **`fiqhc`**, a compiler that lifts *compliance by construction* from a property
of one contract to a property of a **language**: from a high-level specification of an
Islamic-finance contract it (a) refuses to lower any specification whose declared economics
contradict its declared fiqh rule-base, and (b) emits a verified, deployable Solidity
contract from one that is consistent. The headline result is negative by design — a
specification that disguises *riba* as a partnership *fails to compile*, with fiqh-cited
diagnostics, and no contract is produced. We prove the compiler is a *primitive* and not a
one-off by generating three structurally different instruments — Musharakah Mutanaqisah,
Mudarabah, and Ijarah Muntahia Bittamleek — from the same engine; all compile and pass
generated property tests, and each rejects its own riba/gharar negative control. One
generated contract was deployed and exercised live on Hedera testnet. We also report an
experimental natural-language front-end whose drafts are subordinate to the same formal
gate. We are explicit about the epistemic boundary: the compiler issues no *fatwa*. It
proves consistency with a human-authored, citation-bearing rule-base; the validity of that
rule-base, and of instruments that carry *khilaf*, remains a qualified scholar's domain.
*Allahu a'lam.*

## 1. From an artifact to a primitive

*"Attention Is All You Need"* (Vaswani et al., 2017) did not make translation marginally
better; it introduced a *primitive* — an architecture general enough to be reused across
domains. The analogy that organises this paper is deliberate. The prior artifact solved one
problem on one chain. The contribution here is to recast *compliance by construction* as a
reusable primitive of **legal computation**: a small domain-specific language (the `.fiqh`
DSL) and a compiler in which **non-compliance is unrepresentable**, and from which compliant
contracts for many instruments are *generated*.

The shift is from *"a compliant contract"* to *"a language in which a contract that
contradicts its declared basis cannot be lowered to code at all."* Compliance moves from a
runtime check, or a board's ex-post attestation, to a property enforced at compile time.

## 2. Compliance by construction, as a property of the language

The fiqh meaning of the DSL is its vocabulary. A `.fiqh` specification declares an instrument
*class*, its parties and roles, the capital split, the return mechanism, the allocation of
risk, named invariants, rescission options, and a lifecycle — all in a fixed fiqh ontology.
The semantic engine reasons over these structured facts. A specification can only *say*
fiqh-meaningful things, and the engine checks their combination. A combination that violates
the declared basis is refused before any code exists.

This is the precise sense in which compliance is a language property: one cannot express
compliance-violating economics that compile.

## 3. Architecture of `fiqhc`

`fiqhc` is written in Rust (lexer → recursive-descent parser → AST → semantic analysis →
code generation). We are explicit about scope. The *contribution* — the DSL, the parser, the
fiqh invariant engine, and the Solidity/test code generation — is entirely in Rust. Rust also
orchestrates. It shells out to the already-proven toolchain for two things it would be
wasteful and risky to reimplement: `solc` (via Hardhat) for Solidity compilation, and the
Hedera SDK (Node) for on-chain deployment. Reimplementing a Solidity compiler or a Hedera SDK
is explicitly *not* the contribution.

The generated Solidity matches the conventions of the hand-written, peer-reviewed artifact:
`immutable` parties, basis-points arithmetic, `onlyX`/`live`/`nonReentrant` modifiers, one
event per state transition, the independent oracle as the trust boundary, and the declared
invariants compiled in as `@dev INVARIANT` annotations on the functions that enforce them.

## 4. The fiqh invariant engine, and the headline refusal

The engine's epistemics are its most important design decision. It issues no *fatwa*. It
proves a specification *consistent or inconsistent with a declared, human-authored,
citation-bearing rule-base R*. The fiqh validity of R is a scholar's judgement; every citation
the engine carries is flagged `[scholar-verify]`. The contribution is the *separation* of the
rule-base (fiqh, to be ratified by people) from the enforcement engine (mechanical, sound) —
the design-science discipline of Hevner (2004) and Peffers et al. (2007) applied to law.

The cross-cutting prohibitions encode *riba* (no guaranteed return of capital; no return tied
to principal), *gharar* (price and value must be independently attested, never self-named),
and role separation (the valuer is never a contracting partner). Per-class rule-sets add the
instrument's required invariants.

The headline demonstration is a specification, `riba_disguised.fiqh`, that wears the
vocabulary of a diminishing partnership while being an interest-bearing loan. `fiqhc` refuses
it (exit non-zero, no `.sol` emitted) with a chorus of cited diagnostics:

```
error[RIBA-1]: capital is guaranteed to 'bank'; a guaranteed return of capital turns a
               partnership into an interest-bearing loan (riba)
      fiqh: Qur'an al-Baqarah 2:275; AAOIFI Shari'ah Standard No. 12 [scholar-verify]
error[RISK-1]: loss allocation is 'none'; a diminishing partnership must share loss
               proportional_to_ownership (no risk-sharing = no partnership)
error[RIBA-2]: rent is charged on principal/capital — interest on a loan, not rent on a
               living share
error[GHARAR-1]: buyout price '11000' is not derived from the independent oracle; a self-named
               or fixed price re-introduces gharar and can disguise a guaranteed return
error[INV-1]: required invariant 'loss_follows_capital' is not declared
error[INV-1]: required invariant 'price_attested' is not declared

refused: 'FakeDiminishingPartnership' is INCONSISTENT with its declared fiqh rule-base
         (6 error(s)). No contract emitted.
```

## 5. Generality: three instruments from one compiler

A primitive must generalise. The same compiler emits three instruments whose economics — and
therefore whose invariants — are genuinely different:

| Instrument | Capital | Return | Loss | Distinctive invariants |
|---|---|---|---|---|
| Musharakah Mutanaqisah | two partners (e.g. 80/20) | rent on the financier's *living* share; oracle-priced buyout | proportional to ownership | `rent_on_living_share`, `loss_follows_capital`, `price_attested` |
| Mudarabah | rabb al-mal 100%, mudarib 0% (labour) | profit by pre-agreed *ratio* | on the rabb al-mal alone (absent the mudarib's *ta'addi/taqsir*) | `capital_from_rabb_al_mal_only`, `profit_by_ratio`, `loss_on_rabb_al_mal`, `no_guaranteed_profit` |
| Ijarah Muntahia Bittamleek | lessor owns the asset | rent for *usufruct* | on the lessor (owner's risk) | `rent_for_usufruct`, `lessor_bears_ownership_risk`, `transfer_separate_from_lease`, `no_late_penalty_interest` |

All three lower to Solidity, compile under `solc 0.8.24`, and pass generated Hardhat property
tests (14 in total: 5 + 4 + 5). Each rejects its own negative control — a riba/gharar
disguise of itself. Mudarabah's loss test verifies on-chain that a 20% shortfall is borne by
the rabb al-mal alone and the mudarib loses only effort; Ijarah's tests verify that ownership
transfers only as a *separate* act after the full term (never two contracts in one), that the
lessor bears impairment, and that any late charge goes to charity, never to the lessor.

## 6. Live evidence on Hedera testnet

The generated Musharakah Mutanaqisah was deployed and exercised on Hedera testnet from its
deploy descriptor, through a single generic runner that serves any generated instrument.

- Contract `MusharakahMutanaqisahGen` = **0.0.9306587**; oracle = 0.0.9306579.
- Three role-separated accounts: bank 0.0.9301070, client 0.0.9306571, arbiter 0.0.9306572,
  beneficiary (maslahah) 0.0.9306574.
- Lifecycle on-chain, all `SUCCESS`: `fundBank`, `fundClient`, `payRent`, `buyShare(2000 bps)`.
- Live state reads after the buyout: `bankShareBps = 6000`, `clientShareBps = 4000` — ownership
  stepped down exactly as specified, at an oracle-attested price, with the constructor's
  cross-contract oracle call working on-chain.

The full record is `deployments/generated_musharakah_mutanaqisah.json`.

## 7. An experimental natural-language front-end

`fiqhc nl` drafts a `.fiqh` specification from a natural-language description via an LLM
(DeepSeek, OpenAI-compatible), then feeds the draft **back through the same formal compiler**.
The LLM never relaxes the shariah; it proposes syntax at the input edge, and a draft that
contradicts the rule-base is refused exactly like a hand-written one.

The behaviour observed is itself the argument. Asked to draft a Mudarabah from an English
paragraph, the model's first attempt invented an unsupported ratio literal in an invariant
body; the formal gate **refused** it on a parse error. With the grammar tightened, the second
draft passed the gate, lowered to `MudarabahInvestmentGen.sol`, compiled, and passed its
generated tests. The gate, not the model, is the authority. The front-end is non-deterministic
and explicitly *not* part of the load-bearing guarantee; it is a bridge toward a future
natural-language layer.

## 8. Epistemics and honest limitations

1. **No fatwa.** The compiler proves consistency with a declared rule-base; it does not certify
   *halal*. The rule-base and the instruments (the diminishing partnership carries *khilaf*)
   require a qualified scholar's *takhrij*. All citations are flagged `[scholar-verify]`.
2. **Orchestrated toolchain.** Solidity compilation and Hedera deployment are delegated to
   proven tools; this is deliberate scope, not the contribution.
3. **Testnet, and a depressed hbar price.** Work is testnet-only inside an isolated container.
   A live deploy costs ~10–35 ℏ at the current exchange rate; this is an operational, not a
   design, constraint.
4. **The NL layer is non-deterministic** and subordinate to the formal gate.
5. **Legal title and adoption** remain institutional, not code-reachable — as the prior stage
   already concluded.

## 9. Relation to the four fault lines, and future work

The four field fault lines — form-over-substance, the risk-sharing paradox,
immutability-vs-flexibility, and the oracle/gharar boundary — were each addressed in code by
the prior artifact. `fiqhc` raises the same discipline to the level of a *language*, so that the
refusal of form-over-substance and the enforcement of risk-sharing are not properties a
reviewer must check per contract, but properties the compiler guarantees for every contract it
emits. Future directions map onto the remaining expansion visions: a richer rule-base and
multi-madhhab parameterisation; a zero-trust valuation layer to shrink the residual gharar; and
a backend abstracted across ledgers. The institutional gate — scholarly ratification of the
rule-base, legal-title recognition, adoption — a compiler cannot cross; it can only make the
conditions auditable, and now, unrepresentable when violated.

## 10. Vision #2 — a zero-trust valuation oracle: gharar as a computed quantity

The prior artifact, and §8 above, named the single trusted valuer as the residual gharar locus.
We now report a direct attack on it and — keeping faith with the primitive — fold it into the
compiler.

**The contract.** `ConsensusValuationOracle` implements the same `IValuationOracle` interface,
so it drops into every generated instrument unchanged. It registers a committee of independent
attestors; each attestor SIGNS `(round, value, oracle, chainId)`, and the oracle recovers the
signer cryptographically (`ecrecover`) and checks committee membership — so the relayer who
submits a batch is trusted with nothing. The fair value is the *median* of the attestations
that fall within an agreed dispersion band; values outside it are rejected as outliers.

**Gharar as a computed quantity.** A price is usable only if it is *ma'lum* (determinable). We
make this executable: if fewer than `quorum` independent attestors agree within
`ghararBoundBps` of the median, the value is *majhul* — that is gharar — and `fairValue()`
reverts. The gharar boundary ceases to be a hidden assumption and becomes a quantity computed
on-chain from the dispersion of independent attestations. A contract cannot transact on an
undeterminable value.

**The DSL declares it.** An instrument now declares its valuation regime:
`oracle { mode: consensus; committee: 5; quorum: 3; gharar_bound_bps: 500; }`. The compiler
validates the parameters (quorum ≤ committee; bound ∈ (0,10000)) and wires the consensus oracle
into the deploy descriptor — it emits not only the contract but its zero-trust valuation layer.

**Autonomous fact-finding.** Off-chain, a committee of autonomous agents performs the
fact-finding. Each agent reasons independently over its OWN market evidence via an LLM
(DeepSeek), produces a fair-value estimate, and signs it; no agent sees the others, and
consensus emerges on-chain. In a local end-to-end run, five agents on a convergent scenario
resolved a consensus median of 1,000,000, off which a generated `MusharakahConsensus` ran its
full lifecycle (ownership 80% → 60% at the consensus price); a divergent scenario produced a
spread the band could not contain, and the oracle correctly refused a value (*majhul*). The
on-chain logic is proven by Hardhat tests (median, outlier rejection, signature verification,
quorum, sequential rounds, and the gharar gate). The live testnet run awaits a faucet top-up;
the path is one command through the same generic runner.

This shifts the trust boundary from one valuer's word to the *agreement* of an independent
committee, and turns the residual gharar from an admitted assumption into an enforced,
measurable condition. What remains a governance choice — who sits on the committee, and the
band's width — is now explicit and auditable, which is the most an engine can honestly offer; a
qualified scholar's ratification of those choices remains, as ever, theirs. *Allahu a'lam.*

## 11. Vision #3 — beyond Islamic finance: universality, and a code-based judiciary

Form-over-substance, and the tension between rigidity and flexibility, are not monopolies of
Islamic finance; common-law commerce has its own doctrines against unjust terms. We tested the
claim that the SAME machinery — declare a rule-base R, refuse specifications inconsistent with
R, emit the enforcing contract — applies across legal regimes. We added a `common_law` regime
and a `commercial_escrow` class. Its rule-base parallels the Islamic one with a striking
symmetry: **certainty of terms ↔ gharar**; the **penalty doctrine** (damages must be a genuine
pre-estimate, not a penalty *in terrorem*) **↔ riba's** prohibition of unjust excess;
**consideration ↔ ʿiwaḍ**; good faith. The compiler refuses a penalty/indefinite escrow exactly
as it refuses a disguised riba (codes `PENALTY-1`, `CERTAINTY-1`, `DISPUTE-1`), citing the
leading authorities (*Cavendish v Makdessi* [2015]; *Scammell v Ouston* [1941]) — flagged
`[verify]` for a lawyer, as the Islamic citations are flagged for a scholar. A declared regime
that contradicts the class is itself refused (`REGIME-1`).

The generated `CommercialEscrowGen` carries a regime-NEUTRAL **code-based judiciary engine**: a
deposit is held, released on a definite condition, and — should a dispute arise — adjudicated by
an arbiter whose ruling either releases to the beneficiary or refunds the depositor. This is the
same machinery that encodes *khiyar al-ʿayb* and *faskh*, abstracted from any one tradition: a
prototype for the automated dispute resolution an arbitration tribunal might adopt. Five
generated tests prove release-on-condition, the arbiter's ruling either way, that only the
arbiter may rule, and that release is barred while a dispute is open.

## 12. Vision #4 — a ledger-agnostic backend: legal invariants, injected in real time

The validation layer need not be bound to Solidity or to one chain. The compiler now emits,
beside the contract, a **portable invariant manifest** (`fiqhc build --target manifest`): the
same facts the engine checks, rendered as machine-checkable constraints
`{code, field, op, value, citation}`. The Solidity target serves EVM ledgers (Ethereum, Hedera,
Polygon); the manifest serves everything else — a non-EVM ledger, or a traditional enterprise
database.

An **invariant gateway** microservice consumes the manifest and injects the rule-base into any
backend in real time. Before a contract's terms are committed, a system POSTs them to
`/enforce`; a compliant transition is allowed, a non-compliant one is refused with the cited
rule — exactly as the compiler would refuse it. We demonstrated this live: a compliant musharakah
term-set was allowed; a disguised loan (`capital_guarantee: bank`, `loss: none`, rent on
principal, self-named price) was refused with four cited violations; a common-law escrow bearing
a penalty clause was refused with three. A `/compile` endpoint exposes the whole pipeline as a
service — a spec in, a fiqh-cited verdict out. Compliance by construction thus reaches beyond the
chain: it becomes a gate any ledger or database can place in front of its writes.

Taken together, the four expansions carry one idea to its conclusion. *Compliance by
construction* began as a property of one contract; it is now a property of a **language**
(Vision #1), defended by a **zero-trust measurement of the very uncertainty it once assumed**
(Vision #2), shown **universal across legal traditions** with a regime-neutral judiciary
(Vision #3), and **injectable into any infrastructure** (Vision #4). What no engine can supply —
the scholar's ratification of the rule-base, the law's recognition of title, a community's
adoption — remains, honestly, with people. *Allahu a'lam.*

## References

- Vaswani, A., et al. (2017). *Attention Is All You Need.* NeurIPS.
- Hevner, A. R., March, S. T., Park, J., & Ram, S. (2004). *Design Science in Information
  Systems Research.* MIS Quarterly, 28(1), 75–105.
- Peffers, K., Tuunanen, T., Rothenberger, M. A., & Chatterjee, S. (2007). *A Design Science
  Research Methodology for Information Systems Research.* JMIS, 24(3), 45–77.
  doi:10.2753/MIS0742-1222240302
- AAOIFI. *Shari'ah Standard No. 12 (Musharakah)*, *No. 13 (Mudarabah)*, *No. 9 (Ijarah)*.
  [scholar-verify]
- OIC International Islamic Fiqh Academy, Resolution 179 (19/5), Sharjah, April 2009.
- Qur'an, al-Baqarah 2:275 (prohibition of riba) [scholar-verify]; the prohibition of *gharar*
  (Sahih Muslim, Kitab al-Buyu') [scholar-verify].
- *Dunlop Pneumatic Tyre v New Garage* [1915] AC 79; *Cavendish Square v Makdessi* [2015]
  UKSC 67 (penalty doctrine) [verify].
- *Scammell & Nephew v Ouston* [1941] AC 251 (certainty of terms) [verify]; *Currie v Misa*
  (1875) LR 10 Ex 153 (consideration) [verify]; *Yam Seng v ITC* [2013] EWHC 111 (good faith);
  UCC §1-304 [verify].
