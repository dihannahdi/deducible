# fiqhc — in-browser validation (WebAssembly)

A zero-backend demo: the fiqhc engine compiled to Wasm validates `.fiqh` specifications
entirely client-side.

## Build & run
```
cd fiqh-compiler
rustup target add wasm32-unknown-unknown
cargo build -p fiqhc-ffi --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/fiqhc_ffi.wasm editors/web/
cd editors/web && python3 -m http.server 8080   # then open http://localhost:8080
```
Edit the spec in the page; violations appear with their fiqh code and daleel as you click Check.
The same `.wasm` + the same marshaling is exercised headlessly by
`crates/fiqhc-ffi/wasm_smoke.js`.
