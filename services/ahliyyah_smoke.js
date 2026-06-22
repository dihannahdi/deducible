// Deterministic smoke test for the ahliyyah (legal capacity) verification module.
// No server, no deps — `node services/ahliyyah_smoke.js`. Exit 0 = all pass.
const path = require("path");
const ahliyyah = require("./ahliyyah");

const reg = ahliyyah.loadRegistry(path.join(__dirname, "did_registry.json"));
let pass = 0, fail = 0;
function check(name, cond) {
  if (cond) { pass++; console.log("  ok   " + name); }
  else { fail++; console.log("  FAIL " + name); }
}
function reasons(did) {
  return ahliyyah.verifyCredential(did, reg[did]).reasons.map((r) => r.code);
}

console.log("ahliyyah module — per-credential capacity:");
check("adult solvent institution is capable", ahliyyah.verifyCredential("did:fiqh:bank-alpha", reg["did:fiqh:bank-alpha"]).capable === true);
check("adult solvent person is capable", ahliyyah.verifyCredential("did:fiqh:client-zayd", reg["did:fiqh:client-zayd"]).capable === true);
check("a minor (saghir) is refused AHL-MINOR", reasons("did:fiqh:minor-omar").includes("AHL-MINOR"));
check("the interdicted (safih) is refused AHL-SAFIH", reasons("did:fiqh:safih-qasim").includes("AHL-SAFIH"));
check("a bankrupt (taflis) is refused AHL-TAFLIS", reasons("did:fiqh:bankrupt-co").includes("AHL-TAFLIS"));
check("an un-KYC'd firm is refused AHL-KYC", reasons("did:fiqh:no-kyc-firm").includes("AHL-KYC"));
check("a sanctioned party is refused AHL-AML", reasons("did:fiqh:sanctioned-x").includes("AHL-AML"));
check("an unknown DID is refused AHL-UNKNOWN", ahliyyah.verifyCredential("did:fiqh:ghost", reg["did:fiqh:ghost"]).reasons.map((r) => r.code).includes("AHL-UNKNOWN"));

console.log("ahliyyah module — multi-party authorize (manifest principals):");
const okParties = { financier: "did:fiqh:bank-alpha", acquirer: "did:fiqh:client-zayd" };
const a1 = ahliyyah.authorize(reg, okParties, ["financier", "acquirer"]);
check("both capable principals => allCapable", a1.allCapable === true);

const badParties = { financier: "did:fiqh:bank-alpha", acquirer: "did:fiqh:minor-omar" };
const a2 = ahliyyah.authorize(reg, badParties, ["financier", "acquirer"]);
check("a minor acquirer => NOT allCapable", a2.allCapable === false);
check("the minor's failure is attributed to the acquirer", a2.parties.acquirer.capable === false);

const a3 = ahliyyah.authorize(reg, { financier: "did:fiqh:bank-alpha" }, ["financier", "acquirer"]);
check("a missing required principal => NOT allCapable", a3.allCapable === false && a3.missingPrincipals.includes("acquirer"));

console.log(`\nahliyyah smoke: ${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
