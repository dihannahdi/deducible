// Live testnet deploy + on-chain proof of the Zakat al-Tijarah vector (enterprise vector #5).
// Deploys the fiqhc-generated MusharakahZakatGen, runs the core diminishing-partnership
// lifecycle, then PROVES the zakat routing on a public network: a payZakat() of a base above
// nisab moves EXACTLY 2.5% (rubʿ al-ʿushr, 1/40) to the maslahah fund, verified by the fund
// account's balance delta. Testnet only. Reuses deploy_generated.js patterns.
require("dotenv").config();
const fs = require("fs");
const path = require("path");
const Long = require("long");
const {
  Client, AccountId, PrivateKey, Hbar,
  AccountCreateTransaction, AccountBalanceQuery,
  ContractCreateFlow, ContractFunctionParameters, ContractExecuteTransaction,
  ContractCallQuery,
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
    let evm = "";
    try { evm = k.publicKey.toEvmAddress().toLowerCase(); } catch (e) {}
    if (want && evm === want) return k;
  }
  throw new Error("no private key matches HEDERA_EVM_ADDRESS");
}

function artifact(name) {
  const root = path.join(__dirname, "..", "artifacts", "contracts");
  const stack = [root];
  while (stack.length) {
    const dir = stack.pop();
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) stack.push(p);
      else if (e.name === name + ".json") return JSON.parse(fs.readFileSync(p, "utf8"));
    }
  }
  throw new Error("artifact not found: " + name);
}
const bytecodeOf = (n) => artifact(n).bytecode.replace(/^0x/, "");

async function mkAcct(payer, key, hbar) {
  let tx = new AccountCreateTransaction().setInitialBalance(new Hbar(hbar));
  tx = typeof tx.setKeyWithoutAlias === "function" ? tx.setKeyWithoutAlias(key.publicKey) : tx.setKey(key.publicKey);
  return (await (await tx.execute(payer)).getReceipt(payer)).accountId;
}

async function tinybarBalance(client, id) {
  const b = await new AccountBalanceQuery().setAccountId(id).execute(client);
  return BigInt(b.hbars.toTinybars().toString());
}

async function exec(signer, target, fn, args, valueTinybar) {
  let tx = new ContractExecuteTransaction().setContractId(target).setGas(500000).setFunction(
    fn, (args || []).reduce((p, a) => p.addUint256(Long.fromString(String(a))), new ContractFunctionParameters())
  );
  if (valueTinybar != null) tx = tx.setPayableAmount(Hbar.fromTinybars(Long.fromString(String(valueTinybar))));
  return (await (await tx.execute(signer)).getReceipt(signer)).status.toString();
}

async function readUint(client, cid, fn, arg) {
  const params = arg != null ? new ContractFunctionParameters().addUint256(Long.fromString(String(arg))) : undefined;
  const q = new ContractCallQuery().setContractId(cid).setGas(80000).setFunction(fn, params);
  return (await q.execute(client)).getUint256(0).toString();
}

async function main() {
  const ZAKAT_BASE = 10000000n;     // above nisab (8_500_000)
  const ZAKAT_DUE  = ZAKAT_BASE * 250n / 10000n;   // = 250_000 tinybar (exactly 2.5%)
  const BELOW_NISAB = 8000000n;     // below nisab -> due 0

  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const operator = Client.forTestnet().setOperator(operatorId, operatorKey);
  operator.setDefaultMaxTransactionFee(new Hbar(20));

  // role accounts (bank = operator)
  const clientKey = PrivateKey.generateECDSA();
  const clientId = await mkAcct(operator, clientKey, 2);
  const clientC = Client.forTestnet().setOperator(clientId, clientKey); clientC.setDefaultMaxTransactionFee(new Hbar(5));
  const arbiterId = await mkAcct(operator, PrivateKey.generateECDSA(), 0.2);
  const maslahahId = await mkAcct(operator, PrivateKey.generateECDSA(), 0.2);
  console.log("accounts: bank(operator)=" + operatorId + " client=" + clientId + " arbiter=" + arbiterId + " maslahah=" + maslahahId);

  // oracle + main contract
  const oracleId = (await (await new ContractCreateFlow().setBytecode(bytecodeOf("MockValuationOracle")).setGas(1500000)
    .setConstructorParameters(new ContractFunctionParameters().addUint256(Long.fromString("1000000"))).execute(operator)).getReceipt(operator)).contractId;
  console.log("oracle MockValuationOracle = " + oracleId);

  let cp = new ContractFunctionParameters()
    .addAddress(clientId.toSolidityAddress()).addAddress(oracleId.toSolidityAddress())
    .addAddress(arbiterId.toSolidityAddress()).addAddress(maslahahId.toSolidityAddress())
    .addUint256(Long.fromString("8000")).addUint256(Long.fromString("1")).addUint256(Long.fromString("3600"));
  const cid = (await (await new ContractCreateFlow().setBytecode(bytecodeOf("MusharakahZakatGen")).setGas(4000000)
    .setConstructorParameters(cp).execute(operator)).getReceipt(operator)).contractId;
  console.log("contract MusharakahZakatGen = " + cid);

  const rec = { instrument: "musharakah_mutanaqisah+zakat", contract: "MusharakahZakatGen",
    contractId: cid.toString(), oracleId: oracleId.toString(),
    accounts: { bank: operatorId.toString(), client: clientId.toString(), arbiter: arbiterId.toString(), maslahah: maslahahId.toString() },
    steps: [], reads: [], zakat: {} };

  // funding + core lifecycle
  rec.steps.push({ fn: "fundBank", value: 800000, status: await exec(operator, cid, "fundBank", [], 800000) });
  rec.steps.push({ fn: "fundClient", value: 200000, as: "client", status: await exec(clientC, cid, "fundClient", [], 200000) });
  rec.steps.push({ fn: "payRent", value: 8000, as: "client", status: await exec(clientC, cid, "payRent", [], 8000) });
  rec.steps.push({ fn: "buyShare", args: [2000], value: 200000, as: "client", status: await exec(clientC, cid, "buyShare", [2000], 200000) });
  for (const s of rec.steps) console.log("step " + s.fn + (s.as ? " (as " + s.as + ")" : "") + " -> " + s.status);

  for (const [fn, expect] of [["bankShareBps", "6000"], ["clientShareBps", "4000"]]) {
    const got = await readUint(operator, cid, fn);
    const ok = got === expect; rec.reads.push({ fn, got, expect, ok });
    console.log("read " + fn + " = " + got + " (expect " + expect + ") " + (ok ? "OK" : "MISMATCH"));
  }

  // ===== ZAKAT ROUTING PROOF (on-chain) =====
  const before = await tinybarBalance(operator, maslahahId);
  const zStatus = await exec(operator, cid, "payZakat", [ZAKAT_BASE.toString()], ZAKAT_DUE.toString());
  const after = await tinybarBalance(operator, maslahahId);
  const delta = after - before;
  const dueRead = await readUint(operator, cid, "zakatDue", ZAKAT_BASE.toString());
  const belowRead = await readUint(operator, cid, "zakatDue", BELOW_NISAB.toString());
  rec.zakat = {
    base: ZAKAT_BASE.toString(), expectedDue: ZAKAT_DUE.toString(), payZakatStatus: zStatus,
    maslahahBefore: before.toString(), maslahahAfter: after.toString(), maslahahDelta: delta.toString(),
    deltaEqualsDue: delta === ZAKAT_DUE, zakatDueRead: dueRead, zakatDueReadOk: dueRead === ZAKAT_DUE.toString(),
    belowNisabRead: belowRead, belowNisabOk: belowRead === "0",
    ratePctOfBase: Number(delta) / Number(ZAKAT_BASE) * 100,
  };
  console.log("ZAKAT: payZakat(" + ZAKAT_BASE + ") -> " + zStatus);
  console.log("ZAKAT: maslahah balance " + before + " -> " + after + " (delta " + delta + " tinybar = " + rec.zakat.ratePctOfBase + "% of base) " + (delta === ZAKAT_DUE ? "OK" : "MISMATCH"));
  console.log("ZAKAT: zakatDue(" + ZAKAT_BASE + ") = " + dueRead + " (expect " + ZAKAT_DUE + ") " + (rec.zakat.zakatDueReadOk ? "OK" : "MISMATCH"));
  console.log("ZAKAT: zakatDue(" + BELOW_NISAB + " below nisab) = " + belowRead + " (expect 0) " + (rec.zakat.belowNisabOk ? "OK" : "MISMATCH"));

  const balAfter = (await new AccountBalanceQuery().setAccountId(operatorId).execute(operator)).hbars.toString();
  rec.operatorBalanceAfter = balAfter;
  console.log("operator balance after = " + balAfter);

  const outDir = path.join(__dirname, "..", "deployments");
  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, "testnet_zakat.json"), JSON.stringify(rec, null, 2));
  console.log("wrote deployments/testnet_zakat.json");

  const allOk = rec.reads.every(r => r.ok) && rec.zakat.deltaEqualsDue && rec.zakat.zakatDueReadOk && rec.zakat.belowNisabOk;
  if (!allOk) { console.error("FAIL: a check did not match"); process.exit(1); }
  console.log("LIVE OK: MusharakahZakatGen deployed and zakat routing proven on Hedera testnet");
}
main().catch((e) => { console.error(e); process.exit(1); });
