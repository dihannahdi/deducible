<div align="center">

# ∴ Deducible

**Compliance-by-construction for Islamic-finance contracts.**

A compiler in which a forbidden contract — a loan wearing the costume of a partnership —
simply refuses to be built. Riba does not get flagged after the fact; it *fails to compile.*

[deducible.tech](https://deducible.tech) · [Docs](https://docs.deducible.tech) · [Playground](https://docs.deducible.tech/playground/)

</div>

---

You write the terms of an instrument in a small language called `.fiqh`; the compiler reads them the
way a careful jurist would, and either lowers them to a smart contract — or refuses, naming the
principle the terms offend. The point is in that refusal: a contract that disguises a loan as a
partnership **will not compile**, and no contract is emitted.

## Quick start

```bash
git clone https://github.com/dihannahdi/deducible
cd deducible/fiqh-compiler

cargo build --release
BIN=./target/release/deducible

# a compliant diminishing partnership → consistent
$BIN check specs/musharakah_mutanaqisah.fiqh

# a loan disguised as a partnership → refused, with cited diagnostics
$BIN check specs/riba_disguised.fiqh

# check, then emit Solidity + Hardhat test + invariant manifest
$BIN build specs/musharakah_mutanaqisah.fiqh --target all
```

`deducible` verbs: `parse`, `check`, `build` (`--target solidity|manifest|zk|all`), `nl`, `lsp`,
`fuzz`. Prefer not to install? The [Playground](https://docs.deducible.tech/playground/) runs the
same engine (as WebAssembly) in your browser.

## Repository layout

| Path | What it is |
| --- | --- |
| `fiqh-compiler/` | the Rust workspace — the `deducible` compiler (`crates/fiqhc`, `crates/fiqhc-ffi`) |
| `fiqh-compiler/specs/` | example `.fiqh` specifications (positive + negative controls) |
| `fiqh-compiler/rules/` | pluggable rule modules (`aaoifi`, `dsn-mui`) — the jurisprudence as data |
| `fiqh-compiler/SPEC.md` | the normative `.fiqh` language specification |
| `contracts/` | Solidity (incl. the consensus valuation oracle) + generated output |
| `services/` | the invariant gateway + `ahliyyah` (capacity) middleware |
| `paper/` | the three accompanying papers |

## What it proves

- **Riba is unbuildable** — the headline negative control refuses to compile, citing the rule it offends.
- **One compiler, many instruments** — `musharakah_mutanaqisah`, `mudarabah`, `ijarah_imbt`, and a
  regime-neutral `commercial_escrow` under common law.
- **Proven in the open** — generated contracts have run on the Hedera testnet; a partnership that
  cannot exit whole, and zakat paid to the cent. See the [docs](https://docs.deducible.tech/proof/live-on-hedera/).

## The epistemic boundary

A machine does not issue a fatwa. Deducible proves that a contract is **consistent** with rules a
qualified human authored and cited — every time, identically, for anyone to verify. It does not
decide what is halal. Citations in the rule-base are pointers to the tradition for scholarly
verification, not pronouncements.

## License

[MIT](./LICENSE). *Allāhu aʿlam.*
