// Iteration: V5 corrections on testnet — maslahah disposition of the loss residue, and
// authoritative judicial faskh — with real distinct accounts (bank, client, arbiter, maslahah).
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
    for (const p of [() => PrivateKey.fromStringECDSA(raw), () => PrivateKey.fromStringDer(raw)]) { try { k = p(); break; } catch (e) {} }
    if (!k) continue;
    let evm = ""; try { evm = k.publicKey.toEvmAddress().toLowerCase(); } catch (e) {}
    if (want && evm === want) return k;
  }
  throw new Error("no key matches HEDERA_EVM_ADDRESS");
}
function bytecodeOf(name) {
  const p = path.join(__dirname, "..", "artifacts", "contracts", name + ".sol", name + ".json");
  return JSON.parse(fs.readFileSync(p, "utf8")).bytecode.replace(/^0x/, "");
}
async function mkAcct(payer, key, hbar) {
  let tx = new AccountCreateTransaction().setInitialBalance(new Hbar(hbar));
  tx = (typeof tx.setKeyWithoutAlias === "function" ? tx.setKeyWithoutAlias(key.publicKey) : tx.setKey(key.publicKey));
  return (await (await tx.execute(payer)).getReceipt(payer)).accountId;
}
async function tinybars(client, id) { return (await new AccountBalanceQuery().setAccountId(id).execute(client)).hbars.toTinybars(); }

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const bank = Client.forTestnet().setOperator(operatorId, selectOperatorKey());
  bank.setDefaultMaxTransactionFee(new Hbar(20));

  const ck = PrivateKey.generateECDSA(), ak = PrivateKey.generateECDSA(), mk = PrivateKey.generateECDSA();
  const clientId = await mkAcct(bank, ck, 30), arbiterId = await mkAcct(bank, ak, 30), maslahahId = await mkAcct(bank, mk, 2);
  const clientC = Client.forTestnet().setOperator(clientId, ck); clientC.setDefaultMaxTransactionFee(new Hbar(20));
  const arbiterC = Client.forTestnet().setOperator(arbiterId, ak); arbiterC.setDefaultMaxTransactionFee(new Hbar(20));
  console.log("bank=" + operatorId + " client=" + clientId + " arbiter=" + arbiterId + " maslahah=" + maslahahId);

  const oracleId = (await (await new ContractCreateFlow().setBytecode(bytecodeOf("MockValuationOracle"))
    .setGas(3000000).setConstructorParameters(new ContractFunctionParameters().addUint256(Long.fromString("100000000"))).execute(bank)).getReceipt(bank)).contractId;

  function deployV5() {
    return new ContractCreateFlow().setBytecode(bytecodeOf("MusharakahMutanaqisahV5")).setGas(4500000)
      .setConstructorParameters(new ContractFunctionParameters()
        .addAddress(clientId.toSolidityAddress()).addAddress(oracleId.toSolidityAddress())
        .addAddress(arbiterId.toSolidityAddress()).addAddress(maslahahId.toSolidityAddress())
        .addUint256(8000).addUint256(1).addUint256(3600)).execute(bank);
  }
  const A = (await (await deployV5()).getReceipt(bank)).contractId;
  const B = (await (await deployV5()).getReceipt(bank)).contractId;
  console.log("V5 A=" + A + "  V5 B=" + B);

  const fund = async (id) => {
    await (await new ContractExecuteTransaction().setContractId(id).setGas(400000).setFunction("fundBank").setPayableAmount(Hbar.fromTinybars(80000000)).execute(bank)).getReceipt(bank);
    await (await new ContractExecuteTransaction().setContractId(id).setGas(400000).setFunction("fundClient").setPayableAmount(Hbar.fromTinybars(20000000)).execute(clientC)).getReceipt(clientC);
  };
  await fund(A); await fund(B);

  // A: attest a 10% loss, then settle -> maslahah receives the impaired residue (0.1 hbar = 1e7 tinybar)
  await (await new ContractExecuteTransaction().setContractId(oracleId).setGas(200000).setFunction("attest", new ContractFunctionParameters().addUint256(Long.fromString("90000000"))).execute(bank)).getReceipt(bank);
  const masBefore = await tinybars(bank, maslahahId);
  await (await new ContractExecuteTransaction().setContractId(A).setGas(1200000).setFunction("settle").execute(bank)).getReceipt(bank);
  const masAfter = await tinybars(bank, maslahahId);
  const maslahahReceived = masAfter.subtract(masBefore).toString();

  // B: arbiter rescinds by authority (judicial faskh)
  await (await new ContractExecuteTransaction().setContractId(B).setGas(1200000).setFunction("judicialFaskh").execute(arbiterC)).getReceipt(arbiterC);

  // read B.rescinded via a query call
  const { ContractCallQuery } = require("@hashgraph/sdk");
  const r = await new ContractCallQuery().setContractId(B).setGas(60000).setFunction("rescinded").setQueryPayment(new Hbar(2)).execute(bank);
  const rescinded = r.getBool(0);

  const record = {
    network: "testnet", accounts: { bank: operatorId.toString(), client: clientId.toString(), arbiter: arbiterId.toString(), maslahah: maslahahId.toString() },
    contracts: { oracle: oracleId.toString(), v5_settle: A.toString(), v5_faskh: B.toString() },
    maslahahDisposition: { expectedTinybar: "10000000", receivedTinybar: maslahahReceived, ok: maslahahReceived === "10000000" },
    judicialFaskh: { arbiterRescinded: rescinded },
    hashscan: "https://hashscan.io/testnet/contract/" + A,
  };
  fs.writeFileSync(path.join(__dirname, "..", "deployments", "testnet_v5.json"), JSON.stringify(record, null, 2));
  console.log("\n=== V5 corrections on testnet ===");
  console.log("  maslahah received residue: " + maslahahReceived + " tinybar (expected 10000000): " + record.maslahahDisposition.ok);
  console.log("  judicial faskh by arbiter -> rescinded: " + rescinded);
  console.log("  hashscan: " + record.hashscan);
  bank.close(); clientC.close(); arbiterC.close();
}
main().catch((e) => { console.error("RUN FAILED:", e.message || e); process.exit(1); });
