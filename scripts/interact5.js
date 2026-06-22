// Iteration 5: bind the enforcement contract to the real HTS asset token.
// Deploy AssetTokenCustodian, associate it with MMS, fund it units, and have the
// CONTRACT transfer units to a freshly-associated client -> contract-mediated custody
// and movement of the real tokenized asset on Hedera testnet.
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const {
  Client, AccountId, PrivateKey, Hbar, TokenId, ContractId,
  AccountCreateTransaction, AccountBalanceQuery, TokenAssociateTransaction, TransferTransaction,
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

async function tokenUnits(client, id, tokenId) {
  const b = await new AccountBalanceQuery().setAccountId(id).execute(client);
  const u = b.tokens.get(tokenId);
  return u ? u.toString() : "0";
}

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const bankClient = Client.forTestnet().setOperator(operatorId, operatorKey);
  bankClient.setDefaultMaxTransactionFee(new Hbar(20));

  const tokRec = JSON.parse(fs.readFileSync(path.join(__dirname, "..", "deployments", "tokenized_asset.json"), "utf8"));
  const tokenId = TokenId.fromString(tokRec.token.id);
  const tokenEvm = tokenId.toSolidityAddress();
  console.log("- using HTS asset token " + tokenId + " (evm 0x" + tokenEvm + ")");

  console.log("- creating + associating a fresh client ...");
  const clientKey = PrivateKey.generateECDSA();
  const cc = new AccountCreateTransaction().setInitialBalance(new Hbar(30));
  const ccTx = (typeof cc.setKeyWithoutAlias === "function" ? cc.setKeyWithoutAlias(clientKey.publicKey) : cc.setKey(clientKey.publicKey));
  const clientId = (await (await ccTx.execute(bankClient)).getReceipt(bankClient)).accountId;
  const clientClient = Client.forTestnet().setOperator(clientId, clientKey);
  clientClient.setDefaultMaxTransactionFee(new Hbar(20));
  await (await new TokenAssociateTransaction().setAccountId(clientId).setTokenIds([tokenId]).execute(clientClient)).getReceipt(clientClient);
  const clientEvm = clientId.toSolidityAddress();
  console.log("  client=" + clientId);

  console.log("- deploying AssetTokenCustodian ...");
  const dTx = await new ContractCreateFlow().setBytecode(bytecodeOf("AssetTokenCustodian")).setGas(1500000).execute(bankClient);
  const contractId = (await dTx.getReceipt(bankClient)).contractId;
  const contractAcct = AccountId.fromString(contractId.toString());
  console.log("  custodian=" + contractId);

  console.log("- contract associates itself with the token ...");
  await (await new ContractExecuteTransaction().setContractId(contractId).setGas(1500000)
    .setFunction("associate", new ContractFunctionParameters().addAddress(tokenEvm)).execute(bankClient)).getReceipt(bankClient);

  console.log("- treasury funds the contract 1000 units ...");
  await (await new TransferTransaction()
    .addTokenTransfer(tokenId, operatorId, -1000)
    .addTokenTransfer(tokenId, contractAcct, 1000)
    .execute(bankClient)).getReceipt(bankClient);
  const custodianBefore = await tokenUnits(bankClient, contractAcct, tokenId);

  console.log("- CONTRACT transfers 1000 units to the client (HTS precompile) ...");
  await (await new ContractExecuteTransaction().setContractId(contractId).setGas(1500000)
    .setFunction("transferShare", new ContractFunctionParameters().addAddress(clientEvm).addInt64(1000)).execute(bankClient)).getReceipt(bankClient);

  const clientUnits = await tokenUnits(bankClient, clientId, tokenId);
  const custodianAfter = await tokenUnits(bankClient, contractAcct, tokenId);

  const record = {
    network: "testnet",
    token: tokenId.toString(),
    custodianContract: contractId.toString(),
    client: clientId.toString(),
    custodianUnitsAfterFunding: custodianBefore,
    clientUnitsAfterContractTransfer: clientUnits,
    custodianUnitsAfterTransfer: custodianAfter,
    contractMovedRealAssetToken: clientUnits === "1000" && custodianAfter === "0",
    hashscan: "https://hashscan.io/testnet/contract/" + contractId,
  };
  fs.writeFileSync(path.join(__dirname, "..", "deployments", "contract_hts_binding.json"), JSON.stringify(record, null, 2));

  console.log("\n=== contract-mediated HTS asset transfer ===");
  console.log("  custodian held after funding: " + custodianBefore + " units");
  console.log("  client received from CONTRACT: " + clientUnits + " units");
  console.log("  custodian after transfer: " + custodianAfter + " units");
  console.log("  contract moved real asset token: " + record.contractMovedRealAssetToken);
  console.log("  hashscan: " + record.hashscan);
  bankClient.close(); clientClient.close();
}

main().catch((e) => { console.error("RUN FAILED:", e.message || e); process.exit(1); });
