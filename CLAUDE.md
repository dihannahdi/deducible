# CLAUDE.md — Deducible (compiler)

Guidance for Claude Code (and humans) working in this repo.

## What this is

**Deducible** — a compliance-by-construction compiler for Islamic-finance contracts. You write an
instrument in `.fiqh`; the engine proves it *consistent* with a cited rule-base and lowers it to
Solidity + a portable invariant manifest, or **refuses** (a disguised loan won't compile). The CLI
binary is `deducible`. The web + docs live at **deducible.tech** / **docs.deducible.tech** (repo
`dihannahdi/deducibleweb`); the in-browser playground runs *this* engine compiled to WebAssembly.

> Internal note: the Rust crate is named `fiqhc` (and `fiqhc-ffi`) and the language file extension is
> `.fiqh` — those name the *discipline*. The user-facing **tool/CLI is `deducible`**. Don't "fix" the
> internal crate names to deducible without an intentional, verified refactor.

## Build & run

The compiler is a Rust workspace under `fiqh-compiler/` (edition 2021).

```bash
cd fiqh-compiler
cargo test                                  # all green
cargo build --release                       # produces ./target/release/deducible

./target/release/deducible check specs/musharakah_mutanaqisah.fiqh   # ✓ consistent
./target/release/deducible check specs/riba_disguised.fiqh           # ✗ refused, cited
./target/release/deducible build specs/musharakah_mutanaqisah.fiqh --target all
```

CLI verbs: `parse`, `check`, `build` (`--target solidity|manifest|zk|all`), `nl`, `lsp`, `fuzz`.
Select a jurisdiction with `--rules aaoifi|dsn-mui`.

## Layout

| Path | What |
| --- | --- |
| `fiqh-compiler/crates/fiqhc` | the engine: `lexer · parser · ast · sema · codegen` + `composite · zakat · faraid · zk · nl · lsp · fuzz` |
| `fiqh-compiler/crates/fiqhc-ffi` | C-ABI + WebAssembly surface (`fiqh_check_json` …); builds `.so` and the playground `.wasm` |
| `fiqh-compiler/specs/` | example `.fiqh` (positive + negative controls) |
| `fiqh-compiler/rules/` | pluggable rule modules as JSON (`aaoifi`, `dsn-mui`) — jurisprudence is data, not code |
| `fiqh-compiler/SPEC.md` | **authoritative** `.fiqh` language spec (grammar/AST/diagnostics) |
| `contracts/`, `services/`, `agents/`, `scripts/` | Solidity (incl. consensus oracle), invariant gateway + `ahliyyah` middleware, valuer agents, Hedera deploy |
| `paper/` | the three accompanying papers |

## Working rules

- **`SPEC.md` is the source of truth** for `.fiqh` syntax — read it before changing the parser or docs.
- **Codegen is deterministic and the 4 base generators are byte-stable** — `tests/codegen_safety.rs`
  enforces a uniform safety property (every value-moving external fn carries `nonReentrant`, role
  modifiers + pinned pragma present). New language constructs are **opt-in** so existing output doesn't
  shift; run `cargo test` (incl. the generated Hardhat tests) after any codegen change.
- **The engine is a pure function** of `(source, regime, rule-module)` — no clock/RNG/IO in the check
  path. Keep it that way (it's what makes "identical for anyone to verify" literal). `deducible nl`
  is the only LLM-touching verb, and its output is re-checked by the same deterministic gate.
- **On-chain deploys are TESTNET ONLY.** Never paste keys; `.env` is gitignored (`.env.example` shows
  the shape). See `scripts/` and the docs' "Live on Hedera" page for costs/gotchas.

## The epistemic boundary (non-negotiable)

A machine does not issue a fatwa. Deducible proves a contract is **consistent** with rules a qualified
human authored and cited — it does not decide what is halal. Citations are flagged `[scholar-verify]`
(scriptural/fiqh) or `[verify]` (common-law); they are pointers to the tradition, never pronouncements.
Preserve this framing in code comments, diagnostics, and docs. *Allāhu aʿlam.*
