# SyariahChain — Compliance by Construction

> Enforcing the *contractual conditions* of a Shari'ah-compliant **Musharakah Mutanaqisah**
> (diminishing partnership) on a public ledger — so compliance is *enforced by execution*,
> not merely *attested after the fact*.

This is an open, reproducible proof-of-concept and an accompanying academic paper. It is a
*movement* invitation: to build Islamic-economic instruments whose substance a machine can
verify continuously, and to submit that work openly to scholars, engineers, and regulators.

---

## The idea, in one breath

Most "blockchain for Islamic finance" work makes blockchain the protagonist and the *Shari'ah*
a use-case. **We invert it.** The Shari'ah economy — real ownership, shared risk, the just
circulation of wealth — is the substantive proposal; the ledger is simply the first execution
substrate able to run it *faithfully*. The recurring critique of Islamic finance is
*form over substance*: a board certifies an instrument once, and afterward no one can
continuously verify that its substance still matches its form. Our answer is
**compliance by construction** — encode the conditions that distinguish a genuine
Shari'ah-compliant instrument from disguised debt as **invariants a smart contract enforces at
execution**, so a violating transaction simply does not execute.

## ⚠️ Honest status & disclaimers (please read)

- **Proof-of-concept, testnet only.** Nothing here is deployed to mainnet or handles real value.
- **No fatwa is claimed.** This project makes an *engineering* claim, not a *juristic* one.
  Whether these encodings — and the diminishing partnership itself, which carries *khilāf* —
  are *ḥalāl* is a ruling for **qualified scholars**. See [`ISTIFTA.md`](./ISTIFTA.md) for the
  questions we are bringing to a Shari'ah board.
- **Not financial or religious advice.**
- The asset, the contracting parties, and **legal title** are abstracted in this PoC; a real
  instrument needs a real leasable asset, real parties, and a recognised registry bridge.

## The four fault lines, each given a working mechanism

| Fault line | Mechanism in this repo |
|---|---|
| Form over substance | conditions enforced as contract invariants (rent on living share, oracle-priced buyout, role separation) |
| The risk-sharing paradox | capital custody + `settle()` paying current value by transfer — a loss provably reduces the financier's recovery |
| Immutability vs. *fiqh* flexibility | *khiyar al-shart*, *iqālah*, *khiyar al-'ayb* (arbiter-adjudicated), and authoritative *faskh* encoded |
| Oracle / *gharar* | valuation externalised to an independent, access-controlled valuer (relocated and made explicit, not abolished) |

## Verified on Hedera testnet (verify it yourself)

| What | Contract / token | Explorer |
|---|---|---|
| Capital-custody settlement (loss → financier recovers less) | `0.0.9304241` | https://hashscan.io/testnet/contract/0.0.9304241 |
| HTS-native **atomic buyout** (units + payment in one tx) | `0.0.9304707` | https://hashscan.io/testnet/contract/0.0.9304707 |
| Loss + clean dissolution | `0.0.9304884` | https://hashscan.io/testnet/contract/0.0.9304884 |
| Tokenized ownership (HTS asset-share token) | `0.0.9304628` | https://hashscan.io/testnet/token/0.0.9304628 |
| Contract custodies & moves the real asset token | `0.0.9304674` | https://hashscan.io/testnet/contract/0.0.9304674 |

**Quality gates:** 20/20 unit tests passing · `solhint` 0 errors · `slither` 0 security findings.
Machine-readable run records are in [`deployments/`](./deployments).

## How this compares

| Dimension | Conventional MMP (board-attested) | Typical blockchain-IF (transparency layer) | This work (compliance by construction) |
|---|---|---|---|
| Compliance decision | ex-post human attestation | ledger records; board still attests | **enforced by execution** |
| Loss-sharing | manual settlement | recorded | **enforced by transfer** |
| Monitoring cost | high (human audit) | medium | low (machine-checked state) |
| On-chain rescission | n/a | usually none | *khiyar* / *iqālah* / defect / *faskh* |
| Asset representation | registry only | token (provenance) | token **custodied by the contract** |

## Reproduce it

Prerequisites: Node.js 20+ (tested on 22), npm. (Docker optional.)

```bash
git clone https://github.com/dihannahdi/shariachain.git
cd shariachain
npm install
npx hardhat compile
npx hardhat test          # 20/20 should pass
```

To deploy/exercise on **Hedera testnet**, create a free testnet account at
[portal.hedera.com](https://portal.hedera.com), copy `.env.example` to `.env`, fill in your own
credentials (**testnet only**), then:

```bash
node scripts/deploy.js          # deploy + read live state
node scripts/interact.js        # full lifecycle + gas/fee capture
node scripts/interact3.js       # 3-account adversarial + settlement
node scripts/interact6.js       # HTS-native atomic buyout (V3)
```

Build the paper (requires `pandoc`):
```bash
pandoc paper/full_paper_v3.md -o paper.docx
```

## Repository layout

```
contracts/    Solidity: MusharakahMutanaqisah (V1) → V2 (capital custody) →
              V3 (HTS-native) → V4 (khiyar/iqalah) → V5 (defect/faskh/maslahah);
              IValuationOracle, MockValuationOracle, AssetTokenCustodian
test/         Hardhat test suites (20 tests)
scripts/      deploy + on-chain interaction + diagnostics
paper/        the manuscript (markdown), figures, peer-review rounds, security report
deployments/  machine-readable testnet run records (contract IDs, state, gas/fees)
STATE_OF_ARTIFACT.md   honest ledger: what code has proven vs. what awaits institutions
ISTIFTA.md    the questions for a qualified Shari'ah board
```

## The road ahead (and what code cannot do)

Code has carried this as far as code can. What remains to make it *real* is **institutional**,
not technical:

1. A **qualified scholar's ruling** (*fatwa*) — including verification (*takhrīj*) of any
   evidences, which must be checked against primary sources by a human.
2. **Legal recognition** of on-chain title via a real registry bridge.
3. **Adoption** by an institution willing to use it.

Contributions toward any of these — code, scholarship, or law — are welcome.

## License

MIT — see [`LICENSE`](./LICENSE). *Allahu a'lam.*
