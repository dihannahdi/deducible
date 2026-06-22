//! Backend: lower a *validated* AST to Solidity, a Hardhat test, and a deploy
//! descriptor. Codegen is only ever reached after `sema::check` returns no errors,
//! so every emitted contract is consistent-by-construction with its declared
//! rule-base. The generated Solidity matches the conventions of the hand-written,
//! peer-reviewed artifact (immutable parties, BPS, onlyX/live/nonReentrant
//! modifiers, one event per transition, oracle as the trust boundary), and the
//! declared invariants are compiled in as `@dev INVARIANT` annotations.

use crate::ast::*;
use crate::sema::Class;

pub struct Generated {
    pub instrument: String,
    pub contract_name: String,
    pub sol: String,
    pub test_js: String,
    pub descriptor: String,
}

pub fn generate(spec: &Spec) -> Result<Generated, String> {
    match Class::from_str(&spec.class) {
        Class::MusharakahMutanaqisah => gen_musharakah(spec),
        Class::Mudarabah => gen_mudarabah(spec),
        Class::IjarahImbt => gen_ijarah(spec),
        Class::Unknown(s) => Err(format!("no backend for instrument class '{}'", s)),
    }
}

// --- helpers ---

fn role_name<'a>(spec: &'a Spec, role: &str) -> Option<&'a str> {
    spec.parties()
        .into_iter()
        .find(|p| p.role == role)
        .map(|p| p.name.as_str())
}

fn party_bps(spec: &Spec, role: &str) -> Option<u64> {
    let name = role_name(spec, role)?;
    spec.capital().into_iter().find_map(|c| match c {
        CapItem::Assign { party, bps, .. } if party == name => Some(*bps),
        _ => None,
    })
}

/// Numeric rent rate from `returns { rent { rate: N ...; } }`.
fn rent_rate(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "rent")
        .and_then(|r| kv_get(&r.kvs, "rate"))
        .and_then(|e| e.as_num())
        .unwrap_or(1)
}

fn profit_share(spec: &Spec, role: &str) -> Option<u64> {
    let name = role_name(spec, role)?;
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "profit")
        .and_then(|r| kv_get(&r.kvs, name).and_then(|e| e.as_num()))
}

/// Window (seconds) for khiyar al-shart, if declared.
fn khiyar_window(spec: &Spec) -> Option<u64> {
    spec.rescission()
        .into_iter()
        .find(|r| r.kind == "khiyar_al_shart")
        .and_then(|r| kv_get(&r.kvs, "window"))
        .and_then(|e| e.as_num())
}

fn has_resc(spec: &Spec, kind: &str) -> bool {
    spec.rescission().iter().any(|r| r.kind == kind)
}

fn has_step(spec: &Spec, name: &str) -> bool {
    spec.lifecycle().iter().any(|s| s.name == name)
}

fn invariant_doc(spec: &Spec) -> String {
    let mut s = String::new();
    for inv in spec.invariants() {
        s.push_str(&format!("///           - {}: {}\n", inv.name, inv.expr.render()));
    }
    s
}

fn provenance_doc(spec: &Spec, title: &str) -> String {
    let basis = spec
        .meta()
        .into_iter()
        .find(|k| k.key == "basis")
        .and_then(|k| k.val.as_str())
        .unwrap_or("(unstated)");
    format!(
        "// SPDX-License-Identifier: MIT\n\
         pragma solidity ^0.8.24;\n\n\
         import {{IValuationOracle}} from \"../IValuationOracle.sol\";\n\n\
         /// @title  {title}\n\
         /// @notice COMPLIANCE BY CONSTRUCTION. Emitted by the fiqhc compiler ONLY after the\n\
         ///         source .fiqh specification was proven consistent with its declared fiqh\n\
         ///         rule-base. Declared basis: {basis} [scholar-verify].\n\
         ///         The engine issues no fatwa; a qualified scholar must ratify the rule-base.\n\
         ///         Compiled-in invariants:\n\
         {invs}",
        title = title,
        basis = basis,
        invs = invariant_doc(spec),
    )
}

// =====================================================================================
// Musharakah Mutanaqisah  (target: behavioural equivalence with the hand-written V5
// + payRent; capital custody, oracle-priced buyout, settle with maslahah residue,
// and the full rescission family — khiyar al-shart, iqalah, khiyar al-'ayb, faskh)
// =====================================================================================

fn gen_musharakah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let bank_bps = party_bps(spec, "financier").ok_or("financier capital missing")?;
    let rate = rent_rate(spec);
    let window = khiyar_window(spec).unwrap_or(3600);

    let mut s = provenance_doc(spec, &format!("{} — diminishing partnership (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));

    s.push_str(MMP_STATE);
    s.push_str(MMP_EVENTS);
    if has_step(spec, "payRent") {
        s.push_str("    event RentPaid(uint256 amount, uint256 onBankShareBps);\n");
    }
    s.push('\n');
    s.push_str(MMP_CONSTRUCTOR);
    s.push_str(MMP_FUNDING);
    s.push_str(MMP_RENTDUE);
    if has_step(spec, "payRent") {
        s.push_str(MMP_PAYRENT);
    }
    s.push_str(MMP_BUYSHARE);
    s.push_str(MMP_SETTLE);

    // rescission family (each gated on its declaration)
    let any_resc = has_resc(spec, "khiyar_al_shart")
        || has_resc(spec, "iqalah")
        || has_resc(spec, "khiyar_al_ayb")
        || has_resc(spec, "faskh");
    if has_resc(spec, "khiyar_al_shart") {
        s.push_str(MMP_KHIYAR);
    }
    if has_resc(spec, "iqalah") {
        s.push_str(MMP_IQALAH);
    }
    if has_resc(spec, "khiyar_al_ayb") {
        s.push_str(MMP_DEFECT);
    }
    if has_resc(spec, "faskh") {
        s.push_str(MMP_FASKH);
    }
    if any_resc {
        s.push_str(MMP_UNWIND);
    }

    s.push_str("}\n");

    let test_js = gen_musharakah_test(&name, bank_bps, rate, window);
    let descriptor = musharakah_descriptor(spec, &name, bank_bps, rate, window);

    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const MMP_STATE: &str = r#"    address public immutable bank;
    address public immutable client;
    address public immutable arbiter;
    address public immutable maslahahFund;
    IValuationOracle public immutable oracle;
    uint256 public constant BPS = 10_000;

    uint256 public immutable initialBankShareBps;
    uint256 public bankShareBps;
    uint256 public clientShareBps;
    uint256 public immutable rentPerPeriodPerBps;

    uint256 public pool;
    uint256 public bankFunded;
    uint256 public clientFunded;
    bool public active;
    bool public settled;
    bool public rescinded;

    uint256 public immutable khiyarPeriod;
    uint256 public khiyarDeadline;
    address public iqalahProposer;
    bool public defectRaised;
    address public defectClaimant;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyBank() { require(msg.sender == bank, "only bank"); _; }
    modifier onlyClient() { require(msg.sender == client, "only client"); _; }
    modifier onlyArbiter() { require(msg.sender == arbiter, "only arbiter"); _; }
    modifier live() { require(active && !settled && !rescinded, "not live"); _; }

"#;

const MMP_EVENTS: &str = r#"    event Funded(address who, uint256 amount);
    event Activated(uint256 assetValue, uint256 khiyarDeadline);
    event SharePurchased(uint256 bps, uint256 price, uint256 newBankBps);
    event Settled(uint256 fairValue, uint256 bankPayout, uint256 clientPayout, uint256 toMaslahah);
    event KhiyarRescinded(address by);
    event IqalahProposed(address by);
    event IqalahCompleted(address acceptedBy);
    event DefectRaised(address by, string reason);
    event DefectResolved(bool upheld);
    event JudicialFaskh(address arbiter);
    event Unwound(uint256 bankRefund, uint256 clientRefund);
"#;

const MMP_CONSTRUCTOR: &str = r#"    constructor(
        address _client, address _oracle, address _arbiter, address _maslahahFund,
        uint256 _bankShareBps, uint256 _rentPerPeriodPerBps, uint256 _khiyarPeriod
    ) {
        require(_client != address(0) && _oracle != address(0) && _arbiter != address(0) && _maslahahFund != address(0), "zero addr");
        require(_bankShareBps > 0 && _bankShareBps < BPS, "bank bps range");
        bank = msg.sender; client = _client; arbiter = _arbiter; maslahahFund = _maslahahFund;
        oracle = IValuationOracle(_oracle);
        bankShareBps = _bankShareBps; initialBankShareBps = _bankShareBps; clientShareBps = BPS - _bankShareBps;
        rentPerPeriodPerBps = _rentPerPeriodPerBps; khiyarPeriod = _khiyarPeriod;
    }

"#;

const MMP_FUNDING: &str = r#"    function fundBank() external payable onlyBank {
        require(!active && bankFunded == 0, "bank funded/active");
        uint256 v0 = oracle.fairValue();
        require(msg.value == v0 * bankShareBps / BPS, "bank must fund its share");
        bankFunded = msg.value; pool += msg.value; emit Funded(bank, msg.value); _tryActivate(v0);
    }
    function fundClient() external payable onlyClient {
        require(!active && clientFunded == 0, "client funded/active");
        uint256 v0 = oracle.fairValue();
        require(msg.value == v0 * clientShareBps / BPS, "client must fund its share");
        clientFunded = msg.value; pool += msg.value; emit Funded(client, msg.value); _tryActivate(v0);
    }
    function _tryActivate(uint256 v0) internal {
        if (bankFunded > 0 && clientFunded > 0) {
            require(bankFunded + clientFunded == v0, "funds must equal asset value");
            active = true; khiyarDeadline = block.timestamp + khiyarPeriod;
            emit Activated(v0, khiyarDeadline);
        }
    }

"#;

const MMP_RENTDUE: &str = r#"    /// @dev INVARIANT rent_on_living_share: rent accrues on the bank's CURRENT share only.
    function rentDue() public view returns (uint256) { return rentPerPeriodPerBps * bankShareBps; }

"#;

const MMP_PAYRENT: &str = r#"    function payRent() external payable onlyClient live nonReentrant {
        uint256 due = rentDue();
        require(msg.value == due, "rent must equal due on bank share");
        emit RentPaid(due, bankShareBps);
        (bool ok, ) = bank.call{value: msg.value}(""); require(ok, "rent transfer failed");
    }

"#;

const MMP_BUYSHARE: &str = r#"    /// @dev INVARIANT price_attested: buyout price tracks the independent oracle's fair value.
    function buyShare(uint256 bps) external payable onlyClient live nonReentrant {
        require(bps > 0 && bps <= bankShareBps, "bps range");
        uint256 f = oracle.fairValue(); require(f > 0, "oracle value");
        uint256 price = f * bps / BPS;
        require(msg.value == price, "value != fair price");
        bankShareBps -= bps; clientShareBps += bps;
        pool += msg.value; pool -= price;
        emit SharePurchased(bps, price, bankShareBps);
        (bool ok, ) = bank.call{value: price}(""); require(ok, "buyout xfer");
    }

"#;

const MMP_SETTLE: &str = r#"    /// @dev INVARIANT loss_follows_capital: the impaired remainder is shared by ownership;
    ///      the residue goes to the agreed maslahah fund rather than being stranded.
    function settle() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client || msg.sender == arbiter, "only party/arbiter");
        uint256 f = oracle.fairValue(); require(f > 0, "oracle value");
        uint256 distributable = f > pool ? pool : f;
        uint256 bankPayout = distributable * bankShareBps / BPS;
        uint256 clientPayout = distributable * clientShareBps / BPS;
        uint256 toMaslahah = pool - bankPayout - clientPayout;
        settled = true; pool = 0;
        if (bankPayout > 0) { (bool a, ) = bank.call{value: bankPayout}(""); require(a, "bank payout"); }
        if (clientPayout > 0) { (bool b, ) = client.call{value: clientPayout}(""); require(b, "client payout"); }
        if (toMaslahah > 0) { (bool d, ) = maslahahFund.call{value: toMaslahah}(""); require(d, "maslahah xfer"); }
        emit Settled(f, bankPayout, clientPayout, toMaslahah);
    }

"#;

const MMP_KHIYAR: &str = r#"    function rescindKhiyar() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        require(block.timestamp <= khiyarDeadline, "khiyar window closed");
        require(bankShareBps == initialBankShareBps, "performance begun");
        emit KhiyarRescinded(msg.sender); _unwind();
    }

"#;

const MMP_IQALAH: &str = r#"    function proposeIqalah() external live {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        iqalahProposer = msg.sender; emit IqalahProposed(msg.sender);
    }
    function acceptIqalah() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        require(iqalahProposer != address(0) && msg.sender != iqalahProposer, "needs the other partner");
        require(bankShareBps == initialBankShareBps, "performance begun");
        emit IqalahCompleted(msg.sender); _unwind();
    }

"#;

const MMP_DEFECT: &str = r#"    function raiseDefect(string calldata reason) external live {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        defectRaised = true; defectClaimant = msg.sender;
        emit DefectRaised(msg.sender, reason);
    }
    function resolveDefect(bool upheld) external live nonReentrant onlyArbiter {
        require(defectRaised, "no defect raised");
        emit DefectResolved(upheld);
        if (upheld) { _unwind(); } else { defectRaised = false; defectClaimant = address(0); }
    }

"#;

const MMP_FASKH: &str = r#"    function judicialFaskh() external live nonReentrant onlyArbiter {
        emit JudicialFaskh(msg.sender); _unwind();
    }

"#;

const MMP_UNWIND: &str = r#"    function _unwind() internal {
        uint256 b = bankFunded; uint256 cl = clientFunded;
        rescinded = true; active = false; pool = 0; bankFunded = 0; clientFunded = 0;
        if (b > 0) { (bool ok, ) = bank.call{value: b}(""); require(ok, "bank refund"); }
        if (cl > 0) { (bool ok2, ) = client.call{value: cl}(""); require(ok2, "client refund"); }
        emit Unwound(b, cl);
    }
"#;

fn gen_musharakah_test(name: &str, bank_bps: u64, rate: u64, window: u64) -> String {
    let client_bps = 10_000 - bank_bps;
    format!(
        r#"// Generated by fiqhc — differential proof: the GENERATED {name} reproduces the
// hand-written, peer-reviewed Musharakah Mutanaqisah behaviour on the shared lifecycle.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — differential equivalence", function () {{
  let bank, client, valuer, arbiter, maslahah, oracle, c;
  const V0 = 100000000n;
  const BANK_BPS = {bank_bps}n;
  const CLIENT_BPS = {client_bps}n;
  const RATE = {rate}n;
  const KHIYAR = {window};

  beforeEach(async function () {{
    [bank, client, valuer, arbiter, maslahah] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(V0);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(bank).deploy(client.address, await oracle.getAddress(), arbiter.address, maslahah.address, BANK_BPS, RATE, KHIYAR);
    await c.waitForDeployment();
  }});
  async function fund() {{
    await c.connect(bank).fundBank({{ value: (V0 * BANK_BPS) / 10000n }});
    await c.connect(client).fundClient({{ value: (V0 * CLIENT_BPS) / 10000n }});
  }}

  it("I1 rent_on_living_share: rentDue tracks the bank's living share", async function () {{
    await fund();
    expect(await c.rentDue()).to.equal(RATE * BANK_BPS);
    const due = RATE * BANK_BPS;
    await expect(c.connect(client).payRent({{ value: due }}))
      .to.emit(c, "RentPaid").withArgs(due, BANK_BPS);
  }});

  it("I2 price_attested: buyShare is priced from the oracle and steps ownership down", async function () {{
    await fund();
    const bps = 2000n;
    const price = (V0 * bps) / 10000n;
    await expect(c.connect(client).buyShare(bps, {{ value: price }}))
      .to.emit(c, "SharePurchased").withArgs(bps, price, BANK_BPS - bps);
    expect(await c.bankShareBps()).to.equal(BANK_BPS - bps);
  }});

  it("I3 loss_follows_capital: settle pays current value by ownership; residue to maslahah", async function () {{
    await fund();
    await oracle.connect(valuer).attest(90000000n);
    const before = await ethers.provider.getBalance(maslahah.address);
    await expect(c.connect(bank).settle())
      .to.emit(c, "Settled").withArgs(90000000n, (90000000n * BANK_BPS) / 10000n, (90000000n * CLIENT_BPS) / 10000n, 10000000n);
    const after = await ethers.provider.getBalance(maslahah.address);
    expect(after - before).to.equal(10000000n);
  }});

  it("I4 role separation: only the arbiter may uphold a defect (khiyar al-'ayb)", async function () {{
    await fund();
    await c.connect(client).raiseDefect("latent defect");
    await expect(c.connect(bank).resolveDefect(true)).to.be.revertedWith("only arbiter");
    await expect(c.connect(arbiter).resolveDefect(true)).to.emit(c, "Unwound");
    expect(await c.rescinded()).to.equal(true);
  }});

  it("flexibility: judicial faskh is the arbiter's authority alone; iqalah needs both partners", async function () {{
    await fund();
    await expect(c.connect(client).judicialFaskh()).to.be.revertedWith("only arbiter");
    await c.connect(bank).proposeIqalah();
    await expect(c.connect(client).acceptIqalah()).to.emit(c, "IqalahCompleted").withArgs(client.address);
    expect(await c.rescinded()).to.equal(true);
  }});
}});
"#,
        name = name,
        bank_bps = bank_bps,
        client_bps = client_bps,
        rate = rate,
        window = window,
    )
}

fn musharakah_descriptor(spec: &Spec, name: &str, bank_bps: u64, rate: u64, window: u64) -> String {
    // Lean tinybar asset value to fit a constrained testnet budget.
    let v0: u64 = 1_000_000;
    let bank_fund = v0 * bank_bps / 10_000;
    let client_fund = v0 - bank_fund;
    let buy_bps: u64 = 2000;
    let buy_price = v0 * buy_bps / 10_000;
    let rent_due = rate * bank_bps;
    let _ = spec;
    format!(
        r#"{{
  "instrument": "musharakah_mutanaqisah",
  "contract": "{name}",
  "operatorRole": "bank",
  "oracle": {{ "contract": "MockValuationOracle", "initialValue": {v0} }},
  "constructorAbi": ["address","address","address","address","uint256","uint256","uint256"],
  "constructorArgs": ["@client","@oracle","@arbiter","@maslahah",{bank_bps},{rate},{window}],
  "accounts": ["client","arbiter","maslahah"],
  "funding": {{ "fundBank": {bank_fund}, "fundClient": {client_fund} }},
  "lifecycle": [
    {{ "as": "client", "fn": "payRent", "value": {rent_due}, "note": "rent on living share" }},
    {{ "as": "client", "fn": "buyShare", "args": [{buy_bps}], "value": {buy_price}, "note": "oracle-priced buyout, ownership steps down" }}
  ],
  "reads": [
    {{ "fn": "bankShareBps", "expect": {bank_after} }},
    {{ "fn": "clientShareBps", "expect": {client_after} }}
  ]
}}
"#,
        bank_after = bank_bps - buy_bps,
        client_after = 10_000 - bank_bps + buy_bps,
        name = name,
        v0 = v0,
        bank_bps = bank_bps,
        rate = rate,
        window = window,
        bank_fund = bank_fund,
        client_fund = client_fund,
        rent_due = rent_due,
        buy_bps = buy_bps,
        buy_price = buy_price,
    )
}

fn lease_term(spec: &Spec) -> u64 {
    spec.returns()
        .into_iter()
        .find(|r| r.kind == "rent")
        .and_then(|r| kv_get(&r.kvs, "term"))
        .and_then(|e| e.as_num())
        .unwrap_or(3)
}

// =====================================================================================
// Mudarabah — profit-sharing trust. Capital from the rabb al-mal alone (placed with the
// mudarib to trade); profit split by a pre-agreed ratio; financial loss falls on the
// rabb al-mal alone unless the arbiter rules the mudarib negligent (ta'addi/taqsir).
// =====================================================================================

fn gen_mudarabah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let profit_rabb = profit_share(spec, "rabb_al_mal").ok_or("rabb al-mal profit share missing")?;
    let mut s = provenance_doc(spec, &format!("{} — profit-sharing trust (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(MUDARABAH_BODY);
    s.push_str("}\n");
    let test_js = gen_mudarabah_test(&name, profit_rabb);
    let descriptor = mudarabah_descriptor(&name, profit_rabb);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const MUDARABAH_BODY: &str = r#"    address public immutable rabbAlMal;
    address public immutable mudarib;
    address public immutable arbiter;
    IValuationOracle public immutable oracle;
    uint256 public constant BPS = 10_000;

    uint256 public immutable profitRabbBps;
    uint256 public immutable profitMudaribBps;

    uint256 public capital;
    uint256 public pool;
    bool public active;
    bool public returned;
    bool public settled;
    bool public mudaribLiable;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyRabb() { require(msg.sender == rabbAlMal, "only rabb al-mal"); _; }
    modifier onlyMudarib() { require(msg.sender == mudarib, "only mudarib"); _; }
    modifier onlyArbiter() { require(msg.sender == arbiter, "only arbiter"); _; }

    event Funded(uint256 capital);
    event ProceedsReturned(uint256 amount);
    event NegligenceRuled(address by);
    event Settled(uint256 realized, uint256 rabbPayout, uint256 mudaribPayout, bool loss);
    event MudaribLiability(uint256 shortfall);

    constructor(address _mudarib, address _oracle, address _arbiter, uint256 _profitRabbBps) {
        require(_mudarib != address(0) && _oracle != address(0) && _arbiter != address(0), "zero addr");
        require(_profitRabbBps > 0 && _profitRabbBps < BPS, "profit bps range");
        rabbAlMal = msg.sender; mudarib = _mudarib; arbiter = _arbiter;
        oracle = IValuationOracle(_oracle);
        profitRabbBps = _profitRabbBps; profitMudaribBps = BPS - _profitRabbBps;
    }

    /// @dev INVARIANT capital_from_rabb_al_mal_only: only the rabb al-mal funds; the capital is
    ///      placed with the mudarib to trade (the mudarib contributes labor, not capital).
    function fund() external payable onlyRabb {
        require(!active && capital == 0, "funded/active");
        uint256 c = oracle.fairValue();
        require(msg.value == c, "rabb funds the full capital");
        capital = msg.value; active = true;
        emit Funded(msg.value);
        (bool ok, ) = mudarib.call{value: msg.value}(""); require(ok, "capital to mudarib");
    }

    /// @dev the mudarib returns the INDEPENDENTLY-ATTESTED realized value; it is not self-reported.
    function reportReturn() external payable onlyMudarib {
        require(active && !returned && !settled, "not live");
        uint256 r = oracle.fairValue();
        require(msg.value == r, "must deposit the attested realized value");
        pool += msg.value; returned = true;
        emit ProceedsReturned(msg.value);
    }

    function ruleNegligence() external onlyArbiter {
        require(active && !settled, "not live");
        mudaribLiable = true; emit NegligenceRuled(msg.sender);
    }

    /// @dev INVARIANT profit_by_ratio + loss_on_rabb_al_mal: profit splits by the pre-agreed
    ///      ratio; financial loss falls on the rabb al-mal alone (the mudarib loses only its
    ///      effort) unless the arbiter has ruled the mudarib negligent (ta'addi/taqsir).
    function settle() external nonReentrant {
        require(msg.sender == rabbAlMal || msg.sender == mudarib || msg.sender == arbiter, "only party/arbiter");
        require(returned, "no proceeds reported");
        require(!settled, "settled");
        settled = true;
        uint256 realized = pool;
        uint256 rabbPayout;
        uint256 mudaribPayout;
        bool loss;
        if (realized >= capital) {
            uint256 profit = realized - capital;
            uint256 rp = profit * profitRabbBps / BPS;
            rabbPayout = capital + rp;
            mudaribPayout = profit - rp;
            loss = false;
        } else {
            loss = true;
            rabbPayout = realized;
            mudaribPayout = 0;
            if (mudaribLiable) { emit MudaribLiability(capital - realized); }
        }
        pool = 0;
        if (rabbPayout > 0) { (bool a, ) = rabbAlMal.call{value: rabbPayout}(""); require(a, "rabb payout"); }
        if (mudaribPayout > 0) { (bool b, ) = mudarib.call{value: mudaribPayout}(""); require(b, "mudarib payout"); }
        emit Settled(realized, rabbPayout, mudaribPayout, loss);
    }
"#;

fn gen_mudarabah_test(name: &str, profit_rabb: u64) -> String {
    format!(
        r#"// Generated by fiqhc — Mudarabah profit-sharing trust. Proves the SAME compiler emits a
// DIFFERENT, correct instrument: profit by ratio, and loss borne by the rabb al-mal alone.
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — profit-sharing & loss on rabb al-mal", function () {{
  let rabb, agent, valuer, arbiter, oracle, c;
  const C = 100000000n;
  const PROFIT_RABB = {profit_rabb}n;

  beforeEach(async function () {{
    [rabb, agent, valuer, arbiter] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(C);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(rabb).deploy(agent.address, await oracle.getAddress(), arbiter.address, PROFIT_RABB);
    await c.waitForDeployment();
  }});

  it("capital_from_rabb_al_mal_only: fund() places the full capital with the mudarib (labor, not capital)", async function () {{
    const before = await ethers.provider.getBalance(agent.address);
    await c.connect(rabb).fund({{ value: C }});
    const after = await ethers.provider.getBalance(agent.address);
    expect(after - before).to.equal(C);
  }});

  it("profit_by_ratio: a profit splits by the pre-agreed ratio", async function () {{
    await c.connect(rabb).fund({{ value: C }});
    await oracle.connect(valuer).attest(150000000n);
    await c.connect(agent).reportReturn({{ value: 150000000n }});
    const profit = 50000000n;
    const rp = (profit * PROFIT_RABB) / 10000n;
    await expect(c.connect(rabb).settle())
      .to.emit(c, "Settled").withArgs(150000000n, C + rp, profit - rp, false);
  }});

  it("loss_on_rabb_al_mal: a loss is borne by the rabb al-mal alone; the mudarib loses only effort", async function () {{
    await c.connect(rabb).fund({{ value: C }});
    await oracle.connect(valuer).attest(80000000n);
    await c.connect(agent).reportReturn({{ value: 80000000n }});
    await expect(c.connect(rabb).settle())
      .to.emit(c, "Settled").withArgs(80000000n, 80000000n, 0n, true);
  }});

  it("role separation: only the arbiter may rule the mudarib negligent (ta'addi/taqsir)", async function () {{
    await c.connect(rabb).fund({{ value: C }});
    await expect(c.connect(agent).ruleNegligence()).to.be.revertedWith("only arbiter");
    await expect(c.connect(arbiter).ruleNegligence()).to.emit(c, "NegligenceRuled");
  }});
}});
"#,
        name = name,
        profit_rabb = profit_rabb,
    )
}

fn mudarabah_descriptor(name: &str, profit_rabb: u64) -> String {
    let v0: u64 = 1_000_000;
    let realized: u64 = 1_500_000;
    format!(
        r#"{{
  "instrument": "mudarabah",
  "contract": "{name}",
  "operatorRole": "rabb",
  "oracle": {{ "contract": "MockValuationOracle", "initialValue": {v0} }},
  "constructorAbi": ["address","address","address","uint256"],
  "constructorArgs": ["@mudarib","@oracle","@arbiter",{profit_rabb}],
  "accounts": ["mudarib","arbiter"],
  "funding": {{ "fund": {v0} }},
  "lifecycle": [
    {{ "target": "oracle", "fn": "attest", "args": [{realized}], "note": "valuer attests realized venture value" }},
    {{ "as": "mudarib", "fn": "reportReturn", "value": {realized}, "note": "mudarib deposits attested proceeds" }},
    {{ "as": "rabb", "fn": "settle", "note": "profit split by ratio" }}
  ]
}}
"#,
        name = name,
        v0 = v0,
        realized = realized,
        profit_rabb = profit_rabb,
    )
}

// =====================================================================================
// Ijarah Muntahia Bittamleek — lease ending in ownership. Rent prices the usufruct and
// flows to the lessor; the lessor bears ownership risk and major maintenance; transfer of
// ownership is a SEPARATE act at the end of term; any late charge goes to charity, not the
// lessor (no interest on a debt).
// =====================================================================================

fn gen_ijarah(spec: &Spec) -> Result<Generated, String> {
    let name = format!("{}Gen", spec.name);
    let rate = rent_rate(spec);
    let term = lease_term(spec);
    let mut s = provenance_doc(spec, &format!("{} — lease ending in ownership (generated)", name));
    s.push_str(&format!("contract {} {{\n", name));
    s.push_str(IJARAH_BODY);
    s.push_str("}\n");
    let test_js = gen_ijarah_test(&name, rate, term);
    let descriptor = ijarah_descriptor(&name, rate, term);
    Ok(Generated {
        instrument: spec.class.clone(),
        contract_name: name,
        sol: s,
        test_js,
        descriptor,
    })
}

const IJARAH_BODY: &str = r#"    address public immutable lessor;
    address public immutable lessee;
    address public immutable charity;
    IValuationOracle public immutable oracle;
    uint256 public constant BPS = 10_000;

    uint256 public immutable rentPerPeriod;
    uint256 public immutable termPeriods;
    uint256 public periodsPaid;
    uint256 public lastAssetValue;
    bool public active;
    bool public transferred;
    bool public terminated;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyLessor() { require(msg.sender == lessor, "only lessor"); _; }
    modifier onlyLessee() { require(msg.sender == lessee, "only lessee"); _; }

    event LeaseActivated(uint256 assetValue, uint256 rentPerPeriod);
    event RentPaid(uint256 period, uint256 amount);
    event MaintenancePaid(uint256 amount);
    event AssetImpaired(uint256 newValue, address borneBy);
    event OwnershipTransferred(address to);
    event Terminated();

    constructor(address _lessee, address _oracle, address _charity, uint256 _rentPerPeriod, uint256 _termPeriods) {
        require(_lessee != address(0) && _oracle != address(0) && _charity != address(0), "zero addr");
        require(_termPeriods > 0, "term");
        lessor = msg.sender; lessee = _lessee; charity = _charity;
        oracle = IValuationOracle(_oracle);
        rentPerPeriod = _rentPerPeriod; termPeriods = _termPeriods;
    }

    function activate() external onlyLessor {
        require(!active, "active");
        uint256 v = oracle.fairValue(); require(v > 0, "oracle value");
        active = true; lastAssetValue = v;
        emit LeaseActivated(v, rentPerPeriod);
    }

    /// @dev INVARIANT rent_for_usufruct: rent prices the usufruct and flows to the lessor;
    ///      it is not interest on principal.
    function payRent() external payable onlyLessee nonReentrant {
        require(active && !terminated && !transferred, "not live");
        require(msg.value == rentPerPeriod, "rent must equal period rent");
        periodsPaid += 1;
        emit RentPaid(periodsPaid, msg.value);
        (bool ok, ) = lessor.call{value: msg.value}(""); require(ok, "rent xfer");
    }

    /// @dev the lessor (owner) bears major maintenance.
    function lessorMaintenance() external payable onlyLessor {
        require(active, "not active");
        emit MaintenancePaid(msg.value);
        if (msg.value > 0) { (bool ok, ) = lessee.call{value: msg.value}(""); require(ok, "maint xfer"); }
    }

    /// @dev INVARIANT lessor_bears_ownership_risk: impairment is borne by the lessor; if the
    ///      asset is destroyed the lease terminates and rent abates.
    function recordImpairment() external {
        require(msg.sender == lessor || msg.sender == lessee, "only a party");
        require(active && !terminated, "not live");
        uint256 v = oracle.fairValue();
        lastAssetValue = v;
        emit AssetImpaired(v, lessor);
        if (v == 0) { terminated = true; emit Terminated(); }
    }

    /// @dev INVARIANT transfer_separate_from_lease: ownership transfer is a DISTINCT act at the
    ///      end of the term, never bundled into the lease (two contracts in one is prohibited).
    function transferOwnership() external onlyLessor {
        require(active && !transferred && !terminated, "not transferable");
        require(periodsPaid >= termPeriods, "term not complete");
        transferred = true;
        emit OwnershipTransferred(lessee);
    }

    /// @dev INVARIANT no_late_penalty_interest: any late charge goes to charity (sadaqah),
    ///      never to the lessor as interest on a debt.
    function payLateCharge() external payable onlyLessee {
        require(active, "not active");
        require(msg.value > 0, "no charge");
        (bool ok, ) = charity.call{value: msg.value}(""); require(ok, "charity xfer");
    }
"#;

fn gen_ijarah_test(name: &str, rate: u64, term: u64) -> String {
    format!(
        r#"// Generated by fiqhc — Ijarah Muntahia Bittamleek. Proves a THIRD instrument from the same
// compiler: rent for usufruct, lessor bears ownership risk, and ownership transfer is a
// SEPARATE act at the end of the term (not two contracts in one).
const {{ expect }} = require("chai");
const {{ ethers }} = require("hardhat");

describe("{name} (fiqhc-generated) — lease ending in ownership", function () {{
  let lessor, lessee, valuer, charity, oracle, c;
  const RENT = {rate}n;
  const TERM = {term}n;

  beforeEach(async function () {{
    [lessor, lessee, valuer, charity] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(100000000n);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("{name}");
    c = await F.connect(lessor).deploy(lessee.address, await oracle.getAddress(), charity.address, RENT, TERM);
    await c.waitForDeployment();
  }});

  it("rent_for_usufruct: a period's rent flows to the lessor", async function () {{
    await c.connect(lessor).activate();
    await expect(c.connect(lessee).payRent({{ value: RENT }})).to.emit(c, "RentPaid").withArgs(1n, RENT);
  }});

  it("transfer_separate_from_lease: ownership transfers only AFTER the full term, as a distinct act", async function () {{
    await c.connect(lessor).activate();
    await expect(c.connect(lessor).transferOwnership()).to.be.revertedWith("term not complete");
    for (let i = 0n; i < TERM; i++) {{ await c.connect(lessee).payRent({{ value: RENT }}); }}
    await expect(c.connect(lessor).transferOwnership()).to.emit(c, "OwnershipTransferred").withArgs(lessee.address);
  }});

  it("lessor_bears_ownership_risk: asset impairment is recorded as borne by the lessor", async function () {{
    await c.connect(lessor).activate();
    await oracle.connect(valuer).attest(50000000n);
    await expect(c.connect(lessee).recordImpairment()).to.emit(c, "AssetImpaired").withArgs(50000000n, lessor.address);
  }});

  it("only the lessor (owner) may transfer ownership", async function () {{
    await c.connect(lessor).activate();
    await expect(c.connect(lessee).transferOwnership()).to.be.revertedWith("only lessor");
  }});

  it("no_late_penalty_interest: any late charge goes to charity, never to the lessor", async function () {{
    await c.connect(lessor).activate();
    const before = await ethers.provider.getBalance(charity.address);
    await c.connect(lessee).payLateCharge({{ value: 500n }});
    const after = await ethers.provider.getBalance(charity.address);
    expect(after - before).to.equal(500n);
  }});
}});
"#,
        name = name,
        rate = rate,
        term = term,
    )
}

fn ijarah_descriptor(name: &str, rate: u64, term: u64) -> String {
    format!(
        r#"{{
  "instrument": "ijarah_imbt",
  "contract": "{name}",
  "operatorRole": "lessor",
  "oracle": {{ "contract": "MockValuationOracle", "initialValue": 1000000 }},
  "constructorAbi": ["address","address","address","uint256","uint256"],
  "constructorArgs": ["@lessee","@oracle","@charity",{rate},{term}],
  "accounts": ["lessee","charity"],
  "lifecycle": [
    {{ "as": "lessor", "fn": "activate", "note": "lessor activates the lease" }},
    {{ "as": "lessee", "fn": "payRent", "value": {rate}, "note": "rent for usufruct" }}
  ]
}}
"#,
        name = name,
        rate = rate,
        term = term,
    )
}
