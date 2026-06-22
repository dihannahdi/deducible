// Iteration 2: three distinct REAL testnet accounts (bank=operator, client, valuer).
// Proves on-chain: (a) adversarial role enforcement (I4), (b) settlement reduces the
// financier's recoverable capital on a loss (the closed CRITICAL), with capital custody.
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const Long = require("long");
const {
  Client, AccountId, PrivateKey, Hbar,
  AccountCreateTransaction, AccountBalanceQuery,
  ContractCreateFlow, ContractFunctionParameters, ContractExecuteTransaction,
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
    if (want && evm === want) return k;
  }
  throw new Error("no candidate key matches HEDERA_EVM_ADDRESS");
}

function bytecodeOf(name) {
  const p = path.join(__dirname, "..", "artifacts", "contracts", name + ".sol", name + ".json");
  return JSON.parse(fs.readFileSync(p, "utf8")).bytecode.replace(/^0x/, "");
}

async function createAccount(payer, key, hbar) {
  let tx = new AccountCreateTransaction().setInitialBalance(new Hbar(hbar));
  if (typeof tx.setKeyWithoutAlias === "function") tx = tx.setKeyWithoutAlias(key.publicKey);
  else tx = tx.setKey(key.publicKey);
  const resp = await tx.execute(payer);
  return (await resp.getReceipt(payer)).accountId;
}

async function expectRevert(label, fn) {
  try { await fn(); return { check: label, result: "DID NOT REVERT (unexpected)" }; }
  catch (e) {
    const m = (e.message || String(e));
    return { check: label, result: "reverted as expected", detail: m.slice(0, 90) };
  }
}

async function tinybarBalance(client, id) {
  const b = await new AccountBalanceQuery().setAccountId(id).execute(client);
  return b.hbars.toTinybars();
}

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const bankClient = Client.forTestnet().setOperator(operatorId, operatorKey);
  bankClient.setDefaultMaxTransactionFee(new Hbar(20));

  console.log("- creating client + valuer accounts (funded from operator) ...");
  const clientKey = PrivateKey.generateECDSA();
  const valuerKey = PrivateKey.generateECDSA();
  const clientId = await createAccount(bankClient, clientKey, 30);
  const valuerId = await createAccount(bankClient, valuerKey, 40);
  const clientC = Client.forTestnet().setOperator(clientId, clientKey);
  const valuerC = Client.forTestnet().setOperator(valuerId, valuerKey);
  clientC.setDefaultMaxTransactionFee(new Hbar(20));
  valuerC.setDefaultMaxTransactionFee(new Hbar(20));
  const clientEvm = clientId.toSolidityAddress();
  console.log("  bank(operator)=" + operatorId + "  client=" + clientId + "  valuer=" + valuerId);

  const V0 = Long.fromString("100000000"); // 1e8 tinybar = 1 hbar
  const RENT_RATE = Long.fromString("10");

  console.log("- valuer deploys the oracle; bank deploys V2 ...");
  const oTx = await new ContractCreateFlow().setBytecode(bytecodeOf("MockValuationOracle"))
    .setGas(3000000).setConstructorParameters(new ContractFunctionParameters().addUint256(V0)).execute(valuerC);
  const oracleId = (await oTx.getReceipt(valuerC)).contractId;
  const mTx = await new ContractCreateFlow().setBytecode(bytecodeOf("MusharakahMutanaqisahV2"))
    .setGas(4000000).setConstructorParameters(new ContractFunctionParameters()
      .addAddress(clientEvm).addAddress(oracleId.toSolidityAddress()).addUint256(8000).addUint256(RENT_RATE))
    .execute(bankClient);
  const mId = (await mTx.getReceipt(bankClient)).contractId;
  console.log("  oracle=" + oracleId + "  musharakahV2=" + mId);

  const callV2 = (signer, fn, params, payTinybar, gas) => {
    let tx = new ContractExecuteTransaction().setContractId(mId).setGas(gas);
    tx = params ? tx.setFunction(fn, params) : tx.setFunction(fn);
    if (payTinybar) tx = tx.setPayableAmount(Hbar.fromTinybars(payTinybar));
    return tx.execute(signer).then(r => r.getReceipt(signer));
  };

  console.log("- funding both partners (capital custody) ...");
  await callV2(bankClient, "fundBank", null, 80000000, 400000);   // 80M tinybar = 0.8 hbar
  await callV2(clientC, "fundClient", null, 20000000, 400000);    // 20M tinybar = 0.2 hbar

  console.log("- adversarial role checks on-chain ...");
  const adversarial = [];
  adversarial.push(await expectRevert("client tries to attest value (I4)", async () => {
    const tx = new ContractExecuteTransaction().setContractId(oracleId).setGas(200000)
      .setFunction("attest", new ContractFunctionParameters().addUint256(Long.fromString("1")));
    await (await tx.execute(clientC)).getReceipt(clientC);
  }));
  adversarial.push(await expectRevert("valuer (non-partner) tries to settle", async () => {
    await callV2(valuerC, "settle", null, 0, 300000);
  }));
  adversarial.push(await expectRevert("bank tries fundClient (wrong role)", async () => {
    await callV2(bankClient, "fundClient", null, 20000000, 300000);
  }));

  console.log("- valuer attests a loss; a partner settles ...");
  const aTx = new ContractExecuteTransaction().setContractId(oracleId).setGas(200000)
    .setFunction("attest", new ContractFunctionParameters().addUint256(Long.fromString("90000000")));
  await (await aTx.execute(valuerC)).getReceipt(valuerC);

  const bankBefore = await tinybarBalance(bankClient, operatorId);
  await callV2(clientC, "settle", null, 0, 400000); // client pays gas, bank delta = pure payout
  const bankAfter = await tinybarBalance(bankClient, operatorId);
  const bankPayout = bankAfter.subtract(bankBefore).toString();

  const record = {
    network: "testnet",
    accounts: { bank: operatorId.toString(), client: clientId.toString(), valuer: valuerId.toString() },
    contracts: { oracle: oracleId.toString(), musharakahV2: mId.toString() },
    hashscan: "https://hashscan.io/testnet/contract/" + mId,
    adversarial: adversarial,
    settlement: {
      bankFundedTinybar: "80000000",
      attestedFairValueTinybar: "90000000",
      bankPayoutTinybar: bankPayout,
      expectedBankPayoutTinybar: "72000000",
      bankBoreLossTinybar: "8000000",
      cannotExitWhole: bankPayout === "72000000",
    },
  };
  fs.writeFileSync(path.join(__dirname, "..", "deployments", "testnet_v2.json"), JSON.stringify(record, null, 2));
  console.log("\n=== adversarial ===");
  adversarial.forEach(a => console.log("  " + a.check + " -> " + a.result));
  console.log("=== settlement ===");
  console.log("  bank funded 80,000,000; recovered " + bankPayout + " (expected 72,000,000)");
  console.log("  cannot-exit-whole enforced on-chain: " + record.settlement.cannotExitWhole);
  console.log("  musharakahV2: " + record.hashscan);
  bankClient.close(); clientC.close(); valuerC.close();
}

main().catch((e) => { console.error("RUN FAILED:", e.message || e); process.exit(1); });
