// Iteration 4: tokenize the co-owned asset as a real Hedera Token Service (HTS) token.
// Ownership becomes transferable token units (8000 bank / 2000 client of 10000 = 80/20),
// addressing the "ownership is a bare counter" gap at the representation level.
// Legal title remains an off-chain bridge (acknowledged in the paper).
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const {
  Client, AccountId, PrivateKey, Hbar,
  AccountCreateTransaction, AccountBalanceQuery,
  TokenCreateTransaction, TokenType, TokenSupplyType,
  TokenAssociateTransaction, TransferTransaction,
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

async function main() {
  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const bankClient = Client.forTestnet().setOperator(operatorId, operatorKey);
  bankClient.setDefaultMaxTransactionFee(new Hbar(20));

  console.log("- creating client account ...");
  const clientKey = PrivateKey.generateECDSA();
  const cc = await new AccountCreateTransaction()
    .setInitialBalance(new Hbar(30));
  const ccTx = (typeof cc.setKeyWithoutAlias === "function" ? cc.setKeyWithoutAlias(clientKey.publicKey) : cc.setKey(clientKey.publicKey));
  const clientId = (await (await ccTx.execute(bankClient)).getReceipt(bankClient)).accountId;
  const clientClient = Client.forTestnet().setOperator(clientId, clientKey);
  clientClient.setDefaultMaxTransactionFee(new Hbar(20));
  console.log("  bank(treasury)=" + operatorId + "  client=" + clientId);

  console.log("- creating HTS asset-share token (finite, 10000 units = 100%) ...");
  const tokenTx = await new TokenCreateTransaction()
    .setTokenName("Musharakah Asset Share")
    .setTokenSymbol("MMS")
    .setDecimals(0)
    .setInitialSupply(10000)
    .setTreasuryAccountId(operatorId)
    .setTokenType(TokenType.FungibleCommon)
    .setSupplyType(TokenSupplyType.Finite)
    .setMaxSupply(10000)
    .execute(bankClient);
  const tokenId = (await tokenTx.getReceipt(bankClient)).tokenId;
  console.log("  tokenId=" + tokenId);

  console.log("- client associates the token; bank transfers the client's 20% share ...");
  await (await new TokenAssociateTransaction().setAccountId(clientId).setTokenIds([tokenId]).execute(clientClient)).getReceipt(clientClient);
  await (await new TransferTransaction()
    .addTokenTransfer(tokenId, operatorId, -2000)
    .addTokenTransfer(tokenId, clientId, 2000)
    .execute(bankClient)).getReceipt(bankClient);

  const bankBal = await new AccountBalanceQuery().setAccountId(operatorId).execute(bankClient);
  const clientBal = await new AccountBalanceQuery().setAccountId(clientId).execute(bankClient);
  const bankUnits = bankBal.tokens.get(tokenId);
  const clientUnits = clientBal.tokens.get(tokenId);
  const b = bankUnits ? bankUnits.toString() : "0";
  const c = clientUnits ? clientUnits.toString() : "0";

  const record = {
    network: "testnet",
    token: { id: tokenId.toString(), name: "Musharakah Asset Share", symbol: "MMS", decimals: 0, supply: 10000 },
    treasury_bank: operatorId.toString(),
    client: clientId.toString(),
    holdings: { bankUnits: b, clientUnits: c, bankPct: Number(b) / 100, clientPct: Number(c) / 100 },
    hashscan: "https://hashscan.io/testnet/token/" + tokenId,
    note: "Real transferable HTS token = fractional asset ownership. Legal title remains an off-chain registry bridge (paper limitation).",
  };
  fs.writeFileSync(path.join(__dirname, "..", "deployments", "tokenized_asset.json"), JSON.stringify(record, null, 2));

  console.log("\n=== tokenized ownership (HTS) ===");
  console.log("  token " + tokenId + " (MMS, 10000 units)");
  console.log("  bank holds " + b + " units (" + record.holdings.bankPct + "%)");
  console.log("  client holds " + c + " units (" + record.holdings.clientPct + "%)");
  console.log("  hashscan: " + record.hashscan);
  console.log("  tokenized split correct: " + (b === "8000" && c === "2000"));
  bankClient.close(); clientClient.close();
}

main().catch((e) => { console.error("RUN FAILED:", e.message || e); process.exit(1); });
