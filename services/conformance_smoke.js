// Deterministic smoke test for the continuous-conformance module (services/conformance.js).
// No server, no deps — `node services/conformance_smoke.js`. Exit 0 = all pass.
const fs = require("fs");
const os = require("os");
const path = require("path");
const conformance = require("./conformance");

const DIR = fs.mkdtempSync(path.join(os.tmpdir(), "conformance-smoke-"));
let pass = 0, fail = 0;
function check(name, cond) {
  if (cond) { pass++; console.log("  ok   " + name); }
  else { fail++; console.log("  FAIL " + name); }
}

const TARGET = "TestInstrumentGen";
const CONSTRAINED = ["risk.capital_guarantee", "risk.loss"];

console.log("conformance module — baseline + drift:");

const e0 = conformance.attest(DIR, TARGET, { "risk.capital_guarantee": "none", "risk.loss": "proportional_to_ownership", "admin.fee_bps": 25 }, { allowed: true, violations: [] }, CONSTRAINED);
check("first attestation is seq 0", e0.seq === 0);
check("first attestation has no drift (nothing to drift from)", e0.drift === null);

const e1 = conformance.attest(DIR, TARGET, { "risk.capital_guarantee": "none", "risk.loss": "proportional_to_ownership", "admin.fee_bps": 40 }, { allowed: true, violations: [] }, CONSTRAINED);
check("second attestation is seq 1", e1.seq === 1);
check("unconstrained field change is surfaced as drift", e1.drift.unconstrained_drift.includes("admin.fee_bps"));
check("unconstrained drift does not touch rule-covered fields", !e1.drift.unconstrained_drift.includes("risk.capital_guarantee"));

const e2 = conformance.attest(DIR, TARGET, { "risk.capital_guarantee": "bank", "risk.loss": "proportional_to_ownership", "admin.fee_bps": 40 }, { allowed: false, violations: [{ code: "RIBA-1" }] }, CONSTRAINED);
check("a rule-covered drift is recorded as changed", e2.drift.changed.some((c) => c.field === "risk.capital_guarantee"));
check("a rule-covered drift is NOT counted as unconstrained_drift (enforce() already governs it)", !e2.drift.unconstrained_drift.includes("risk.capital_guarantee"));
check("the enforcement verdict itself is carried through unchanged", e2.allowed === false && e2.violations[0].code === "RIBA-1");

console.log("conformance module — hash chain integrity:");
const entries = conformance.readLog(DIR, TARGET);
check("three entries were logged", entries.length === 3);
const okChain = conformance.verifyChain(entries);
check("an untouched chain verifies intact", okChain.ok === true);

// Tamper with the FIRST entry directly on disk, as an external editor of the log file would.
const logFile = path.join(DIR, TARGET + ".jsonl");
const lines = fs.readFileSync(logFile, "utf8").trim().split("\n");
const tampered = JSON.parse(lines[0]);
tampered.terms["risk.capital_guarantee"] = "TAMPERED";
lines[0] = JSON.stringify(tampered);
fs.writeFileSync(logFile, lines.join("\n") + "\n");

const tamperedEntries = conformance.readLog(DIR, TARGET);
const brokenChain = conformance.verifyChain(tamperedEntries);
check("editing a past entry is detected as a broken chain", brokenChain.ok === false && brokenChain.brokenAtSeq === 0);

fs.rmSync(DIR, { recursive: true, force: true });

console.log(`\nconformance smoke: ${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
