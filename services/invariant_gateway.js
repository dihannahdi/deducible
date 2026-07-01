// Vision #4 — the invariant gateway. A ledger-agnostic microservice that injects the compiled
// legal invariants into ANY backend in real time: before a contract's terms are committed (to a
// blockchain OR a traditional database), POST them to /enforce and a non-compliant transition is
// refused — exactly as the compiler would refuse it — with the cited rule.
//
//   node services/invariant_gateway.js     (binds 127.0.0.1:8799 inside the container)
//
// Endpoints:
//   GET  /                    dashboard
//   GET  /manifests           list loaded invariant manifests
//   POST /enforce             { target, terms } -> { allowed, violations:[{code,field,expected,got,citation}] }
//   POST /compile             { spec }          -> runs `fiqhc check` and returns the verdict
//   POST /attest              { target, terms } -> re-checks + appends a tamper-evident conformance
//                              record (see conformance.js); closes the formation-vs-execution gap
//                              for the "gateway in front of an existing core banking system" mode
//   GET  /conformance/:target -> the attestation history for one target, with drift surfaced and
//                              the hash chain's integrity verified
const http = require("http");
const fs = require("fs");
const path = require("path");
const { execFile } = require("child_process");
const ahliyyah = require("./ahliyyah");
const conformance = require("./conformance");

const OUT = path.join(__dirname, "..", "fiqh-compiler", "out");
// The compiled binary is `deduce` (brand: Deducible; internal crate/bin history: fiqhc) —
// `.exe` on Windows, no extension elsewhere.
const FIQHC = path.join(__dirname, "..", "fiqh-compiler", "target", "debug", process.platform === "win32" ? "deduce.exe" : "deduce");
const DID_REGISTRY = path.join(__dirname, "did_registry.json");
const CONFORMANCE_DIR = path.join(__dirname, "conformance-log");
const PORT = process.env.GATEWAY_PORT || 8799;

function loadDids() {
  const reg = ahliyyah.loadRegistry(DID_REGISTRY);
  // strip the documentation key
  const out = {};
  for (const [k, v] of Object.entries(reg)) if (!k.startsWith("_")) out[k] = v;
  return out;
}

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
  if (req.method === "GET" && req.url === "/dids") return send(200, Object.keys(loadDids()));
  // Ahliyyah + DID middleware (vector #3): verify the legal capacity of every contracting party
  // (and, optionally, the invariant terms) BEFORE a compiled contract may execute.
  if (req.method === "POST" && req.url === "/authorize") {
    try {
      const { target, terms, parties } = JSON.parse(await readBody(req));
      const manifests = loadManifests();
      const m = manifests[target];
      if (!m) return send(404, { error: "no manifest for '" + target + "'", available: Object.keys(manifests) });
      const required = (m.ahliyyah && m.ahliyyah.principals) || [];
      const cap = ahliyyah.authorize(loadDids(), parties || {}, required);
      const inv = terms ? enforce(m, terms) : { allowed: true, violations: [] };
      const allowed = cap.allCapable && inv.allowed;
      return send(200, {
        allowed,
        instrument: m.instrument,
        regime: m.regime,
        ahliyyah: cap,
        invariants: inv,
        note: "an 'aqd is valid only if both the terms AND the parties pass; capacity (ahliyyat al-ada') is verified per party. No fatwa is issued. [scholar-verify]",
      });
    } catch (e) {
      return send(400, { error: String(e) });
    }
  }
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
  if (req.method === "POST" && req.url === "/attest") {
    try {
      const { target, terms } = JSON.parse(await readBody(req));
      const m = loadManifests()[target];
      if (!m) return send(404, { error: "no manifest for '" + target + "'", available: Object.keys(loadManifests()) });
      const enforcement = enforce(m, terms || {});
      const constrainedFields = (m.constraints || []).map((c) => c.field);
      const entry = conformance.attest(CONFORMANCE_DIR, target, terms || {}, enforcement, constrainedFields);
      return send(200, {
        allowed: entry.allowed,
        violations: entry.violations,
        seq: entry.seq,
        is_baseline: entry.seq === 0,
        drift: entry.drift,
        note:
          "this attestation is now part of the tamper-evident conformance log for '" +
          target +
          "'; GET /conformance/" +
          target +
          " for the full history. Re-attestation is only as good as how often the live system is made to submit one — that cadence is an operational/audit-policy requirement, not something this endpoint can enforce.",
      });
    } catch (e) {
      return send(400, { error: String(e) });
    }
  }
  if (req.method === "GET" && req.url.startsWith("/conformance/")) {
    const target = decodeURIComponent(req.url.slice("/conformance/".length));
    const entries = conformance.readLog(CONFORMANCE_DIR, target);
    if (!entries.length) return send(404, { error: "no conformance history for '" + target + "' — POST /attest at least once first" });
    const integrity = conformance.verifyChain(entries);
    const everUnconstrainedDrift = Array.from(
      new Set(entries.flatMap((e) => (e.drift && e.drift.unconstrained_drift) || []))
    );
    return send(200, {
      target,
      entries,
      chain_intact: integrity.ok,
      broken_at_seq: integrity.brokenAtSeq,
      ever_had_unconstrained_drift: everUnconstrainedDrift,
      note:
        "chain_intact=false means a past entry was edited, reordered, or removed outside this service — treat the whole log as suspect from broken_at_seq onward. ever_had_unconstrained_drift lists fields that changed since the baseline attestation but that the rule module does not constrain, so a violation there was never possible; it is visibility for an auditor, not a refusal.",
    });
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
