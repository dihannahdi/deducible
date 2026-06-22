// End-to-end test of the gateway's /authorize endpoint (ahliyyah + invariants together).
// Requires the manifest to exist (run `fiqhc build --target manifest` first).
// `node services/authorize_e2e.js` — exit 0 = all pass.
require("./invariant_gateway"); // starts listening on 127.0.0.1:8799
const http = require("http");
const PORT = process.env.GATEWAY_PORT || 8799;

function post(p, body) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify(body);
    const r = http.request(
      { host: "127.0.0.1", port: PORT, path: p, method: "POST", headers: { "Content-Type": "application/json", "Content-Length": Buffer.byteLength(data) } },
      (resp) => { let d = ""; resp.on("data", (x) => (d += x)); resp.on("end", () => resolve(JSON.parse(d))); }
    );
    r.on("error", reject);
    r.write(data);
    r.end();
  });
}

(async () => {
  await new Promise((r) => setTimeout(r, 300));
  const target = "MusharakahMutanaqisahGen";
  const goodTerms = {
    "risk.capital_guarantee": "none",
    "risk.loss": "proportional_to_ownership",
    "returns.rent.basis": "bank.share",
    "returns.buyout.priceSource": "oracle",
  };
  let pass = 0, fail = 0;
  const check = (n, c) => (c ? (pass++, console.log("  ok   " + n)) : (fail++, console.log("  FAIL " + n)));

  const a = await post("/authorize", { target, terms: goodTerms, parties: { financier: "did:fiqh:bank-alpha", acquirer: "did:fiqh:client-zayd" } });
  check("valid terms + capable parties => ALLOWED", a.allowed === true);

  const b = await post("/authorize", { target, terms: goodTerms, parties: { financier: "did:fiqh:bank-alpha", acquirer: "did:fiqh:minor-omar" } });
  check("valid terms + MINOR acquirer => REFUSED", b.allowed === false && b.ahliyyah.parties.acquirer.reasons.some((r) => r.code === "AHL-MINOR"));

  const c = await post("/authorize", { target, terms: goodTerms, parties: { financier: "did:fiqh:bankrupt-co", acquirer: "did:fiqh:client-zayd" } });
  check("valid terms + BANKRUPT financier => REFUSED (taflis)", c.allowed === false && c.ahliyyah.parties.financier.reasons.some((r) => r.code === "AHL-TAFLIS"));

  const d = await post("/authorize", { target, terms: { "risk.capital_guarantee": "bank" }, parties: { financier: "did:fiqh:bank-alpha", acquirer: "did:fiqh:client-zayd" } });
  check("RIBA terms + capable parties => REFUSED (invariants)", d.allowed === false && d.invariants.violations.some((v) => v.code === "RIBA-1"));

  const e = await post("/authorize", { target, terms: goodTerms, parties: { financier: "did:fiqh:bank-alpha" } });
  check("missing principal (acquirer) => REFUSED", e.allowed === false && e.ahliyyah.missingPrincipals.includes("acquirer"));

  console.log(`\nauthorize E2E: ${pass} passed, ${fail} failed`);
  process.exit(fail === 0 ? 0 : 1);
})();
