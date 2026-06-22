require("dotenv").config();
const id = (process.env.HEDERA_OPERATOR_ID || "").trim();
const url = `https://testnet.mirrornode.hedera.com/api/v1/accounts/${id}`;
(async () => {
  try {
    const r = await fetch(url);
    const j = await r.json();
    console.log("account        :", id);
    console.log("evm_address    :", j.evm_address);
    console.log("key._type      :", j.key && j.key._type);
    console.log("key (public)   :", j.key && j.key.key);
    console.log("balance (tinybar):", j.balance && j.balance.balance);
  } catch (e) {
    console.log("mirror fetch failed:", e.message || e);
  }
})();
