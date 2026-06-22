# fiqhc — a compliance-by-construction compiler for Islamic-finance contracts

`fiqhc` lifts *compliance by construction* from a property of one hand-written contract to a
property of a **language**. From a high-level `.fiqh` specification it:

1. **refuses to compile** any specification whose declared economics contradict its declared
   fiqh rule-base (riba / gharar / loss of risk-sharing / self-dealing), with fiqh-cited
   diagnostics — and emits no contract; and
2. **emits** a verified, deployable Solidity contract (plus a Hardhat property test and a deploy
   descriptor) from one that is consistent.

It is written in Rust (lexer → parser → AST → the fiqh invariant engine → codegen). It shells
out to the proven toolchain — `solc` via Hardhat, and the Hedera SDK (Node) — for Solidity
compilation and on-chain deployment; reimplementing those is out of scope.

> **The compiler issues no fatwa.** It proves a specification is *consistent with a declared,
> human-authored, citation-bearing rule-base*. The fiqh validity of that rule-base — and of
> instruments that carry *khilaf* — is a qualified scholar's domain. Every citation is flagged
> `[scholar-verify]`. *Allahu a'lam.*

## Commands

```
fiqhc parse  <file.fiqh>            # dump the AST
fiqhc check  <file.fiqh>            # run the fiqh invariant engine (no codegen)
fiqhc build  <file.fiqh> --root R   # check, then emit .sol + test + deploy descriptor under R
fiqhc nl     <file.txt>             # draft a .fiqh from natural language (experimental, LLM)
```

## Layout

```
crates/fiqhc/src/   lexer.rs parser.rs ast.rs sema.rs codegen.rs nl.rs main.rs
specs/              musharakah_mutanaqisah.fiqh  mudarabah.fiqh  ijarah_imbt.fiqh
                    riba_disguised.fiqh  mudarabah_riba.fiqh  ijarah_riba.fiqh   (negative controls)
examples/nl/        mudarabah.txt
```

## Build & test (inside the container)

```
cargo test                                  # parser + fiqh engine + generality
fiqhc check specs/riba_disguised.fiqh       # MUST be refused (exit 1, no .sol)
fiqhc build specs/musharakah_mutanaqisah.fiqh --root /workspace
fiqhc build specs/mudarabah.fiqh            --root /workspace
fiqhc build specs/ijarah_imbt.fiqh          --root /workspace
npx hardhat test test/generated/*.test.js   # 14 generated tests
node scripts/deploy_generated.js fiqh-compiler/out/MusharakahMutanaqisahGen.deploy.json
```

## Specification
See **[`SPEC.md`](SPEC.md)** for the full language specification — EBNF grammar, AST, parser
mechanics, the legal invariant engine + diagnostic-code reference, the pluggable rule-base format,
and the codegen targets — documented to mainstream-language depth for security peer review.

See `paper/algorithmic_jurisprudence.md` for the write-up and on-chain evidence.
