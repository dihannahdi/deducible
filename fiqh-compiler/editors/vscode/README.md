# fiqh — VS Code extension

Live, fiqh-cited diagnostics for `.fiqh` specifications, powered by the `fiqhc` Language Server.
As you type a contract spec, a guaranteed-capital clause or a self-named price is underlined with
a red squiggle — e.g. `[RIBA-1] capital is guaranteed to 'bank' …  daleel: al-Baqarah 2:275; AAOIFI SS No. 12` —
before you ever touch a terminal.

## Prerequisites
- Build the compiler: `cargo build` in `fiqh-compiler/`, then put `fiqhc` on your `PATH`
  (or set `fiqh.serverPath` to its absolute path in VS Code settings).

## Run (development)
```
cd editors/vscode
npm install
code .          # then press F5 to launch an Extension Development Host
```
Open any `.fiqh` file; diagnostics appear from `fiqhc lsp` over stdio.

## How it works
The extension starts `fiqhc lsp` (a stdio JSON-RPC Language Server) and forwards
`textDocument/didOpen|didChange|didSave`. The server lexes, parses, and runs the fiqh invariant
engine, publishing `textDocument/publishDiagnostics` with precise ranges, the error code, and the
cited daleel. The engine issues no fatwa — it proves consistency with a declared rule-base.
