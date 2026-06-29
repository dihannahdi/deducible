# deducible — a compliance-by-construction compiler for the fiqh of muʿāmalāt

**deducible** lifts *compliance by construction* from a property of one hand-written contract to a
property of a **language**. From a high-level `.fiqh` specification it:

1. **refuses to compile** any specification whose declared economics contradict its declared fiqh
   rule-base — riba, gharar, the loss of risk-sharing, self-dealing — with fiqh-cited, line-precise
   diagnostics, and emits no contract; and
2. **emits** a verified, deployable Solidity contract (plus a Hardhat property test, a deploy
   descriptor, and a portable JSON invariant manifest) from one that is consistent.

It is written in Rust (lexer → parser → AST → the fiqh invariant engine → codegen). It orchestrates
the proven toolchain — `solc` via Hardhat, and the Hedera SDK (Node) — for Solidity compilation and
on-chain deployment; reimplementing those is out of scope.

> **deducible issues no fatwa.** It proves a specification is *consistent with a declared,
> human-authored, citation-bearing rule-base*. The fiqh validity of that rule-base — and of
> instruments that carry *khilāf* — is a qualified scholar's domain. Every citation is flagged
> `[scholar-verify]`. Where a contract is form-valid but its *intent* (maqṣad) is the question,
> deducible raises a **warning, never a ruling**. *Allāhu aʿlam.*

## What it covers — the whole economy, not one instrument

**~27 contract classes** across every family of the fiqh of transactions, each refusing its own riba/
gharar form by construction:

| Family | Classes |
|---|---|
| Sale (buyūʿ) | murābaḥah, salam, istiṣnāʿ, ṣarf, tawarruq |
| Partnership | mushārakah, mushārakah mutanāqiṣa, muḍāraba, muḍāraba pool |
| Lease & service | ijārah, ijārah muntahiya bi-l-tamlīk, juʿāla, ʿāriyya, wakāla |
| Security & credit | qarḍ ḥasan, rahn, kafāla, ḥawāla, wadīʿa |
| Capital markets | sukūk, takāful |
| Social | waqf, hibah, waṣiyya |
| Charge | zakat (all genera + the eight aṣnāf of al-Tawba 9:60) |
| Common-law (universality) | commercial escrow + a regime-neutral judiciary engine |

Beyond single instruments it also carries:

- **A multilateral `pool` primitive** — moving past the bilateral ʿaqd to many participants
  (sukūk holders, takāful participants, a muḍāraba's rabb al-māl pool), shares summing to the whole,
  distributed pro-rata.
- **Composite (al-ʿuqūd al-murakkabah) cycle detection** — riba treated as *topological*: bayʿ
  al-ʿīnah and organised tawarruq are forbidden by the *shape* of the asset-flow ring, not the markup.
- **Pluggable, madhhab-level rule modules** — AAOIFI, DSN-MUI, and the four schools (ḥanafī, mālikī,
  shāfiʿī, ḥanbalī). The *same* spec can be consistent under one school and refused under another
  (a live khilāf, e.g. profit-must-track-capital), each module carrying its own daleel.
- **A maqāṣid / ḥiyal-risk layer** — the deliberate ceiling: it *warns* (never errors) where a
  form-compliant contract's purpose (e.g. *taḥāyul al-murābaḥa ʿalā al-ribā*) a scholar should judge.

## Commands

```
deduce parse <file.fiqh>                  # dump the AST
deduce check <file.fiqh> [--rules <auth>] # run the fiqh invariant engine (no codegen)
deduce build <file.fiqh> --root R [--rules <auth>]
                                             # check, then emit .sol + test + descriptor + manifest under R
deduce fuzz  [N]                          # property-fuzz the engine (default 100k; never panics)
deduce lsp                                # stdio Language Server (diagnostics in your editor)
deduce nl   <file.txt>                    # draft a .fiqh from natural language (experimental, LLM)
```

`--rules <authority>` (e.g. `aaoifi`, `dsn-mui`, `hanafi`, `maliki`, `shafii`, `hanbali`, or any
`rules/<name>.rules.json`) governs **both** `check` and `build`: a spec the chosen authority refuses
will not generate a contract. Without it, the builtin engine (the universal core) applies.

## Build & test

```
cargo test                                            # the full engine + per-instrument suites
deduce check specs/riba_disguised.fiqh             # MUST be refused (exit 1, no .sol)
deduce build specs/murabahah.fiqh --root <root>    # emit a murabaha contract + test + manifest
deduce check specs/musharakah_mutanaqisah.fiqh --rules maliki   # refused (profit_tracks_capital)
deduce check specs/musharakah_mutanaqisah.fiqh --rules hanafi   # consistent (khilaf)
npx hardhat test test/generated/*.test.js             # the generated property tests
```

The flagship **murābaḥah** has been proven **live on Hedera testnet** running its full cost-plus
lifecycle (qabḍ → cost disclosure → sale → the buyer paying the fixed deferred total).

## Specification & papers

- **[`SPEC.md`](SPEC.md)** — the `.fiqh` language spec: EBNF grammar, AST, the invariant engine +
  diagnostic-code reference, the pluggable rule-base format, the codegen targets.
- **[`QAWAID.md`](QAWAID.md)** — how the compiled invariants descend from the five great qawāʿid.
- `paper/` — four design-science papers, including **`the_whole_economy.md`** (the extension from
  four instruments to the breadth of muʿāmalāt).

## Layout

```
crates/fiqhc/src/   lexer · parser · ast · sema (the invariant engine) · codegen · maqasid · zakat · faraid · composite · zk · lsp · nl
specs/              one positive + one negative-control .fiqh per instrument
rules/              aaoifi · dsn-mui · hanafi · maliki · shafii · hanbali  (pluggable rule modules)
paper/              the four papers
```

*(Naming: the project and brand are **deducible**; the CLI command/binary is **`deduce`** (you
*deduce* a contract's compliance); the Rust library crate is internally named `fiqhc` for historical
reasons — a code-level detail.)*

## License

MIT. The rule modules and citations require ratification by a qualified scholar before any reliance.
