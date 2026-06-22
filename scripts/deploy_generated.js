// Generic deploy runner for fiqhc-generated contracts. ONE runner, many instruments:
// it reads a deploy descriptor (emitted by `fiqhc build`) and deploys + exercises any
// generated contract on Hedera testnet. Reuses the proven patterns of interact8.js
// (operator-key selection, ContractCreateFlow, tinybar denomination). Testnet only.
//
//   node scripts/deploy_generated.js fiqh-compiler/out/<Contract>.deploy.json
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

function findArtifact(name) {
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
  throw new Error("artifact not found for " + name + " (run: npx hardhat compile)");
}

function bytecodeOf(name) {
  return findArtifact(name).bytecode.replace(/^0x/, "");
}

async function mkAcct(payer, key, hbar) {
  let tx = new AccountCreateTransaction().setInitialBalance(new Hbar(hbar));
  tx = typeof tx.setKeyWithoutAlias === "function" ? tx.setKeyWithoutAlias(key.publicKey) : tx.setKey(key.publicKey);
  return (await (await tx.execute(payer)).getReceipt(payer)).accountId;
}

function addArg(params, type, value, ids, fallbackId) {
  if (type === "address") {
    const key = String(value).replace(/^@/, "");
    const id = ids[key] || fallbackId;
    if (!id) throw new Error("unresolved address @" + key);
    if (!ids[key]) console.log("note: role @" + key + " mapped to operator (solo/lean run)");
    params.addAddress(id.toSolidityAddress());
  } else if (type === "uint256") {
    params.addUint256(Long.fromString(String(value)));
  } else {
    throw new Error("unsupported constructor type " + type);
  }
  return params;
}

async function main() {
  const descPath = process.argv[2];
  if (!descPath) throw new Error("usage: node scripts/deploy_generated.js <descriptor.json>");
  const d = JSON.parse(fs.readFileSync(descPath, "utf8"));

  const operatorId = AccountId.fromString((process.env.HEDERA_OPERATOR_ID || "").trim());
  const operatorKey = selectOperatorKey();
  const operator = Client.forTestnet().setOperator(operatorId, operatorKey);
  // contract-create is USD-priced (~$1); at the current depressed testnet hbar price that is
  // ~12 hbar, so the cap must exceed it or the node returns INSUFFICIENT_TX_FEE. Note the SDK
  // validates the cap via a 32-bit toInt(), which overflows above ~21.47 hbar (2^31 tinybar),
  // so 20 hbar is the practical ceiling — comfortably above the ~12 hbar contract-create fee.
  operator.setDefaultMaxTransactionFee(new Hbar(20));

  // signer + id registries; the operator plays the owning role (bank/rabb/lessor).
  const solo = process.argv.includes("--solo");
  const signers = { [d.operatorRole]: operator };
  const ids = { [d.operatorRole]: operatorId };
  if (solo) console.log("solo mode: creating no new accounts; all roles map to the operator");

  for (const name of (solo ? [] : (d.accounts || []))) {
    const key = PrivateKey.generateECDSA();
    // the client/lessee/mudarib send paying calls; others only receive.
    const bal = (name === "client" || name === "lessee" || name === "mudarib") ? 2 : 0.2;
    const id = await mkAcct(operator, key, bal);
    const c = Client.forTestnet().setOperator(id, key);
    c.setDefaultMaxTransactionFee(new Hbar(5));
    signers[name] = c;
    ids[name] = id;
    console.log("account " + name + " = " + id);
  }

  // 1) oracle
  const oracleId = (await (await new ContractCreateFlow()
    .setBytecode(bytecodeOf(d.oracle.contract))
    .setGas(1500000)
    .setConstructorParameters(new ContractFunctionParameters().addUint256(Long.fromString(String(d.oracle.initialValue))))
    .execute(operator)).getReceipt(operator)).contractId;
  ids.oracle = oracleId;
  console.log("oracle " + d.oracle.contract + " = " + oracleId + " (initial " + d.oracle.initialValue + ")");

  // 2) main contract
  let cparams = new ContractFunctionParameters();
  d.constructorAbi.forEach((t, i) => { cparams = addArg(cparams, t, d.constructorArgs[i], ids, operatorId); });
  const contractId = (await (await new ContractCreateFlow()
    .setBytecode(bytecodeOf(d.contract))
    .setGas(4000000)
    .setConstructorParameters(cparams)
    .execute(operator)).getReceipt(operator)).contractId;
  console.log("contract " + d.contract + " = " + contractId);

  const exec = async (clientForSigner, target, fn, args, valueTinybar) => {
    let tx = new ContractExecuteTransaction().setContractId(target).setGas(500000).setFunction(
      fn,
      (args || []).reduce((p, a) => p.addUint256(Long.fromString(String(a))), new ContractFunctionParameters())
    );
    if (valueTinybar != null) tx = tx.setPayableAmount(Hbar.fromTinybars(Long.fromString(String(valueTinybar))));
    const rec = await (await tx.execute(clientForSigner)).getReceipt(clientForSigner);
    return rec.status.toString();
  };

  // 3) funding
  const record = { instrument: d.instrument, contract: d.contract, contractId: contractId.toString(), oracleId: oracleId.toString(), accounts: {}, steps: [], reads: [] };
  for (const k of Object.keys(ids)) record.accounts[k] = ids[k].toString();

  for (const [fn, value] of Object.entries(d.funding || {})) {
    const role = fn.replace(/^fund/, "").toLowerCase() || d.operatorRole;
    const who = signers[role] || operator;
    const status = await exec(who, contractId, fn, [], value);
    console.log("funding " + fn + " (as " + role + ", " + value + " tinybar) -> " + status);
    record.steps.push({ fn, as: role, value, status });
  }

  // 4) lifecycle
  for (const step of (d.lifecycle || [])) {
    const who = signers[step.as] || operator;
    const target = step.target === "oracle" ? oracleId : contractId;
    const status = await exec(who, target, step.fn, step.args, step.value);
    console.log("step " + step.fn + " (as " + step.as + (step.target ? " on " + step.target : "") + ") -> " + status + (step.note ? "  // " + step.note : ""));
    record.steps.push({ fn: step.fn, as: step.as, target: step.target || "contract", args: step.args || [], value: step.value || null, status });
  }

  // 5) live state reads
  let allOk = true;
  for (const r of (d.reads || [])) {
    const q = await new ContractCallQuery().setContractId(contractId).setGas(80000).setFunction(r.fn).execute(operator);
    const got = q.getUint256(0).toString();
    const ok = String(r.expect) === got;
    allOk = allOk && ok;
    console.log("read " + r.fn + " = " + got + " (expect " + r.expect + ") " + (ok ? "OK" : "MISMATCH"));
    record.reads.push({ fn: r.fn, got, expect: r.expect, ok });
  }

  const bal = (await new AccountBalanceQuery().setAccountId(operatorId).execute(operator)).hbars.toString();
  record.operatorBalanceAfter = bal;
  console.log("operator balance after = " + bal);

  const outDir = path.join(__dirname, "..", "deployments");
  fs.mkdirSync(outDir, { recursive: true });
  const outFile = path.join(outDir, "generated_" + d.instrument + ".json");
  fs.writeFileSync(outFile, JSON.stringify(record, null, 2));
  console.log("wrote " + outFile);

  if (!allOk) { console.error("FAIL: a live state read did not match"); process.exit(1); }
  console.log("LIVE OK: " + d.contract + " deployed and exercised on Hedera testnet");
}

main().catch((e) => { console.error(e); process.exit(1); });
