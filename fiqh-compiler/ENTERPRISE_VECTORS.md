# `fiqhc` — Enterprise Vectors

> Five advanced engineering vectors that carry `fiqhc` from a solid open-core compiler toward
> infrastructure for the *whole* of fiqh al-muʿamalat — the problems faced by central banks,
> multinationals, and the messy real world.
>
> **Epistemic boundary (unchanged).** The engine proves a specification is *consistent or
> inconsistent with a declared, human-authored, citation-bearing rule-base*. It issues **no
> fatwa**. Every fiqh citation is flagged `[scholar-verify]`; a qualified scholar must ratify the
> rule-base, and any actionable matter belongs to a qualified local scholar. *Allahu aʿlam.*

Each vector is implemented in Rust (the contribution), verified by `cargo test`, and — where it
touches a contract — proven on a local-EVM Hardhat run. Server commits `7213c6d → d476509`.

---

## Vector 2 — Graph-based invariant checker for composite contracts (al-ʿuqud al-murakkabah)

A single leg may be impeccable; their *composition* can still encode a ruse. A new top-level
`bundle` declares legs (directed asset/cash flows between parties); `src/composite.rs` builds the
directed asset-flow graph, enumerates its simple cycles, and refuses riba-by-composition no
single-contract check can see.

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
| `INAH-1` | **bay' al-ʿinah** — a 2-cycle on the same asset back to its origin, with a deferred leg + price differential (the time/price gap is the riba) | Abu Dawud (hadith of ʿinah); majority via sadd al-dharaʾiʿ `[scholar-verify]` |
| `INAH-2` | **organized tawarruq** — a ≥3-leg ring returning the asset to the financier, **and** the monetization flip (deferred-in + spot-out of the same asset) | OIC Fiqh Academy Res. 179 (19/5), 2009; AAOIFI SS No. 30 `[scholar-verify]` |
| `INAH-3` | (warning) a round-trip with no net economic transfer — a formalistic red flag (hila) | — |

The crucial contrast: the **same** deferred markup that is riba in the ʿinah cycle is licit in an
acyclic `wakalah + murabahah` composite (the customer keeps the asset). The engine forbids the
**cycle**, not the markup. `fiqhc check|build <bundle.fiqh>`; `build` emits a composite manifest.

## Vector 5 — Corporate Zakat al-Tijarah (algorithmic zakat at the AST)

A `zakat { }` section lifts corporate zakat from a year-end act of conscience into a property of
the contract. `src/zakat.rs` computes rubʿ al-ʿushr (1/40 = 2.5%) exactly; the generated contract
routes it to the maslahah/zakat fund, on-chain, non-bypassably.

```
zakat {
  rate_bps: 250;            // 1/40 — the only valid trade-goods rate
  nisab: 8500000 tinybar;   // the 85g-gold / 595g-silver equivalent
  haul: hijri_year;         // lunar year — a solar haul under-collects
  beneficiary: maslahah;
}
```

`ZAKAT-1` rate must be 250 bps · `ZAKAT-2` haul must be lunar · `ZAKAT-3` positive nisab ·
`ZAKAT-4` beneficiary resolves to a party. Generated Solidity: `zakatDue(base)` (zero below
nisab) + `payZakat(base)` → routes 2.5% to `maslahahFund`. *al-Tawbah 9:103; Abu Dawud (Samurah
b. Jundub); AAOIFI SS No. 35 `[scholar-verify]`.* Proven: 6/6 Hardhat tests.

## Vector 4 — Lifecycle off-ramps: jaaʾihah abatement + faraid dissolution

A `contingency { }` section declares the emergency exits the fiqh itself provides.

```
contingency {
  jaaihah: reschedule;   // calamity abates the obligation — never adds interest (wadʿ al-jawaʾih)
  death:   faraid;       // dissolution distributes by the fixed Qurʾanic shares
}
```

- `CONT-1` a jaaʾihah may only `reschedule`/`abate` (never a penalty). Generated `declareJaaihah()`
  drops `effectiveRentDue()` to zero; `rescheduleWithoutRiba()` can only extend a deadline.
- `CONT-2` a death may only invoke `faraid`. `src/faraid.rs` is a deterministic inheritance engine
  (spouse / parents / children, the ʿaṣaba 2:1 apportion, and the corrections ʿawl and
  radd-excluding-spouse), guarding the ʿUmariyyatayn case and unmodeled collaterals (it refers
  those to a faradi rather than guessing). Generated `dissolveByFaraid(heirs, shareBps)`
  distributes the deceased's escrowed capital by shares validated to total 10000 bps.

*Sahih Muslim (hadith of Jabir, wadʿ al-jawaʾih); al-Nisaʾ 4:11-12, 4:176 `[scholar-verify]`.*
Proven: 9 faraid unit tests vs textbook estates; 7/7 Hardhat tests.

## Vector 3 — Ahliyyah (legal capacity) + DID middleware

Fiqh is not only about a contract's content but about *who* contracts. The invariant gateway
(`services/invariant_gateway.js`) gains an ahliyyah layer (`services/ahliyyah.js`): it resolves
each party's DID to a credential and verifies **ahliyyat al-adaʾ** — capacity for execution —
before a compiled contract may execute.

```
POST /authorize { target, terms, parties: { financier: "did:fiqh:bank-alpha", acquirer: "did:fiqh:..." } }
  → allowed only if BOTH the invariant terms AND every party's capacity pass.
```

| code | refusal | citation |
|------|---------|-----------|
| `AHL-MINOR` | a minor (saghir) — lacks bulugh | ahliyyat al-adaʾ requires bulugh `[scholar-verify]` |
| `AHL-SAFIH` | the interdicted (safih/majnun) — lacks rushd/ʿaql | al-Baqarah 2:282; al-Nisaʾ 4:5 `[scholar-verify]` |
| `AHL-TAFLIS` | a bankrupt under hajr — estate reserved for creditors | Sahih Muslim `[scholar-verify]` |
| `AHL-KYC` / `AHL-AML` | statutory overlay | — |

The compiler emits `ahliyyah.principals` (the contracting roles) into the manifest so the gateway
knows whose capacity to require. Proven: module smoke 12/12; live `/authorize` 5/5.

## Vector 1 — Zero-Knowledge Fiqh

A global bank will not place its real loss figures on a public ledger; yet the invariant must
still be **proven, not asserted**.

- `src/zk.rs` — a self-contained sigma-protocol PoC (its own SHA-256, Miller-Rabin safe-prime
  search, Pedersen commitments, Schnorr/Fiat-Shamir) that proves the committed losses satisfy
  `lossBank·clientBps == lossClient·bankBps` — **[RISK-1]**, loss shared proportional to
  ownership — **without revealing the amounts**. It reduces the relation to proving
  `C_z = Cb^clientBps · Cc^(q−bankBps)` is a commitment to zero. *Honest scope: the protocol is
  real; the ~61-bit modulus is illustrative, not production.*
- `fiqhc build --target zk` — the **production** path: a Circom circuit (private loss witnesses;
  INV-1 + RISK-1 R1CS constraints), a Groth16 zk manifest (snarkjs pipeline), and a Solidity
  `settleWithProof()` gate that admits a settlement only on a valid proof of [RISK-1].

*AAOIFI SS No. 12 `[scholar-verify]`.* Proven: 8 zk unit tests (honest verifies,
disproportionate/tampered rejected, amounts hidden); circuit + gate emitted.

---

### Verification summary

| | artifact | tests |
|---|---|---|
| V2 | `composite.rs`, `bundle` grammar | `tests/composite.rs` 5/5 |
| V5 | `zakat.rs`, `zakat{}`, generated routing | `tests/zakat.rs` 5/5 + Hardhat 6/6 |
| V4 | `faraid.rs`, `contingency{}`, off-ramps | `tests/contingency.rs` 4/4 + faraid 9/9 + Hardhat 7/7 |
| V3 | `ahliyyah.js`, `/authorize`, DID registry | smoke 12/12 + live 5/5 |
| V1 | `zk.rs`, `--target zk` | `tests/zk.rs` 3/3 + zk unit 8/8 |

Full `cargo test`: 21 lib unit tests + all integration binaries green; fuzz clean; the four
base generators unchanged (codegen-safety holds). The new constructs are **opt-in** — they do
not destabilize the existing instruments.
