// Deploy the Musharakah Mutanaqisah artifact to Hedera testnet and read live state.
// Design-science evaluation step: prove the contract runs on the chosen network.
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const {
  Client,
  AccountId,
  PrivateKey,
  Hbar,
  ContractCreateFlow,
  ContractFunctionParameters,
  ContractCallQuery,
  AccountBalanceQuery,
} = require("@hashgraph/sdk");

// Self-select the operator key: among the candidate env vars, pick the one whose
// derived EVM address actually matches the account's EVM address. Robust against
// stale/mismatched key vars sitting in the same .env.
function selectOperatorKey() {
  const want = (process.env.HEDERA_EVM_ADDRESS || "").trim().toLowerCase().replace(/^0x/, "");
  const candidates = ["HEDERA_OPERATOR_KEY", "HEDERA_HEX_PRIVATE_KEY", "HEDERA_DER_PRIVATE_KEY"];
  let firstParseable = null, firstName = null;
  for (const v of candidates) {
    const raw = (process.env[v] || "").trim();
    if (!raw) continue;
    let k = null;
    for (const p of [() => PrivateKey.fromStringECDSA(raw), () => PrivateKey.fromStringDer(raw), () => PrivateKey.fromStringED25519(raw)]) {
      try { k = p(); break; } catch (_) {}
    }
    if (!k) continue;
    if (!firstParseable) { firstParseable = k; firstName = v; }
    let evm = "";
    try { evm = k.publicKey.toEvmAddress().toLowerCase(); } catch (_) {}
    if (want && evm === want) { console.log(`operator key: using ${v} (matches account EVM)`); return k; }
  }
  if (!want && firstParseable) { console.log(`operator key: using ${firstName}`); return firstParseable; }
  throw new Error("no candidate key derives to HEDERA_EVM_ADDRESS");
}

function bytecodeOf(name) {
  const p = path.join(__dirname, "..", "artifacts", "contracts", `${name}.sol`, `${name}.json`);
  const j = JSON.parse(fs.readFileSync(p, "utf8"));
  return j.bytecode.replace(/^0x/, ""); // hex string; the SDK hex-decodes it
}

async function readUint(client, contractId, fn) {
  const r = await new ContractCallQuery()
    .setContractId(contractId)
    .setGas(120000)
    .setFunction(fn)
    .setQueryPayment(new Hbar(2))
    .execute(client);
  return r.getUint256(0).toString();
}

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const clientEvm = (process.env.HEDERA_EVM_ADDRESS || "").trim().replace(/^0x/, "");
  if (clientEvm.length !== 40) throw new Error(`HEDERA_EVM_ADDRESS looks wrong (len ${clientEvm.length})`);

  const client = Client.forTestnet().setOperator(operatorId, operatorKey);
  client.setDefaultMaxTransactionFee(new Hbar(20));
  client.setDefaultMaxQueryPayment(new Hbar(5));

  const bal = await new AccountBalanceQuery().setAccountId(operatorId).execute(client);
  console.log(`operator ${operatorId.toString()} balance: ${bal.hbars.toString()}`);

  // 1) Deploy the independent valuation oracle (initial attested value 1,000,000).
  console.log("\n- deploying MockValuationOracle ...");
  const oracleTx = await new ContractCreateFlow()
    .setBytecode(bytecodeOf("MockValuationOracle"))
    .setGas(3000000)
    .setConstructorParameters(new ContractFunctionParameters().addUint256(1000000))
    .execute(client);
  const oracleId = (await oracleTx.getReceipt(client)).contractId;
  const oracleEvm = oracleId.toSolidityAddress();
  console.log(`  oracle contractId=${oracleId.toString()} evm=0x${oracleEvm}`);

  // 2) Deploy the Musharakah Mutanaqisah contract wired to that oracle.
  //    bank = operator (deployer); client = operator's EVM address; bank 80% / client 20%.
  console.log("\n- deploying MusharakahMutanaqisah ...");
  const mTx = await new ContractCreateFlow()
    .setBytecode(bytecodeOf("MusharakahMutanaqisah"))
    .setGas(4000000)
    .setConstructorParameters(
      new ContractFunctionParameters()
        .addAddress(clientEvm)
        .addAddress(oracleEvm)
        .addUint256(8000)
        .addUint256(1)
    )
    .execute(client);
  const mId = (await mTx.getReceipt(client)).contractId;
  const mEvm = mId.toSolidityAddress();
  console.log(`  musharakah contractId=${mId.toString()} evm=0x${mEvm}`);

  // 3) Read live state straight off the network (no signing) to prove it runs.
  console.log("\n- reading live on-chain state ...");
  const state = {
    bankShareBps: await readUint(client, mId, "bankShareBps"),
    clientShareBps: await readUint(client, mId, "clientShareBps"),
    assetValue: await readUint(client, mId, "assetValue"),
    initialAssetValue: await readUint(client, mId, "initialAssetValue"),
    rentDue: await readUint(client, mId, "rentDue"),
    oracleFairValue: await readUint(client, oracleId, "fairValue"),
  };
  console.log(state);

  // 4) Record the deployment for reproducibility (the paper's evidence trail).
  const record = {
    network: "testnet",
    deployedBy: operatorId.toString(),
    oracle: { contractId: oracleId.toString(), evm: `0x${oracleEvm}` },
    musharakah: { contractId: mId.toString(), evm: `0x${mEvm}` },
    state,
    hashscan: {
      oracle: `https://hashscan.io/testnet/contract/${oracleId.toString()}`,
      musharakah: `https://hashscan.io/testnet/contract/${mId.toString()}`,
    },
  };
  const outDir = path.join(__dirname, "..", "deployments");
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, "testnet.json"), JSON.stringify(record, null, 2));

  console.log("\n=== DEPLOYED (testnet) ===");
  console.log(`oracle      : ${record.hashscan.oracle}`);
  console.log(`musharakah  : ${record.hashscan.musharakah}`);
  console.log("record written to deployments/testnet.json");

  client.close();
}

main().catch((e) => {
  console.error("DEPLOY FAILED:", e.message || e);
  process.exit(1);
});
