require("dotenv").config();
const { PrivateKey } = require("@hashgraph/sdk");
const raw = (process.env.HEDERA_OPERATOR_KEY || "").trim();
const wantEvm = (process.env.HEDERA_EVM_ADDRESS || "").trim().toLowerCase().replace(/^0x/, "");
console.log("key length:", raw.length, "prefix6:", raw.slice(0, 6));
console.log("want EVM :", wantEvm);
const parsers = {
  fromStringDer: () => PrivateKey.fromStringDer(raw),
  fromStringECDSA: () => PrivateKey.fromStringECDSA(raw),
  fromStringED25519: () => PrivateKey.fromStringED25519(raw),
};
for (const [name, fn] of Object.entries(parsers)) {
  try {
    const k = fn();
    let evm = "n/a";
    try { evm = k.publicKey.toEvmAddress().toLowerCase(); } catch (e) { evm = "no-evm"; }
    console.log(`${name}: OK  evm=${evm}  match=${evm === wantEvm}`);
  } catch (e) {
    console.log(`${name}: FAIL ${(e.message || e).toString().slice(0, 80)}`);
  }
}
