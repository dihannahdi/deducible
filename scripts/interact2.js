// Full step-down lifecycle on Hedera testnet: buy the partnership all the way to
// full client ownership, recording (bankShareBps, clientShareBps, rentDue) at each
// step -> real curve data for Figure 1 and the terminal-state proof (rent -> 0).
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const Long = require("long");
const {
  Client, AccountId, PrivateKey, Hbar,
  ContractCreateFlow, ContractFunctionParameters,
  ContractExecuteTransaction, ContractCallQuery, AccountBalanceQuery,
} = require("@hashgraph/sdk");

function selectOperatorKey() {
  const want = (process.env.HEDERA_EVM_ADDRESS || "").trim().toLowerCase().replace(/^0x/, "");
  for (const v of ["HEDERA_OPERATOR_KEY", "HEDERA_HEX_PRIVATE_KEY", "HEDERA_DER_PRIVATE_KEY"]) {
    const raw = (process.env[v] || "").trim();
    if (!raw) continue;
    let k = null;
    for (const p of [() => PrivateKey.fromStringECDSA(raw), () => PrivateKey.fromStringDer(raw)]) {
      try { k = p(); break; } catch (e) {}
    }
    if (!k) continue;
    let evm = ""; try { evm = k.publicKey.toEvmAddress().toLowerCase(); } catch (e) {}
    if (want && evm === want) { console.log("operator key: using " + v); return k; }
  }
  throw new Error("no candidate key matches HEDERA_EVM_ADDRESS");
}

function bytecodeOf(name) {
  const p = path.join(__dirname, "..", "artifacts", "contracts", name + ".sol", name + ".json");
  return JSON.parse(fs.readFileSync(p, "utf8")).bytecode.replace(/^0x/, "");
}

async function readUint(client, id, fn) {
  const r = await new ContractCallQuery().setContractId(id).setGas(120000)
    .setFunction(fn).setQueryPayment(new Hbar(2)).execute(client);
  return r.getUint256(0).toString();
}

async function row(client, mId, step) {
  return {
    step: step,
    bankShareBps: Number(await readUint(client, mId, "bankShareBps")),
    clientShareBps: Number(await readUint(client, mId, "clientShareBps")),
    rentDue: Number(await readUint(client, mId, "rentDue")),
  };
}

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const clientEvm = (process.env.HEDERA_EVM_ADDRESS || "").trim().replace(/^0x/, "");
  const client = Client.forTestnet().setOperator(operatorId, operatorKey);
  client.setDefaultMaxTransactionFee(new Hbar(20));

  const WHOLE = Long.fromString("100000000"); // 1e8 tinybar = 1 hbar
  const RENT_RATE = Long.fromString("10");     // rentDue = 10 * bankShareBps

  console.log("- deploying ...");
  const oTx = await new ContractCreateFlow().setBytecode(bytecodeOf("MockValuationOracle"))
    .setGas(3000000).setConstructorParameters(new ContractFunctionParameters().addUint256(WHOLE)).execute(client);
  const oracleId = (await oTx.getReceipt(client)).contractId;
  const mTx = await new ContractCreateFlow().setBytecode(bytecodeOf("MusharakahMutanaqisah"))
    .setGas(4000000).setConstructorParameters(new ContractFunctionParameters()
      .addAddress(clientEvm).addAddress(oracleId.toSolidityAddress()).addUint256(8000).addUint256(RENT_RATE))
    .execute(client);
  const mId = (await mTx.getReceipt(client)).contractId;
  console.log("  oracle=" + oracleId + "  musharakah=" + mId);

  const curve = [];
  curve.push(await row(client, mId, 0));
  console.log("  step 0:", JSON.stringify(curve[0]));

  // price per 2000 bps = WHOLE * 2000 / 10000 = 2e7 tinybar
  const pricePer2000 = Number((BigInt("100000000") * 2000n) / 10000n);
  for (let i = 1; i <= 4; i++) {
    const tx = new ContractExecuteTransaction().setContractId(mId).setGas(500000)
      .setFunction("buyShare", new ContractFunctionParameters().addUint256(2000))
      .setPayableAmount(Hbar.fromTinybars(pricePer2000));
    const resp = await tx.execute(client);
    await resp.getReceipt(client);
    const r = await row(client, mId, i);
    curve.push(r);
    console.log("  step " + i + ":", JSON.stringify(r));
  }

  const fullyAcquired = (await readUint(client, mId, "bankShareBps")) === "0";
  console.log("  fullyAcquired:", fullyAcquired);

  const record = {
    network: "testnet",
    contracts: { oracle: oracleId.toString(), musharakah: mId.toString() },
    hashscan: "https://hashscan.io/testnet/contract/" + mId,
    curve: curve,
    fullyAcquired: fullyAcquired,
  };
  fs.writeFileSync(path.join(__dirname, "..", "deployments", "stepdown.json"), JSON.stringify(record, null, 2));
  console.log("recorded to deployments/stepdown.json");
  client.close();
}

main().catch((e) => { console.error("RUN FAILED:", e.message || e); process.exit(1); });
