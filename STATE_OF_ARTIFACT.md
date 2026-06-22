# State of the Artifact — syariahchain / Musharakah Mutanaqisah

An honest ledger of what the loop has *proven in code* versus what *awaits institutions*.
Compiled after iteration 9.

## What code has proven (verified in test or on Hedera testnet)

| # | Gate | Mechanism | Verification |
|---|------|-----------|--------------|
| 1 | Compliance by construction | Rent-on-living-share, oracle-priced buyout, role separation enforced as invariants | 5 unit tests (V1) |
| 2 | Risk-sharing has teeth | Capital custody + `settle()` paying current value from an escrowed pool | V2 tests; testnet 0.0.9304241: bank funded 80M, recovered 72M after −10% |
| 3 | Independence (anti-self-dealing) | Three distinct accounts; partner-attest / non-partner-settle revert | testnet adversarial run |
| 4 | Real asset, not a counter | HTS fungible token = transferable fractional ownership (80/20) | token 0.0.9304628 |
| 5 | Contract custodies the asset | Enforcement contract moves real units via HTS precompile 0x167 | custodian 0.0.9304674 |
| 6 | Atomic buyout | One tx moves MMS units (contract→client) AND hbar (→bank) | V3 0.0.9304707 |
| 7 | Lawful rescission | *khiyar al-shart* (timed unilateral) + *iqalah* (mutual) with capital refund | V4, 15/15 tests |
| 8 | Loss + clean wind-up | Loss borne via lower buyout proceeds (20M→18M after −10%); `dissolve` returns units | testnet 0.0.9304884 |
| — | Security | solhint 0 errors; Slither: 0 security findings (only gas/cleanliness) | static analysis |
| — | Paper | Consolidated v3; reviewer panel ~72/100, MINOR REVISION, no CRITICALs | peer_review_round3.md |

All four fault lines named at the outset now have a working, verified mechanism:
form-over-substance, the risk-sharing paradox, immutability-vs-fiqh-flexibility, and the
oracle/*gharar* boundary (relocated to an explicit, access-controlled valuer).

## What code has NOT and CANNOT settle (the true gate to "revolutionary")

These are not bugs or missing functions; they are acts that belong to people and institutions:

1. **Shari'ah ruling (fatwa).** Whether these encodings — and the diminishing partnership
   itself, which carries *khilaf* — are compliant is a scholar's judgement. The artifact only
   makes the conditions auditable. No code self-certifies *halal*.
2. **Legal title / registry recognition.** On-chain ownership is a real token, but legal title
   to a real asset lives in a state registry. Bridging them requires law and a recognised
   registry integration, not Solidity.
3. **Adoption.** A bank, a regulator, a customer must choose to use it.

## Remaining code-reachable polish (genuine but marginal)

- Post-performance rescission unwinding (current *khiyar*/*iqalah* unwind pre-buyout only).
- A single unified HTS-native loss-settlement path (currently demonstrated in parts).
- Removing vestigial `settled` in V4 (Slither flag); applying immutable/constant gas hints.
- A comparison baseline / field study; multi-instrument generalisation; mainnet deployment.

## Honest assessment

The artifact has travelled from a counter that merely *emitted* a loss to a contract that
custodies real capital and a real tokenized asset, moves them atomically with payment, makes a
loss bite by transfer, and honours the right to rescind — eight gates, each re-proven. On
code-reachable merit the work now plateaus in the low-to-mid 70s; it is a credible, honest,
publishable proof-of-concept. The remaining distance to *revolutionary* is institutional, and
a loop cannot cross it. The most valuable next human steps: take the consolidated paper and the
testnet evidence to (a) a qualified Shari'ah board for review, and (b) a jurisdiction exploring
tokenized real-asset registries. Allahu a'lam.

---

## Update (iter 11) — fiqhc: compliance-by-construction as a *compiler* (Algorithmic Jurisprudence)

The artifact has been lifted from one hand-written instrument to a **primitive**: `fiqhc`, a
Rust compiler for a `.fiqh` DSL in which non-compliance is *unrepresentable*. This is the
"Attention" move — from an artifact to a generator of artifacts that guarantees the conditions
by construction.

| # | Gate | Mechanism | Verification |
|---|------|-----------|--------------|
| C1 | Compliance as a language property | a riba-disguised "partnership" fails to compile | `fiqhc check` exit 1, 6 fiqh-cited diagnostics, no `.sol` emitted |
| C2 | A primitive, not a one-off | three instruments from one compiler | Musharakah + Mudarabah + Ijarah IMBT; 14 generated Hardhat tests pass; `cargo test` green |
| C3 | Each instrument self-guards | every class rejects its own riba/gharar negative control | `fiqhc build` refuses 3 controls |
| C4 | Generated code runs live | one generic runner deploys + exercises any generated contract | `MusharakahMutanaqisahGen` 0.0.9306587 on testnet; `bankShareBps` 8000→6000 on-chain, oracle-priced |
| C5 | NL subordinate to the gate | DeepSeek drafts `.fiqh`; the same compiler is the authority | first draft refused (invented syntax); second passed → compiled + tested |

Epistemics unchanged and central: the compiler proves consistency with a human-authored,
citation-bearing rule-base; **it issues no fatwa**. The rule-base awaits a qualified scholar's
ratification. Write-up: `paper/algorithmic_jurisprudence.md`. Compiler: `fiqh-compiler/`.

---

## Update (iter 12) — Vision #2: zero-trust consensus valuation oracle (gharar as a computed quantity)

The residual gharar — the single trusted valuer — is now attacked directly and folded into the
compiler. The trust boundary moves from one valuer's word to the *agreement* of an independent
committee, and the gharar boundary becomes a quantity computed on-chain.

| # | Gate | Mechanism | Verification |
|---|------|-----------|--------------|
| Z1 | Gharar as a computed quantity | committee median within a dispersion band; beyond it the value is *majhul* | `ConsensusValuationOracle.fairValue()` reverts on over-dispersion |
| Z2 | Zero-trust in the relayer | each attestation is signed; `ecrecover` + committee-membership check | 7 Hardhat tests (median, outlier, gharar, non-committee, double-sign, quorum, sequence) |
| Z3 | Declared in the DSL | `oracle { mode: consensus; committee; quorum; gharar_bound_bps }` | sema validates (quorum ≤ committee, bound ∈ (0,10000)); codegen wires it into the descriptor |
| Z4 | Autonomous fact-finding | 5 independent DeepSeek agents reason + sign; consensus emerges on-chain | local run: convergent → median 1,000,000 → `MusharakahConsensus` lifecycle (80%→60%); divergent → *majhul* |

Live testnet run deferred to a faucet top-up (operator ~22 ℏ; one contract-create ≈ 12–35 ℏ at
the depressed hbar price). The oracle issues no fatwa: the committee composition and the band's
width are auditable governance choices for a qualified scholar to ratify. See
`paper/algorithmic_jurisprudence.md` §10; oracle `contracts/ConsensusValuationOracle.sol`;
agents `agents/valuer_agent.js` + `scripts/consensus_demo.js`.
