require("dotenv").config();
const { PrivateKey } = require("@hashgraph/sdk");
const want = (process.env.HEDERA_EVM_ADDRESS || "").trim().toLowerCase().replace(/^0x/, "");
const vars = ["HEDERA_OPERATOR_KEY", "HEDERA_HEX_PRIVATE_KEY", "HEDERA_DER_PRIVATE_KEY"];
console.log("want EVM:", want);
for (const v of vars) {
  const raw = (process.env[v] || "").trim();
  if (!raw) { console.log(`${v}: (absent)`); continue; }
  let evm = "unparseable";
  for (const p of [() => PrivateKey.fromStringECDSA(raw), () => PrivateKey.fromStringDer(raw)]) {
    try { evm = p().publicKey.toEvmAddress().toLowerCase(); break; } catch (e) {}
  }
  console.log(`${v}: len=${raw.length} evm=${evm} match=${evm === want}`);
}
