// Iteration 9: V3 loss path + on-chain dissolution.
// After a downward revaluation, a buyout costs less (loss borne via lower proceeds);
// dissolve() returns the bank's remaining escrowed units. Hedera testnet.
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const Long = require("long");
const {
  Client, AccountId, PrivateKey, Hbar,
  AccountCreateTransaction, AccountBalanceQuery, TokenAssociateTransaction, TransferTransaction,
  TokenCreateTransaction, TokenType, TokenSupplyType,
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
async function units(client, id, tokenId) {
  const b = await new AccountBalanceQuery().setAccountId(id).execute(client);
  const u = b.tokens.get(tokenId); return u ? u.toString() : "0";
}

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const bank = Client.forTestnet().setOperator(operatorId, operatorKey);
  bank.setDefaultMaxTransactionFee(new Hbar(20));

  console.log("- token + client ...");
  const tokenId = (await (await new TokenCreateTransaction()
    .setTokenName("MMS Loss Demo").setTokenSymbol("MMS9").setDecimals(0).setInitialSupply(10000)
    .setTreasuryAccountId(operatorId).setTokenType(TokenType.FungibleCommon)
    .setSupplyType(TokenSupplyType.Finite).setMaxSupply(10000).execute(bank)).getReceipt(bank)).tokenId;
  const tokenEvm = tokenId.toSolidityAddress();
  const clientKey = PrivateKey.generateECDSA();
  const cc = new AccountCreateTransaction().setInitialBalance(new Hbar(30));
  const ccTx = (typeof cc.setKeyWithoutAlias === "function" ? cc.setKeyWithoutAlias(clientKey.publicKey) : cc.setKey(clientKey.publicKey));
  const clientId = (await (await ccTx.execute(bank)).getReceipt(bank)).accountId;
  const clientC = Client.forTestnet().setOperator(clientId, clientKey); clientC.setDefaultMaxTransactionFee(new Hbar(20));
  await (await new TokenAssociateTransaction().setAccountId(clientId).setTokenIds([tokenId]).execute(clientC)).getReceipt(clientC);
  const clientEvm = clientId.toSolidityAddress();

  console.log("- oracle + V3 (with dissolve) ...");
  const oracleId = (await (await new ContractCreateFlow().setBytecode(bytecodeOf("MockValuationOracle"))
    .setGas(3000000).setConstructorParameters(new ContractFunctionParameters().addUint256(Long.fromString("100000000"))).execute(bank)).getReceipt(bank)).contractId;
  const v3Id = (await (await new ContractCreateFlow().setBytecode(bytecodeOf("MusharakahMutanaqisahV3")).setGas(2000000)
    .setConstructorParameters(new ContractFunctionParameters()
      .addAddress(clientEvm).addAddress(oracleId.toSolidityAddress()).addAddress(tokenEvm)
      .addUint64(Long.fromNumber(10000)).addUint64(Long.fromNumber(8000)).addUint256(Long.fromNumber(10))).execute(bank)).getReceipt(bank)).contractId;
  const v3Acct = AccountId.fromString(v3Id.toString());
  await (await new ContractExecuteTransaction().setContractId(v3Id).setGas(1500000).setFunction("associate").execute(bank)).getReceipt(bank);
  await (await new TransferTransaction().addTokenTransfer(tokenId, operatorId, -8000).addTokenTransfer(tokenId, v3Acct, 8000).execute(bank)).getReceipt(bank);
  console.log("  token=" + tokenId + " v3=" + v3Id + " client=" + clientId);

  // price for 2000 units at full value (1e8): 2e7 tinybar (reference)
  const priceFull = 20000000;
  // attest a 10% loss -> value 9e7; price for 2000 units now 1.8e7 tinybar
  console.log("- oracle attests a 10% loss; client buys 2000 units at the fallen price ...");
  await (await new ContractExecuteTransaction().setContractId(oracleId).setGas(200000)
    .setFunction("attest", new ContractFunctionParameters().addUint256(Long.fromString("90000000"))).execute(bank)).getReceipt(bank);
  const priceAfterLoss = 18000000; // 9e7 * 2000/10000
  await (await new ContractExecuteTransaction().setContractId(v3Id).setGas(1500000)
    .setFunction("buyShare", new ContractFunctionParameters().addUint64(Long.fromNumber(2000)))
    .setPayableAmount(Hbar.fromTinybars(priceAfterLoss)).execute(clientC)).getReceipt(clientC);
  const clientUnits = await units(bank, clientId, tokenId);

  console.log("- bank dissolves; remaining escrowed units return to the bank ...");
  const bankUnitsBefore = await units(bank, operatorId, tokenId);
  await (await new ContractExecuteTransaction().setContractId(v3Id).setGas(1500000).setFunction("dissolve").execute(bank)).getReceipt(bank);
  const bankUnitsAfter = await units(bank, operatorId, tokenId);
  const escrowAfter = await units(bank, v3Acct, tokenId);

  const record = {
    network: "testnet", token: tokenId.toString(), oracle: oracleId.toString(), musharakahV3: v3Id.toString(), client: clientId.toString(),
    lossBorneViaProceeds: { priceAtFullValueTinybar: priceFull, priceAfterLossTinybar: priceAfterLoss, bankReceivedLessTinybar: priceFull - priceAfterLoss },
    clientUnitsBought: clientUnits,
    dissolve: { bankUnitsBefore, bankUnitsAfter, escrowAfter, returnedToBank: Number(bankUnitsAfter) - Number(bankUnitsBefore) },
    cleanWindUp: escrowAfter === "0",
    hashscan: "https://hashscan.io/testnet/contract/" + v3Id,
  };
  fs.writeFileSync(path.join(__dirname, "..", "deployments", "testnet_v3_dissolve.json"), JSON.stringify(record, null, 2));

  console.log("\n=== V3 loss + dissolution ===");
  console.log("  buyout price: " + priceFull + " (full) -> " + priceAfterLoss + " (after 10% loss); bank received " + (priceFull - priceAfterLoss) + " less");
  console.log("  client bought units: " + clientUnits);
  console.log("  dissolve returned " + record.dissolve.returnedToBank + " units to bank; escrow now " + escrowAfter);
  console.log("  clean wind-up: " + record.cleanWindUp);
  console.log("  hashscan: " + record.hashscan);
  bank.close(); clientC.close();
}
main().catch((e) => { console.error("RUN FAILED:", e.message || e); process.exit(1); });
