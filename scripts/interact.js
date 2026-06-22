// Live lifecycle of a Musharakah Mutanaqisah on Hedera testnet, denominated in
// weibar (1 hbar = 1e18 weibar = 1e8 tinybar), capturing gas + fees per operation.
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const Long = require("long");
const {
  Client, AccountId, PrivateKey, Hbar,
  ContractCreateFlow, ContractFunctionParameters,
  ContractExecuteTransaction, ContractCallQuery, AccountBalanceQuery,
} = require("@hashgraph/sdk");

const WEIBAR_PER_TINYBAR = 10000000000n;

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

async function snapshot(client, mId, oracleId) {
  return {
    bankShareBps: await readUint(client, mId, "bankShareBps"),
    clientShareBps: await readUint(client, mId, "clientShareBps"),
    assetValue: await readUint(client, mId, "assetValue"),
    rentDue: await readUint(client, mId, "rentDue"),
    oracleFairValue: await readUint(client, oracleId, "fairValue"),
  };
}

const tinybarFromWeibar = (wstr) => Number(BigInt(wstr) / WEIBAR_PER_TINYBAR);

async function exec(client, id, fn, params, payableTinybars, gas, label) {
  let tx = new ContractExecuteTransaction().setContractId(id).setGas(gas);
  tx = params ? tx.setFunction(fn, params) : tx.setFunction(fn);
  if (payableTinybars) tx = tx.setPayableAmount(Hbar.fromTinybars(payableTinybars));
  const resp = await tx.execute(client);
  const rec = await resp.getRecord(client);
  const row = {
    op: label,
    status: rec.receipt.status.toString(),
    gasUsed: rec.contractFunctionResult ? rec.contractFunctionResult.gasUsed.toString() : null,
    feeTinybar: rec.transactionFee.toTinybars().toString(),
    feeHbar: rec.transactionFee.toString(),
    txId: resp.transactionId.toString(),
  };
  console.log("  " + label + ": " + row.status + "  gas=" + row.gasUsed + "  fee=" + row.feeHbar);
  return row;
}

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const clientEvm = (process.env.HEDERA_EVM_ADDRESS || "").trim().replace(/^0x/, "");
  const client = Client.forTestnet().setOperator(operatorId, operatorKey);
  client.setDefaultMaxTransactionFee(new Hbar(20));

  const balStart = (await new AccountBalanceQuery().setAccountId(operatorId).execute(client)).hbars;
  console.log("operator " + operatorId + " start balance: " + balStart.toString());

  const WHOLE = Long.fromString("100000000");   // 1e8 tinybar = 1 hbar (asset value)
  const LOWER = Long.fromString("90000000");    // 9e7 tinybar = 0.9 hbar (revalued down)
  const RENT_RATE = Long.fromString("10");      // rentDue = 10 * bankShareBps tinybar

  const fees = [];
  console.log("\n- deploying oracle + musharakah (weibar-denominated) ...");
  const oTx = await new ContractCreateFlow().setBytecode(bytecodeOf("MockValuationOracle"))
    .setGas(3000000).setConstructorParameters(new ContractFunctionParameters().addUint256(WHOLE)).execute(client);
  const oRec = await oTx.getRecord(client);
  const oracleId = oRec.receipt.contractId;
  fees.push({ op: "deploy oracle", status: oRec.receipt.status.toString(),
    gasUsed: oRec.contractFunctionResult ? oRec.contractFunctionResult.gasUsed.toString() : null,
    feeHbar: oRec.transactionFee.toString() });

  const mTx = await new ContractCreateFlow().setBytecode(bytecodeOf("MusharakahMutanaqisah"))
    .setGas(4000000).setConstructorParameters(new ContractFunctionParameters()
      .addAddress(clientEvm).addAddress(oracleId.toSolidityAddress()).addUint256(8000).addUint256(RENT_RATE))
    .execute(client);
  const mRec = await mTx.getRecord(client);
  const mId = mRec.receipt.contractId;
  fees.push({ op: "deploy musharakah", status: mRec.receipt.status.toString(),
    gasUsed: mRec.contractFunctionResult ? mRec.contractFunctionResult.gasUsed.toString() : null,
    feeHbar: mRec.transactionFee.toString() });
  console.log("  oracle=" + oracleId + "  musharakah=" + mId);

  const s0 = await snapshot(client, mId, oracleId);
  console.log("\n  state[0] initial:", s0);

  console.log("\n- live lifecycle ...");
  fees.push(await exec(client, mId, "payRent", null, Number(s0.rentDue), 400000, "payRent (bank 80%)"));
  const s1 = await snapshot(client, mId, oracleId);

  const buyPriceTinybar = (BigInt(s0.oracleFairValue) * 2000n) / 10000n;
  fees.push(await exec(client, mId, "buyShare", new ContractFunctionParameters().addUint256(2000),
    Number(buyPriceTinybar), 500000, "buyShare 2000bps"));
  const s2 = await snapshot(client, mId, oracleId);

  fees.push(await exec(client, oracleId, "attest", new ContractFunctionParameters().addUint256(LOWER), null, 200000, "oracle.attest down"));
  fees.push(await exec(client, mId, "syncValuation", null, null, 400000, "syncValuation loss"));
  const s3 = await snapshot(client, mId, oracleId);

  console.log("\n  state[1] after payRent :", s1);
  console.log("  state[2] after buyShare:", s2);
  console.log("  state[3] after loss    :", s3);

  const balEnd = (await new AccountBalanceQuery().setAccountId(operatorId).execute(client)).hbars;

  const record = {
    network: "testnet", account: operatorId.toString(),
    contracts: { oracle: oracleId.toString(), musharakah: mId.toString() },
    hashscan: {
      oracle: "https://hashscan.io/testnet/contract/" + oracleId,
      musharakah: "https://hashscan.io/testnet/contract/" + mId,
    },
    states: { s0: s0, s1_afterPayRent: s1, s2_afterBuyShare: s2, s3_afterLoss: s3 },
    gasAndFees: fees,
    balanceStart: balStart.toString(), balanceEnd: balEnd.toString(),
  };
  const outDir = path.join(__dirname, "..", "deployments");
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, "testnet_run.json"), JSON.stringify(record, null, 2));
  console.log("\n=== run recorded to deployments/testnet_run.json ===");
  console.log("musharakah: " + record.hashscan.musharakah);
  client.close();
}

main().catch((e) => { console.error("RUN FAILED:", e.message || e); process.exit(1); });
