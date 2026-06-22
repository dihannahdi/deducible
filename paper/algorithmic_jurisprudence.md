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
