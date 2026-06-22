// V2-C/D — full local proof of the zero-trust valuation stack, run on the in-memory Hardhat
// network (free). Five autonomous DeepSeek agents independently value an asset, sign their
// attestations, and the ConsensusValuationOracle resolves the median within the gharar band.
// A generated Musharakah(consensus) then runs its lifecycle off the consensus value. Finally a
// divergent scenario shows the committee disagreeing -> majhul -> fairValue() reverts (gharar).
//
//   npx hardhat run scripts/consensus_demo.js
require("dotenv").config();
const { ethers } = require("hardhat");
const { deriveValue, signAttestation } = require("../agents/valuer_agent");

// round to a multiple of 1000 so the 80/20 capital split divides exactly (realistic rounding).
function clean(v) {
  let r = (v / 1000n) * 1000n;
  return r < 1000n ? 1000n : r;
}

async function runRound(oracle, wallets, chainId, roundId, asset, evidences) {
  const oracleAddr = await oracle.getAddress();
  const values = [];
  const sigs = [];
  for (let i = 0; i < wallets.length; i++) {
    const raw = await deriveValue({ asset, evidence: evidences[i] });
    const v = clean(raw);
    const sig = await signAttestation(wallets[i], oracleAddr, chainId, roundId, v);
    values.push(v);
    sigs.push(sig);
    console.log(`    agent ${i} ${wallets[i].address.slice(0, 12)}… reasoned value = ${v.toString()}`);
  }
  return { values, sigs };
}

async function main() {
  const [relayer, bank, client, , arbiter, maslahah] = await ethers.getSigners();
  const chainId = (await ethers.provider.getNetwork()).chainId;
  const wallets = Array.from({ length: 5 }, () => ethers.Wallet.createRandom());
  const committee = wallets.map((w) => w.address);
  const OF = await ethers.getContractFactory("ConsensusValuationOracle");

  // ---------- Scenario A: convergent fact-finding -> consensus resolves ----------
  console.log("Scenario A — convergent: 5 independent DeepSeek agents value the asset");
  let oracle = await OF.connect(relayer).deploy(committee, 3, 500);
  await oracle.waitForDeployment();
  const convergent = [
    [980000, 1000000, 1010000],
    [1000000, 1005000, 995000],
    [990000, 1000000, 1008000],
    [1002000, 998000, 1000000],
    [995000, 1000000, 1005000],
  ];
  let r = await runRound(oracle, wallets, chainId, 1n, "a 60 sqm apartment in a prime district", convergent);
  await (await oracle.connect(relayer).submitRound(1n, r.values, r.sigs)).wait();
  const fair = await oracle.fairValue();
  console.log("  => consensus fairValue =", fair.toString(), "(median of inliers, within the 5% gharar band)");

  // ---------- run a generated Musharakah(consensus) off the consensus value ----------
  const v0 = fair;
  const MF = await ethers.getContractFactory("MusharakahConsensusGen");
  const m = await MF.connect(bank).deploy(client.address, await oracle.getAddress(), arbiter.address, maslahah.address, 8000n, 1n, 3600);
  await m.waitForDeployment();
  await (await m.connect(bank).fundBank({ value: (v0 * 8000n) / 10000n })).wait();
  await (await m.connect(client).fundClient({ value: (v0 * 2000n) / 10000n })).wait();
  await (await m.connect(client).payRent({ value: 8000n })).wait();
  await (await m.connect(client).buyShare(2000n, { value: (v0 * 2000n) / 10000n })).wait();
  console.log(
    "  => Musharakah(consensus) ran off the consensus price; bankShareBps",
    (await m.bankShareBps()).toString(),
    "clientShareBps",
    (await m.clientShareBps()).toString()
  );

  // ---------- Scenario B: divergent fact-finding -> majhul (gharar) ----------
  console.log("Scenario B — divergent: the same committee faces conflicting comparables");
  let oracle2 = await OF.connect(relayer).deploy(committee, 3, 500);
  await oracle2.waitForDeployment();
  const divergent = [
    [500000, 600000, 550000],
    [1000000, 1100000, 950000],
    [1800000, 2000000, 1900000],
    [300000, 350000, 320000],
    [2500000, 3000000, 2800000],
  ];
  let r2 = await runRound(oracle2, wallets, chainId, 1n, "a disputed asset with wildly conflicting comparables", divergent);
  await (await oracle2.connect(relayer).submitRound(1n, r2.values, r2.sigs)).wait();
  try {
    const v = await oracle2.fairValue();
    console.log("  !! UNEXPECTED: resolved to", v.toString());
    process.exit(1);
  } catch (e) {
    console.log("  => fairValue() reverted: the value is majhul (gharar) — the contract may not transact on it");
  }

  console.log("\nLOCAL OK: autonomous agents -> consensus oracle -> compliant contract, end to end");
}

main().then(() => process.exit(0)).catch((e) => { console.error(e); process.exit(1); });
