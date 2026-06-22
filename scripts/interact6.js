// Iteration 6: Musharakah V3 (HTS-native) atomic buyout on Hedera testnet.
// buyShare moves real MMS units (contract->client) AND hbar (->bank) in one tx.
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const Long = require("long");
const {
  Client, AccountId, PrivateKey, Hbar, TokenId,
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
  const u = b.tokens.get(tokenId);
  return u ? u.toString() : "0";
}

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const bankClient = Client.forTestnet().setOperator(operatorId, operatorKey);
  bankClient.setDefaultMaxTransactionFee(new Hbar(20));

  console.log("- creating a fresh MMS asset token (10000 units) ...");
  const tokenId = (await (await new TokenCreateTransaction()
    .setTokenName("Musharakah Asset Share V3").setTokenSymbol("MMS3").setDecimals(0)
    .setInitialSupply(10000).setTreasuryAccountId(operatorId)
    .setTokenType(TokenType.FungibleCommon).setSupplyType(TokenSupplyType.Finite).setMaxSupply(10000)
    .execute(bankClient)).getReceipt(bankClient)).tokenId;
  const tokenEvm = tokenId.toSolidityAddress();
  console.log("  token=" + tokenId);

  console.log("- creating + associating client ...");
  const clientKey = PrivateKey.generateECDSA();
  const cc = new AccountCreateTransaction().setInitialBalance(new Hbar(30));
  const ccTx = (typeof cc.setKeyWithoutAlias === "function" ? cc.setKeyWithoutAlias(clientKey.publicKey) : cc.setKey(clientKey.publicKey));
  const clientId = (await (await ccTx.execute(bankClient)).getReceipt(bankClient)).accountId;
  const clientC = Client.forTestnet().setOperator(clientId, clientKey);
  clientC.setDefaultMaxTransactionFee(new Hbar(20));
  await (await new TokenAssociateTransaction().setAccountId(clientId).setTokenIds([tokenId]).execute(clientC)).getReceipt(clientC);
  const clientEvm = clientId.toSolidityAddress();
  console.log("  client=" + clientId);

  console.log("- deploying oracle + Musharakah V3 ...");
  const oracleId = (await (await new ContractCreateFlow().setBytecode(bytecodeOf("MockValuationOracle"))
    .setGas(3000000).setConstructorParameters(new ContractFunctionParameters().addUint256(Long.fromString("100000000"))).execute(bankClient)).getReceipt(bankClient)).contractId;
  const v3Id = (await (await new ContractCreateFlow().setBytecode(bytecodeOf("MusharakahMutanaqisahV3")).setGas(2000000)
    .setConstructorParameters(new ContractFunctionParameters()
      .addAddress(clientEvm).addAddress(oracleId.toSolidityAddress()).addAddress(tokenEvm)
      .addUint64(Long.fromNumber(10000)).addUint64(Long.fromNumber(8000)).addUint256(Long.fromNumber(10)))
    .execute(bankClient)).getReceipt(bankClient)).contractId;
  const v3Acct = AccountId.fromString(v3Id.toString());
  console.log("  oracle=" + oracleId + "  musharakahV3=" + v3Id);

  console.log("- contract associates the token; bank escrows its 8000 units ...");
  await (await new ContractExecuteTransaction().setContractId(v3Id).setGas(1500000).setFunction("associate").execute(bankClient)).getReceipt(bankClient);
  await (await new TransferTransaction().addTokenTransfer(tokenId, operatorId, -8000).addTokenTransfer(tokenId, v3Acct, 8000).execute(bankClient)).getReceipt(bankClient);

  const clientBefore = await units(bankClient, clientId, tokenId);
  const escrowBefore = await units(bankClient, v3Acct, tokenId);

  console.log("- ATOMIC buyShare(2000): MMS units -> client, hbar -> bank ...");
  await (await new ContractExecuteTransaction().setContractId(v3Id).setGas(1500000)
    .setFunction("buyShare", new ContractFunctionParameters().addUint64(Long.fromNumber(2000)))
    .setPayableAmount(Hbar.fromTinybars(20000000)) // 1e8 * 2000/10000 = 2e7 tinybar
    .execute(clientC)).getReceipt(clientC);

  const clientAfter = await units(bankClient, clientId, tokenId);
  const escrowAfter = await units(bankClient, v3Acct, tokenId);

  const record = {
    network: "testnet",
    token: tokenId.toString(), oracle: oracleId.toString(), musharakahV3: v3Id.toString(), client: clientId.toString(),
    before: { clientUnits: clientBefore, escrowUnits: escrowBefore },
    after: { clientUnits: clientAfter, escrowUnits: escrowAfter },
    atomicBuyout_unitsMoved: clientAfter === "2000" && escrowAfter === "6000",
    hashscan: "https://hashscan.io/testnet/contract/" + v3Id,
  };
  fs.writeFileSync(path.join(__dirname, "..", "deployments", "testnet_v3.json"), JSON.stringify(record, null, 2));

  console.log("\n=== V3 atomic buyout (units + hbar) ===");
  console.log("  client units: " + clientBefore + " -> " + clientAfter + " (received 2000 MMS from contract)");
  console.log("  escrow units: " + escrowBefore + " -> " + escrowAfter);
  console.log("  atomic unit+hbar buyout proven: " + record.atomicBuyout_unitsMoved);
  console.log("  hashscan: " + record.hashscan);
  bankClient.close(); clientC.close();
}

main().catch((e) => { console.error("RUN FAILED:", e.message || e); process.exit(1); });
