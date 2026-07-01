// Continuous conformance for the invariant gateway — closing the formation-vs-execution gap
// in the "gateway in front of an existing core banking system" deployment mode.
//
// The gateway's /enforce endpoint (invariant_gateway.js) answers one question: are these
// PROPOSED terms consistent with the rule-base, right now, at the moment they are checked?
// That closes the gap between what a board reads and what a contract is FORMED to be. It does
// not, by itself, close a second gap the paper names in its own opening paragraph: the contract
// a board reads, the contract a customer signs, and the logic a core banking system executes
// year after year can still drift apart, because nothing re-checks the booked product after
// formation. A parameter changed in a batch job, a side letter applied in production but never
// re-submitted for a check — none of that is visible to a one-time gate.
//
// This module makes re-submission itself auditable. It does not, and cannot, force a live
// system to re-submit its configuration on any particular schedule — that remains an
// operational/audit-policy requirement an institution or supervisor must impose, not a
// property code can guarantee. What it guarantees is: IF a live configuration is periodically
// re-attested, every attestation is checked against the SAME rule-base as at formation, drift
// outside the rule-base's own declared fields is surfaced (not silently absorbed), and the
// history is tamper-evident (a hash chain, so a deleted or edited past entry is detectable).
//
// Epistemics unchanged: this proves a submitted configuration is, or is not, still consistent
// with the rule-base, and whether it has changed since the first attestation. It does not
// verify that what was submitted is what is actually running in the core system — that
// integration (pulling the live configuration, rather than trusting what is POSTed) is the
// institution's to build and the supervisor's to require. [scholar-verify N/A — this layer is
// operational/audit tooling, not a fiqh ruling]

const fs = require("fs");
const path = require("path");
const crypto = require("crypto");

const GENESIS = "GENESIS";

function stableStringify(value) {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return "[" + value.map(stableStringify).join(",") + "]";
  const keys = Object.keys(value).sort();
  return "{" + keys.map((k) => JSON.stringify(k) + ":" + stableStringify(value[k])).join(",") + "}";
}

function sha256Hex(s) {
  return crypto.createHash("sha256").update(s, "utf8").digest("hex");
}

function logPath(dir, target) {
  const safe = String(target).replace(/[^a-zA-Z0-9_.-]/g, "_");
  return path.join(dir, safe + ".jsonl");
}

function readLog(dir, target) {
  const p = logPath(dir, target);
  if (!fs.existsSync(p)) return [];
  const lines = fs.readFileSync(p, "utf8").split("\n").filter((l) => l.trim().length > 0);
  const entries = [];
  for (const line of lines) {
    try {
      entries.push(JSON.parse(line));
    } catch (e) {
      // A corrupt line breaks the chain read, not the whole service; surfaced by verifyChain.
      entries.push({ corrupt: true, raw: line });
    }
  }
  return entries;
}

// Recompute the hash chain over `entries`; returns { ok, brokenAtSeq } — brokenAtSeq is the
// first entry whose stored hash does not match its recomputed content+prev_hash hash, which is
// exactly what a deleted, reordered, or edited past attestation would produce.
function verifyChain(entries) {
  let prev = GENESIS;
  for (const e of entries) {
    if (e.corrupt) return { ok: false, brokenAtSeq: e.seq ?? "?" };
    const { hash, ...rest } = e;
    const recomputed = sha256Hex(stableStringify(rest) + "|" + prev);
    if (recomputed !== hash || e.prev_hash !== prev) {
      return { ok: false, brokenAtSeq: e.seq };
    }
    prev = hash;
  }
  return { ok: true, brokenAtSeq: null };
}

// Diff `terms` against `baseline` at the FIELD level (dotted keys, matching the gateway's own
// `enforce()`). `constrainedFields` is the set of fields the rule module actually pins — a
// change confined to those fields is already caught by enforce()'s allowed/violations verdict;
// what this surfaces is drift OUTSIDE that set, which the rule module never required fixed and
// so cannot block, but which an auditor should still be able to see happened.
function diffTerms(baseline, terms, constrainedFields) {
  const keys = new Set([...Object.keys(baseline || {}), ...Object.keys(terms || {})]);
  const changed = [];
  const added = [];
  const removed = [];
  for (const k of keys) {
    const inBase = baseline && Object.prototype.hasOwnProperty.call(baseline, k);
    const inNew = terms && Object.prototype.hasOwnProperty.call(terms, k);
    if (inBase && !inNew) removed.push(k);
    else if (!inBase && inNew) added.push(k);
    else if (String(baseline[k]) !== String(terms[k])) changed.push({ field: k, baseline: baseline[k], current: terms[k] });
  }
  const unconstrained = [...changed.map((c) => c.field), ...added, ...removed].filter((f) => !constrainedFields.has(f));
  return { changed, added, removed, unconstrained_drift: unconstrained };
}

// Append one attestation to `target`'s log and return the new entry plus drift-vs-baseline (the
// first attestation ever recorded for this target establishes the baseline; there is nothing to
// drift from before that, so its own `drift` is null).
function attest(dir, target, terms, enforcement, constrainedFields) {
  fs.mkdirSync(dir, { recursive: true });
  const entries = readLog(dir, target);
  const prevHash = entries.length ? entries[entries.length - 1].hash : GENESIS;
  const baseline = entries.length ? entries[0].terms : null;
  const drift = entries.length ? diffTerms(baseline, terms, new Set(constrainedFields)) : null;

  const body = {
    seq: entries.length,
    ts: new Date().toISOString(),
    target,
    terms,
    terms_hash: sha256Hex(stableStringify(terms)),
    allowed: enforcement.allowed,
    violations: enforcement.violations,
    drift,
    prev_hash: prevHash,
  };
  const hash = sha256Hex(stableStringify(body) + "|" + prevHash);
  const entry = { ...body, hash };
  fs.appendFileSync(logPath(dir, target), JSON.stringify(entry) + "\n");
  return entry;
}

module.exports = { stableStringify, sha256Hex, readLog, verifyChain, diffTerms, attest };
