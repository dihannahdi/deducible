// An autonomous valuation agent. Each agent independently reasons over its OWN market
// evidence (zero-trust fact-finding) via DeepSeek and produces a fair-value estimate, then
// signs (round, value) with its own key. No agent sees the others; consensus emerges on-chain.
const { ethers } = require("hardhat");

async function deriveValue({ asset, evidence }) {
  const base = (process.env.DEEPSEEK_BASE_URL || "https://api.deepseek.com").replace(/\/+$/, "");
  const model = process.env.DEEPSEEK_MODEL || "deepseek-chat";
  const key = process.env.DEEPSEEK_API_KEY;
  if (!key) throw new Error("DEEPSEEK_API_KEY not set");

  const prompt =
    `You are an independent asset valuer performing zero-trust fact-finding.\n` +
    `Asset: ${asset}\n` +
    `Your own independent market comparables, in ABSOLUTE accounting units: ${evidence.join(", ")}.\n` +
    `Weigh your comparables and give your single best fair-value estimate in the SAME absolute units.\n` +
    `Reply with ONLY strict JSON and nothing else: {"value": <integer>}`;

  const res = await fetch(`${base}/chat/completions`, {
    method: "POST",
    headers: { Authorization: `Bearer ${key}`, "Content-Type": "application/json" },
    body: JSON.stringify({ model, temperature: 0, stream: false, messages: [{ role: "user", content: prompt }] }),
  });
  const j = await res.json();
  const content = (j && j.choices && j.choices[0] && j.choices[0].message && j.choices[0].message.content) || "";
  const txt = content.replace(/```/g, "");
  const m = txt.match(/"value"\s*:\s*"?([\d,]+)/) || txt.match(/([\d][\d,]{2,})/);
  if (!m) throw new Error("agent could not parse a value from: " + content.slice(0, 200));
  return BigInt(m[1].replace(/,/g, ""));
}

async function signAttestation(wallet, oracleAddr, chainId, roundId, value) {
  const digest = ethers.solidityPackedKeccak256(
    ["uint256", "uint256", "address", "uint256"],
    [roundId, value, oracleAddr, chainId]
  );
  return await wallet.signMessage(ethers.getBytes(digest));
}

module.exports = { deriveValue, signAttestation };
