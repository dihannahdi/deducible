// Ahliyyah (legal capacity) + DID verification for the invariant gateway — enterprise vector #3.
//
// Fiqh is not only about the CONTENT of a contract, but about WHO contracts. An 'aqd is valid
// only if each contracting party possesses ahliyyat al-ada' — the capacity for execution:
// bulugh (majority), 'aql (sanity), and rushd (prudence, the absence of safah). One who is a
// minor (saghir), insane (majnun), prodigal (safih), or a bankrupt placed under judicial
// interdiction (taflis / hajr) cannot validly dispose of wealth — their tasarruf is void or
// suspended. The gateway resolves each party's Decentralized Identifier (DID) to a credential
// and verifies this BEFORE a compiled contract is allowed to execute. On top of the fiqh it
// enforces the statutory overlay every regulated institution must satisfy: KYC and AML.
//
// Citations [scholar-verify]:
//   - ahliyyat al-ada' and its conditions (bulugh, 'aql, rushd): usul al-fiqh, e.g. al-Sarakhsi,
//     al-Usul; the conditions of the 'aqid in the four madhahib.
//   - hajr 'ala al-safih (interdiction of the prodigal): al-Baqarah 2:282 ("...wa in kana
//     alladhi 'alayhi al-haqq safihan aw da'ifan..."); al-Nisa' 4:5 on not giving the
//     foolish (sufaha') their wealth.
//   - taflis and hajr on the muflis: Sahih Muslim (the report of the man whose debts
//     overwhelmed him and the Prophet's ﷺ direction regarding his creditors).
//
// Epistemics: the engine verifies a declared, citation-bearing set of capacity conditions
// against asserted credentials. The truth of a credential (is this person actually solvent?)
// is the issuer's attestation; the fiqh ruling on capacity is the scholar's. No fatwa is issued.

// Each check: `disqualifyIf` true => the claim's presence disqualifies; false => its ABSENCE
// disqualifies (a positive credential, e.g. KYC, must be present and true).
const CHECKS = [
  { key: "minor",          disqualifyIf: true,  code: "AHL-MINOR",  message: "party is a minor (saghir) — lacks bulugh, no capacity to contract", citation: "ahliyyat al-ada' requires bulugh [scholar-verify]" },
  { key: "interdicted",    disqualifyIf: true,  code: "AHL-SAFIH",  message: "party is interdicted (mahjur 'alayh: safih / majnun) — lacks rushd / 'aql", citation: "hajr 'ala al-safih — al-Baqarah 2:282; al-Nisa' 4:5 [scholar-verify]" },
  { key: "bankrupt",       disqualifyIf: true,  code: "AHL-TAFLIS", message: "party is a bankrupt under interdiction (taflis) — the estate is reserved for creditors", citation: "hajr on the muflis — Sahih Muslim [scholar-verify]" },
  { key: "kyc",            disqualifyIf: false, code: "AHL-KYC",    message: "party has not completed KYC", citation: "statutory KYC overlay" },
  { key: "aml_sanctioned", disqualifyIf: true,  code: "AHL-AML",    message: "party is under AML / sanctions screening", citation: "statutory AML overlay" },
];

// Verify a single DID credential. Returns { did, capable, reasons:[{code,message,citation}] }.
function verifyCredential(did, cred) {
  if (!cred) {
    return { did, capable: false, reasons: [{ code: "AHL-UNKNOWN", message: "no DID credential found in the registry", citation: "" }] };
  }
  const reasons = [];
  for (const c of CHECKS) {
    const v = cred[c.key];
    const fail = c.disqualifyIf ? v === true : v !== true;
    if (fail) reasons.push({ code: c.code, message: c.message, citation: c.citation });
  }
  return { did, capable: reasons.length === 0, reasons };
}

function loadRegistry(file) {
  const fs = require("fs");
  try {
    return JSON.parse(fs.readFileSync(file, "utf8"));
  } catch (e) {
    return {};
  }
}

// Verify every party in a { role: did } map against the registry. If `requiredRoles` is given
// (from the manifest's ahliyyah.principals), a missing principal is itself a failure.
function authorize(registry, parties, requiredRoles) {
  parties = parties || {};
  const results = {};
  let allCapable = true;
  for (const [role, did] of Object.entries(parties)) {
    const r = verifyCredential(did, registry[did]);
    results[role] = r;
    if (!r.capable) allCapable = false;
  }
  const missing = (requiredRoles || []).filter((role) => !(role in parties));
  if (missing.length) allCapable = false;
  return { allCapable, parties: results, missingPrincipals: missing };
}

module.exports = { verifyCredential, loadRegistry, authorize, CHECKS };
