# Re-Review (Round 2) — Verification of Revisions

Mode: re-review (verification). Checks whether the Round-1 issues are addressed in the
revised manuscript (`full_paper_v2.md`) and the new on-chain evidence (`deployments/testnet_v2.json`,
`MusharakahMutanaqisahV2.sol`, `paper/security_static_analysis.md`). Read-only.

## Traceability matrix

| Round-1 issue | Severity | Author's change | Verified? |
|---------------|----------|-----------------|-----------|
| Loss-sharing emitted, not settled; "cannot exit whole" unsupported | **CRITICAL** | V2 escrows both partners' capital; `settle()` pays each its share of the oracle's *current* value from the pool. On testnet, after −10% the financier recovered 72,000,000 of 80,000,000 funded — bore 8,000,000 by transfer. | **RESOLVED** — verified on-chain (contract 0.0.9304241) and by unit test |
| Independence/anti-self-dealing untested (single account) | **CRITICAL** | Lifecycle re-run with three distinct funded accounts; client-attests, valuer-settles, bank-fundClient all revert on-chain. | **RESOLVED** — verified on testnet |
| All citations placeholders | MAJOR | Hevner 2004, Peffers 2007, AAOIFI SS No. 12, OIC Res. 179 (19/5) verified; El-Gamal 2006, Usmani 2007 named; References section added. Scriptural refs cited by surah:ayah / collection, flagged `[scholar-verify]`. | **RESOLVED for academic/standards**; scriptural verification honestly deferred to a qualified scholar (no fabrication) |
| Compliance asserted, not adjudicated | MAJOR | Title softened to "…Contractual Conditions of…"; explicit "no fatwa is claimed"; MMP *khilaf* acknowledged. | **RESOLVED** |
| No security audit | MAJOR | `nonReentrant` guards + checks-effects-interactions; solhint 0 errors, no reentrancy/unchecked-call findings. | **PARTIALLY RESOLVED** — solhint clean; deeper Slither pass still future work |
| Valuation cost excluded from "industrial viability" | MAJOR | §5 now states professional valuation is periodic, costly, and would dominate total cost (excluded from on-chain figures). | **RESOLVED (disclosed)** |
| On-chain ownership has no legal title; no real-asset binding | MAJOR | §5 states ownership is a bps counter, not legal title; tokenized-asset (HTS) + registry integration named as future work. | **ACKNOWLEDGED, not built** — remains an honest open limitation |
| Generalization overreach ("the Shari'ah economy") | MINOR | Claims scoped to "for one instrument"; abstract and §1 tempered. | **RESOLVED** |
| Missing Abstract/keywords/References/figure call-outs | MAJOR (apparatus) | All added; Figures 1–2 referenced in §4.3. | **RESOLVED** |

## Residual issues (non-blocking)

- **Real-asset binding + legal title** (MAJOR, acknowledged): the connection from on-chain
  state to a legally recognised, real co-owned asset is discussed but unbuilt. For a venue
  this is acceptable as scoped future work; for a production claim it is the next gate.
- **Deeper security verification** (MINOR): Slither/formal analysis beyond solhint.
- **Scriptural citation finalisation** (MINOR): requires a qualified scholar; correctly not
  self-certified.
- **`khiyar`/`iqalah` vs finality** (MAJOR, theoretical): named as open future work.

## Devil's Advocate re-check

The Round-1 CRITICAL counter-argument — that "enforcement" was bookkeeping because loss was
a printed number and roles were untested — **no longer holds**: loss now moves real value
between separate accounts on a public network, and role separation is demonstrated
adversarially. The standing valid challenge is narrower and honestly owned by the authors:
the binding *real-world* facts (asset existence, valuation) still enter through a trusted
valuer, and on-chain ownership is not legal title. This is a boundary of scope, not an
unsupported claim. **No CRITICAL remains.**

## Decision

**MINOR REVISION** (no CRITICALs). The two gating CRITICALs are resolved and independently
verified on-chain; the central "compliance by construction" claim is now supported by the
evidence for the properties the protocol can verify. Remaining items are scoped limitations
and polish, appropriate to disclose rather than block.

**Loop success condition met: re-review returns no CRITICALs.**
