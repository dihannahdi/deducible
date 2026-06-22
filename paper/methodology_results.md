# Methodology and Results

> Draft sections for the working paper. Citations are flagged `[verify]` where the
> exact reference (standard number, ayah, hadith collection) must be confirmed before
> submission — fabricating an attribution is not permissible.

## 3. Methodology

### 3.1 Research paradigm

This study adopts **Design Science Research (DSR)**, in which knowledge is produced by
constructing and rigorously evaluating a novel artifact rather than by observing an
existing phenomenon (Hevner, March, Park, & Ram, 2004 `[verify]`; Peffers et al., 2007
`[verify]`). DSR is the appropriate paradigm because the contribution is not a claim
*about* the world but a claim that a *new kind of instrument can exist*: a financial
contract whose Shari'ah compliance is enforced by its own execution rather than
attested after the fact. We follow Hevner's three-cycle view — the relevance cycle
(the field's unresolved tension), the design cycle (build and test the artifact), and
the rigor cycle (ground the design in both *fiqh al-mu'amalat* and software
verification practice).

### 3.2 Problem statement and design objective

The recurring critique of contemporary Islamic finance is that compliance lives as
*form* — a contract certified once by a Shari'ah board — while the underlying economic
*substance* drifts toward the interest-based finance the *Shari'ah* forbids (Qur'an,
al-Baqarah 2:275 `[verify]`). The same root explains why genuine risk-sharing contracts
(*musharakah*, *mudarabah*) are marginal in practice: the cost of verifying a partner's
true profit or loss is prohibitive, so institutions retreat to debt-like *murabaha* and
organized *tawarruq* (the latter ruled impermissible by the OIC International Islamic
Fiqh Academy, Resolution 179, Sharjah 2009 `[verify]`).

We therefore state a single design objective: **move Shari'ah compliance from ex-post
human attestation to ex-ante, protocol-enforced economic substance** — *compliance by
construction*. We test this objective on the contract that sits precisely at the
intersection of the form-over-substance and risk-sharing fault lines: *Musharakah
Mutanaqisah* (diminishing partnership), the instrument that underlies much real Islamic
asset and home financing.

### 3.3 The artifact and its enforced invariants

The artifact is a set of three Solidity contracts deployed on Hedera: an independent
valuation oracle interface (`IValuationOracle`), a reference oracle
(`MockValuationOracle`, access-controlled to a single independent valuer), and the
partnership contract (`MusharakahMutanaqisah`). Four properties that *fiqh* requires of
a genuine diminishing partnership — and that critics allege are routinely violated — are
encoded as **invariants the contract enforces by construction**:

| # | Enforced invariant | *Fiqh* basis |
|---|--------------------|--------------|
| I1 | Rent (*ijarah*) is charged only on the financier's **current** ownership share, so it falls automatically as ownership transfers to the client | rent must attach to a genuinely co-owned asset; AAOIFI Shari'ah Standard on Musharakah, No. 12 `[verify]` |
| I2 | A buyout is priced from an **independent attested valuation**, never a pre-fixed schedule; the buyer cannot name their own price | prohibition of a guaranteed-capital return disguised as partnership (*riba*); al-Baqarah 2:275 `[verify]` |
| I3 | A fall in asset value is written down by the **current ownership ratio**; the financier cannot exit whole | risk-sharing is the essence of *musharakah*; loss follows capital |
| I4 | Roles are enforced — only the lessee pays rent, only an independent valuer attests value, neither partner self-reports | separation of the lease and sale undertakings |

The valuation oracle is deliberately **external and access-controlled**. This does not
abolish trust; it **relocates** it from an annual board attestation to a continuously
auditable, role-restricted valuer, and makes that trust boundary explicit. This is our
honest answer to the *gharar* (excessive uncertainty) concern — the Prophet (peace be
upon him) forbade *bay' al-gharar* (reported in Sahih Muslim `[verify]`) — and it is
revisited as a limitation in the Discussion.

### 3.4 Evaluation strategy

The artifact is evaluated at three levels of increasing externality:

1. **Functional correctness (local).** A unit-test suite executes each invariant on a
   local EVM, including adversarial cases (mispriced buyout, role violation, terminal
   acquisition).
2. **On-chain demonstration (testnet).** The contracts are deployed to the Hedera
   testnet and driven through a full lifecycle — rent payment, share buyout, downward
   revaluation, loss synchronisation — with state read back from the network after each
   step.
3. **Cost (testnet).** Gas consumed and fees charged are recorded per operation, to
   assess industrial implementability.

### 3.5 Implementation environment

Contracts are written in Solidity 0.8.24 and compiled with Hardhat. Deployment and
interaction use the Hedera JavaScript SDK (`@hashgraph/sdk`) via `ContractCreateFlow`
and `ContractExecuteTransaction`; off-chain state is read with free `ContractCallQuery`
calls. The operator account is an ECDSA (`secp256k1`) account on the Hedera testnet.
A practical finding worth recording: on this network the value visible to a contract as
`msg.value` is denominated in **tinybar** (1 ℏ = 10⁸ tinybar), not in the Ethereum-style
weibar; the contract's monetary quantities were denominated accordingly (see §4.4).

## 4. Results

### 4.1 Functional correctness

All five invariant tests pass on the local EVM:

| Test | Property exercised | Result |
|------|--------------------|--------|
| I1 | rent falls as ownership transfers | pass |
| I2 | buyout off the attested price is rejected; the buyer cannot set the price | pass |
| I3 | loss shared by ownership ratio; financier cannot exit whole | pass |
| I4 | lessee-pays / valuer-attests roles enforced | pass |
| terminal | rent → 0 once the client owns the whole asset | pass |

### 4.2 On-chain demonstration

The contracts were deployed and exercised on the Hedera testnet
(oracle `0.0.9303337`, partnership `0.0.9303339`;
`https://hashscan.io/testnet/contract/0.0.9303339`). State read back from the network
after each operation:

| Stage | bank bps | client bps | asset value (tinybar) | rent due (tinybar) |
|-------|---------:|-----------:|----------------------:|-------------------:|
| initial | 8000 | 2000 | 100,000,000 | 80,000 |
| after `payRent` | 8000 | 2000 | 100,000,000 | 80,000 |
| after `buyShare` (2000 bps) | **6000** | **4000** | 100,000,000 | **60,000** |
| after downward `syncValuation` | 6000 | 4000 | **90,000,000** | 60,000 |

Two transitions carry the thesis. First, the single `buyShare` transaction moved 2000
basis points of ownership from financier to client **and** reduced the rent due from
80,000 to 60,000 tinybar in the same atomic step — the *ijarah* obligation diminishing
with ownership, enforced by the protocol, not promised in a footnote. Second, the
downward revaluation wrote the asset from 100,000,000 to 90,000,000 tinybar with the
10,000,000 loss apportioned by the live 60/40 ownership ratio — the financier bearing
6,000,000 and the client 4,000,000 — demonstrating that the financier cannot exit whole.

### 4.3 Cost

Gas and fees for each live operation:

| Operation | Gas used | Fee (ℏ) |
|-----------|---------:|--------:|
| `payRent` | 34,546 | 0.03696 |
| `buyShare` (2000 bps) | 49,599 | 0.05307 |
| `attest` (oracle, downward) | 27,908 | 0.02986 |
| `syncValuation` (loss) | 62,800 | 0.06720 |
| **Full lifecycle total** | **174,853** | **0.18709** |

The complete lifecycle — rent, buyout, revaluation, loss-sharing — cost **0.187 ℏ**.
Because Hedera fees are USD-pegged and predictable by design, each state-changing
operation costs on the order of a few US cents `[verify exact USD via current HBAR
price]`. For an instrument whose conventional analogue carries monthly servicing,
custody, and audit overheads, this is firmly in the range of industrial viability.

### 4.4 An honest finding: representing real money on-chain

The first lifecycle attempt reverted because the contract's monetary quantities were
denominated in weibar while the network presents `msg.value` in tinybar — a one-line
mismatch that nonetheless exposes a genuine design concern: the granularity of on-chain
value (whole tinybars) constrains how real-world prices and rents can be represented.
For high-value, low-divisibility assets this is harmless; for micro-rental schedules it
imposes rounding that must be designed for explicitly. We surface this rather than hide
it, as it bears directly on any production deployment.

### 4.5 Limitation carried forward

The valuation oracle is the load-bearing trust assumption. The artifact enforces that no
*contracting partner* can set the value, but the integrity of the *independent valuer*
remains an external guarantee — the relocated, not eliminated, locus of *gharar*. The
Discussion treats this as the central boundary of the compliance-by-construction claim.
