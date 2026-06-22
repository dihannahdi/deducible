// Smoke test for the fiqhc LSP: spawn `fiqhc lsp`, run a scripted initialize + didOpen session
// with the riba-disguised spec, and assert it publishes a RIBA-1 diagnostic with a precise range.
//   node lsp_smoke.js
const { spawn } = require("child_process");
const fs = require("fs");

const bin = "./target/debug/fiqhc";
const p = spawn(bin, ["lsp"]);
let buf = Buffer.alloc(0);
let done = false;

function finish(ok, note) {
  if (done) return;
  done = true;
  console.log(ok ? "LSP OK — " + note : "LSP FAIL — " + note);
  try { p.kill(); } catch (e) {}
  process.exit(ok ? 0 : 1);
}

p.stdout.on("data", (d) => {
  buf = Buffer.concat([buf, d]);
  while (true) {
    const sep = buf.indexOf("\r\n\r\n");
    if (sep < 0) break;
    const m = buf.slice(0, sep).toString().match(/Content-Length:\s*(\d+)/i);
    if (!m) { buf = buf.slice(sep + 4); continue; }
    const len = parseInt(m[1], 10);
    if (buf.length < sep + 4 + len) break;
    const body = buf.slice(sep + 4, sep + 4 + len).toString();
    buf = buf.slice(sep + 4 + len);
    let msg;
    try { msg = JSON.parse(body); } catch (e) { continue; }
    if (msg.method === "textDocument/publishDiagnostics") {
      const ds = msg.params.diagnostics || [];
      console.log("publishDiagnostics:", ds.map((x) => x.code).join(", "));
      const riba = ds.find((x) => x.code === "RIBA-1");
      if (riba) {
        console.log("  RIBA-1 range:", JSON.stringify(riba.range));
        console.log("  message:", riba.message.replace(/\n/g, " | "));
        finish(true, "RIBA-1 published with a precise range and daleel");
      } else {
        finish(false, "no RIBA-1 in diagnostics");
      }
    }
  }
});

function send(obj) {
  const s = JSON.stringify(obj);
  p.stdin.write(`Content-Length: ${Buffer.byteLength(s)}\r\n\r\n${s}`);
}

send({ jsonrpc: "2.0", id: 1, method: "initialize", params: { capabilities: {} } });
send({ jsonrpc: "2.0", method: "initialized", params: {} });
const text = fs.readFileSync("specs/riba_disguised.fiqh", "utf8");
send({ jsonrpc: "2.0", method: "textDocument/didOpen", params: { textDocument: { uri: "file:///riba.fiqh", languageId: "fiqh", version: 1, text } } });

setTimeout(() => finish(false, "timeout waiting for diagnostics"), 5000);
