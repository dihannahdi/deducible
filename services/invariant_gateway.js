// Vision #4 — the invariant gateway. A ledger-agnostic microservice that injects the compiled
// legal invariants into ANY backend in real time: before a contract's terms are committed (to a
// blockchain OR a traditional database), POST them to /enforce and a non-compliant transition is
// refused — exactly as the compiler would refuse it — with the cited rule.
//
//   node services/invariant_gateway.js     (binds 127.0.0.1:8799 inside the container)
//
// Endpoints:
//   GET  /              dashboard
//   GET  /manifests     list loaded invariant manifests
//   POST /enforce       { target, terms } -> { allowed, violations:[{code,field,expected,got,citation}] }
//   POST /compile       { spec }          -> runs `fiqhc check` and returns the verdict
const http = require("http");
const fs = require("fs");
const path = require("path");
const { execFile } = require("child_process");

const OUT = path.join(__dirname, "..", "fiqh-compiler", "out");
const FIQHC = path.join(__dirname, "..", "fiqh-compiler", "target", "debug", "fiqhc");
const PORT = process.env.GATEWAY_PORT || 8799;

function loadManifests() {
  const map = {};
  if (!fs.existsSync(OUT)) return map;
  for (const f of fs.readdirSync(OUT)) {
    if (!f.endsWith(".manifest.json")) continue;
    try {
      const m = JSON.parse(fs.readFileSync(path.join(OUT, f), "utf8"));
      const base = f.replace(/\.manifest\.json$/, "");
      map[base] = m;
      if (m.instrument && !map[m.instrument]) map[m.instrument] = m;
    } catch (e) {}
  }
  return map;
}

function get(terms, field) {
  if (Object.prototype.hasOwnProperty.call(terms, field)) return terms[field];
  return field.split(".").reduce((o, k) => (o == null ? undefined : o[k]), terms);
}

function enforce(manifest, terms) {
  const violations = [];
  for (const c of manifest.constraints || []) {
    const got = get(terms, c.field);
    let ok;
    if (c.op === "eq") ok = String(got) === String(c.value);
    else if (c.op === "ne") ok = got === undefined ? true : String(got) !== String(c.value);
    else if (c.op === "gt") ok = got !== undefined && Number(got) > Number(c.value);
    else ok = false;
    if (!ok) {
      violations.push({ code: c.code, field: c.field, op: c.op, expected: c.value, got: got === undefined ? null : got, citation: c.citation });
    }
  }
  return { allowed: violations.length === 0, instrument: manifest.instrument, regime: manifest.regime, violations };
}

function readBody(req) {
  return new Promise((resolve) => {
    let d = "";
    req.on("data", (x) => (d += x));
    req.on("end", () => resolve(d));
  });
}

const DASH = `<!doctype html><meta charset=utf-8><title>Invariant Gateway</title>
<style>body{font:15px/1.5 system-ui,sans-serif;max-width:760px;margin:2rem auto;padding:0 1rem;color:#1a1a1a}
h1{font-size:1.3rem}textarea,select{width:100%;font-family:ui-monospace,monospace;font-size:13px}
textarea{height:140px}button{padding:.5rem 1rem;font-size:14px;cursor:pointer;margin-top:.5rem}
pre{background:#f5f5f4;padding:1rem;border-radius:6px;white-space:pre-wrap}.ok{color:#15803d}.no{color:#b91c1c}</style>
<h1>Invariant Gateway <span style=font-weight:400;color:#666>— legal invariants, injected real-time</span></h1>
<p>Pick a compiled rule-base, propose a contract's terms, and the gateway allows or refuses the
transition with the cited rule — the same compliance-by-construction, at any backend.</p>
<label>Rule-base</label><select id=t></select>
<label>Proposed terms (JSON, dotted keys)</label>
<textarea id=terms>{
  "risk.capital_guarantee": "bank",
  "risk.loss": "none",
  "returns.rent.basis": "principal",
  "returns.buyout.priceSource": "self"
}</textarea>
<button onclick=run()>Enforce</button>
<pre id=out>—</pre>
<script>
fetch("/manifests").then(r=>r.json()).then(ks=>{const s=document.getElementById("t");
 ks.filter(k=>/Gen$/.test(k)).forEach(k=>{const o=document.createElement("option");o.textContent=k;s.appendChild(o)})});
async function run(){const target=document.getElementById("t").value;let terms;
 try{terms=JSON.parse(document.getElementById("terms").value)}catch(e){return out.textContent="bad JSON: "+e}
 const r=await fetch("/enforce",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({target,terms})});
 const j=await r.json();const el=document.getElementById("out");
 el.innerHTML=(j.allowed?'<b class=ok>ALLOWED</b>':'<b class=no>REFUSED</b>')+"\\n"+JSON.stringify(j,null,2)}
</script>`;

const server = http.createServer(async (req, res) => {
  const send = (code, obj) => {
    res.writeHead(code, { "Content-Type": "application/json" });
    res.end(JSON.stringify(obj, null, 2));
  };
  if (req.method === "GET" && req.url === "/") {
    res.writeHead(200, { "Content-Type": "text/html" });
    return res.end(DASH);
  }
  if (req.method === "GET" && req.url === "/manifests") return send(200, Object.keys(loadManifests()));
  if (req.method === "POST" && req.url === "/enforce") {
    try {
      const { target, terms } = JSON.parse(await readBody(req));
      const m = loadManifests()[target];
      if (!m) return send(404, { error: "no manifest for '" + target + "'", available: Object.keys(loadManifests()) });
      return send(200, enforce(m, terms || {}));
    } catch (e) {
      return send(400, { error: String(e) });
    }
  }
  if (req.method === "POST" && req.url === "/compile") {
    try {
      const { spec } = JSON.parse(await readBody(req));
      const tmp = path.join("/tmp", "gw_" + Date.now() + ".fiqh");
      fs.writeFileSync(tmp, spec);
      execFile(FIQHC, ["check", tmp], (err, stdout, stderr) => {
        try { fs.unlinkSync(tmp); } catch (e) {}
        send(200, { consistent: !err, stdout, stderr });
      });
      return;
    } catch (e) {
      return send(400, { error: String(e) });
    }
  }
  send(404, { error: "not found" });
});

server.listen(PORT, "127.0.0.1", () => console.log("invariant-gateway on http://127.0.0.1:" + PORT));
