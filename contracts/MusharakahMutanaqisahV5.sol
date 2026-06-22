// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IValuationOracle} from "./IValuationOracle.sol";

/// @title  Musharakah Mutanaqisah V5 - defect rescission, authoritative faskh, maslahah disposition
/// @notice Extends V4 (capital custody + khiyar al-shart + iqalah) with the two corrections a
///         scholarly review aid flagged:
///          (1) khiyar al-'ayb (defect rescission) adjudicated by an AGREED ARBITER (a qadi /
///              arbitration proxy), plus an authoritative judicial faskh path — so lawful redress
///              is never foreclosed (the foreclosure of which was likened to tadlis / concealment);
///          (2) on settlement the impaired remainder (the realised-loss residue) is directed to an
///              AGREED MASLAHAH / WAQF fund rather than stranded, avoiding idha'at al-mal.
/// @dev    Whether these encodings satisfy the fiqh is a qualified scholar's ruling, not an
///         engineering claim. The candidate evidences brought for them are unverified and require
///         human takhrij before reliance.
contract MusharakahMutanaqisahV5 {
    address public immutable bank;
    address public immutable client;
    address public immutable arbiter;       // agreed dispute-resolver (qadi / arbitration proxy)
    address public immutable maslahahFund;  // agreed beneficiary for the impaired loss residue
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

    event Funded(address who, uint256 amount);
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

    constructor(
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

    function fundBank() external payable onlyBank {
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

    function rentDue() public view returns (uint256) { return rentPerPeriodPerBps * bankShareBps; }

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

    /// @notice Dissolve at current value; the impaired remainder (loss residue) is directed to
    ///         the agreed maslahah fund rather than stranded -> avoids idha'at al-mal.
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

    // --- rescission family ---
    function rescindKhiyar() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        require(block.timestamp <= khiyarDeadline, "khiyar window closed");
        require(bankShareBps == initialBankShareBps, "performance begun");
        emit KhiyarRescinded(msg.sender); _unwind();
    }
    function proposeIqalah() external live {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        iqalahProposer = msg.sender; emit IqalahProposed(msg.sender);
    }
    function acceptIqalah() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        require(iqalahProposer != address(0) && msg.sender != iqalahProposer, "needs the other partner");
        require(bankShareBps == initialBankShareBps, "performance begun");
        emit IqalahCompleted(msg.sender); _unwind();
    }

    /// @notice khiyar al-'ayb: either partner may raise a defect; the AGREED ARBITER adjudicates.
    function raiseDefect(string calldata reason) external live {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        defectRaised = true; defectClaimant = msg.sender;
        emit DefectRaised(msg.sender, reason);
    }
    function resolveDefect(bool upheld) external live nonReentrant onlyArbiter {
        require(defectRaised, "no defect raised");
        emit DefectResolved(upheld);
        if (upheld) { _unwind(); } else { defectRaised = false; defectClaimant = address(0); }
    }

    /// @notice Authoritative faskh: the agreed arbiter (qadi proxy) may rescind to resolve a dispute.
    function judicialFaskh() external live nonReentrant onlyArbiter {
        emit JudicialFaskh(msg.sender); _unwind();
    }

    function _unwind() internal {
        uint256 b = bankFunded; uint256 cl = clientFunded;
        rescinded = true; active = false; pool = 0; bankFunded = 0; clientFunded = 0;
        if (b > 0) { (bool ok, ) = bank.call{value: b}(""); require(ok, "bank refund"); }
        if (cl > 0) { (bool ok2, ) = client.call{value: cl}(""); require(ok2, "client refund"); }
        emit Unwound(b, cl);
    }
}
