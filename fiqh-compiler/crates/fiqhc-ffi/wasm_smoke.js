// Node harness for the wasm build — the SAME marshaling a browser uses. Loads fiqhc_ffi.wasm,
// feeds the riba-disguised spec through fiqh_check_json, and asserts a RIBA-1 diagnostic.
//   (run from fiqh-compiler/) node crates/fiqhc-ffi/wasm_smoke.js
const fs = require("fs");

(async () => {
  const bytes = fs.readFileSync("target/wasm32-unknown-unknown/release/fiqhc_ffi.wasm");
  const module = await WebAssembly.compile(bytes);
  // Defensive: stub any imports (a pure-compute module usually has none).
  const importObject = {};
  for (const imp of WebAssembly.Module.imports(module)) {
    importObject[imp.module] = importObject[imp.module] || {};
    if (imp.kind === "function") importObject[imp.module][imp.name] = () => 0;
    else if (imp.kind === "memory") importObject[imp.module][imp.name] = new WebAssembly.Memory({ initial: 256 });
    else if (imp.kind === "table") importObject[imp.module][imp.name] = new WebAssembly.Table({ initial: 0, element: "anyfunc" });
    else importObject[imp.module][imp.name] = 0;
  }
  const { exports: ex } = await WebAssembly.instantiate(module, importObject);

  const input = new Uint8Array(fs.readFileSync("specs/riba_disguised.fiqh"));
  const ptr = ex.fiqh_alloc(input.length);
  new Uint8Array(ex.memory.buffer).set(input, ptr);
  const res = ex.fiqh_check_json(ptr, input.length);
  const m = new Uint8Array(ex.memory.buffer);
  let end = res;
  while (m[end] !== 0) end++;
  const json = Buffer.from(m.slice(res, end)).toString("utf8");
  ex.fiqh_free_cstr(res);
  ex.fiqh_free(ptr, input.length);

  const obj = JSON.parse(json);
  const codes = obj.diagnostics.map((d) => d.code);
  console.log("wasm size:", bytes.length, "bytes");
  console.log("wasm ok:", obj.ok, "| diagnostics:", codes.join(", "));
  const riba = obj.diagnostics.find((d) => d.code === "RIBA-1");
  if (riba) console.log("  RIBA-1 @ " + riba.line + ":" + riba.col + " — " + riba.citation);
  process.exit(codes.includes("RIBA-1") ? 0 : 1);
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
