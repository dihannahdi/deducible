// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IValuationOracle} from "./IValuationOracle.sol";

/// @title  Musharakah Mutanaqisah V4 - lawful rescission (khiyar + iqalah)
/// @notice Extends the capital-custody design (V2) with two fiqh rescission rights,
///         reconciling blockchain finality with lawful revocation:
///         - khiyar al-shart: within a stipulated window after activation, either
///           partner may unilaterally rescind (before any buyout/performance);
///         - iqalah: by mutual consent, the partners may cancel and unwind to
///           status quo ante (before any buyout).
///         Rescission returns each partner's funded capital. Post-performance
///         (partial-buyout) unwinding is acknowledged future work.
/// @dev    Whether a given encoding satisfies the fiqh is a scholar's ruling.
///         Iqalah is encouraged in the Sunnah (Sunan collections) [scholar-verify].
contract MusharakahMutanaqisahV4 {
    address public immutable bank;
    address public immutable client;
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

    // --- rescission state ---
    uint256 public immutable khiyarPeriod;
    uint256 public khiyarDeadline;
    bool public rescinded;
    address public iqalahProposer;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyBank() { require(msg.sender == bank, "only bank"); _; }
    modifier onlyClient() { require(msg.sender == client, "only client"); _; }
    modifier live() { require(active && !rescinded, "not live"); _; }

    event Funded(address who, uint256 amount);
    event Activated(uint256 assetValue, uint256 khiyarDeadline);
    event SharePurchased(uint256 bps, uint256 price, uint256 newBankBps);
    event KhiyarRescinded(address by);
    event IqalahProposed(address by);
    event IqalahCompleted(address acceptedBy);
    event Unwound(uint256 bankRefund, uint256 clientRefund);

    constructor(address _client, address _oracle, uint256 _bankShareBps, uint256 _rentPerPeriodPerBps, uint256 _khiyarPeriod) {
        require(_client != address(0) && _oracle != address(0), "zero addr");
        require(_bankShareBps > 0 && _bankShareBps < BPS, "bank bps range");
        bank = msg.sender; client = _client; oracle = IValuationOracle(_oracle);
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

    /// @notice khiyar al-shart: unilateral rescission within the stipulated window,
    ///         permitted only before performance (no buyout yet).
    function rescindKhiyar() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        require(block.timestamp <= khiyarDeadline, "khiyar window closed");
        require(bankShareBps == initialBankShareBps, "performance begun");
        emit KhiyarRescinded(msg.sender);
        _unwind();
    }

    function proposeIqalah() external live {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        iqalahProposer = msg.sender;
        emit IqalahProposed(msg.sender);
    }

    /// @notice iqalah: completes only when the OTHER partner consents; unwinds to
    ///         status quo ante (before performance).
    function acceptIqalah() external live nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        require(iqalahProposer != address(0) && msg.sender != iqalahProposer, "needs the other partner");
        require(bankShareBps == initialBankShareBps, "performance begun");
        emit IqalahCompleted(msg.sender);
        _unwind();
    }

    function _unwind() internal {
        uint256 b = bankFunded; uint256 c = clientFunded;
        rescinded = true; active = false; pool = 0; bankFunded = 0; clientFunded = 0;
        if (b > 0) { (bool ok, ) = bank.call{value: b}(""); require(ok, "bank refund"); }
        if (c > 0) { (bool ok2, ) = client.call{value: c}(""); require(ok2, "client refund"); }
        emit Unwound(b, c);
    }
}
