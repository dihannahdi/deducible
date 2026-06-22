# Re-Review (Round 3) — Consolidated Manuscript

Mode: full panel re-review on `full_paper_v3.md` (adds tokenized ownership, HTS-native atomic
buyout, khiyar/iqalah rescission). Read-only. Builds on Round-2 (both CRITICALs resolved).

## What changed since Round 2

| Addition | Evidence | Effect |
|----------|----------|--------|
| Real tokenized ownership (HTS) | token live; 8000/2000 split | ownership is a transferable token with provenance, not a counter |
| Contract moves the asset token | custodian `0.0.9304674` via precompile `0x167` | the enforcement contract custodies/moves the real asset |
| HTS-native **atomic buyout** (V3) | `0.0.9304707`: 2000 units + hbar in one tx | ownership transfer and payment are inseparable |
| **khiyar + iqalah** rescission (V4) | 15/15 tests | the 4th fault line (immutability-vs-flexibility) now has a mechanism |

## Panel assessment

- **EIC.** The paper now lands all four fault lines it set out, with on-chain or test evidence
  for each. Apparatus is complete (abstract, keywords, references, figures). Novelty is high and
  the scope is honestly bounded. Closer to acceptable; not yet a clean accept (below).
- **Methodology (R1).** Substantially stronger: 15 unit tests, multi-account on-chain runs,
  atomic-buyout and settlement proofs, solhint clean. Still missing: formal/Slither verification;
  the HTS layer is testnet-verified only (acknowledged); no comparison baseline or field study.
- **Domain (R2).** Citations verified; compliance reframed (no fatwa; MMP *khilaf* noted). The
  *khiyar*/*iqalah* encoding is a genuine contribution to the immutability debate — but its
  *fiqh* adequacy is correctly left to scholars, and that scholarly review has not occurred.
- **Perspective (R3).** Tokenization narrows the real-asset gap to its true residue: legal title
  and registry recognition, now clearly stated as the binding external dependency. Valuation cost
  remains the dominant real-world cost, disclosed.
- **Devil's Advocate.** The Round-1 CRITICAL (enforcement = bookkeeping) is dead. The standing
  challenge is now honest and narrow: real-world facts still enter via a trusted valuer; on-chain
  title is not legal title; rescission unwinds only pre-performance. These are scoped limitations,
  not unsupported claims. **No CRITICAL.**

## Scores (vs Round 1)

| Dimension | R1 | R3 |
|-----------|----|----|
| Originality | 80 | 84 |
| Methodological rigor | 42 | 66 |
| Evidence sufficiency | 44 | 68 |
| Argument coherence | 63 | 72 |
| Writing quality | 70 | 76 |
| **Weighted** | **~57** | **~72** |

## Decision

**MINOR REVISION (approaching accept-with-minor); no CRITICALs.** The central
*compliance-by-construction* claim is supported, all four fault lines have working mechanisms
verified in test or on-chain, and the artifact is honestly scoped.

**Residual (non-blocking) work for a clean accept:** (1) legal-title/registry bridge — the true
remaining substance gap (institutional, not code); (2) qualified-scholar review of the *fiqh*
(institutional, not code); (3) post-performance rescission unwinding; (4) a unified HTS-native
loss-settlement path; (5) formal/Slither verification; (6) a comparison baseline or field study;
(7) multi-instrument generalization. Items (1)-(2) are the gate to "revolutionary" in the full
sense and lie beyond what code alone can deliver.
