// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IValuationOracle} from "./IValuationOracle.sol";

/// @title  Musharakah Mutanaqisah V2 - capital custody + settlement
/// @notice Both partners escrow their ownership share of the asset value into the
///         contract. Dissolution (`settle`) pays each partner its share of the
///         oracle's CURRENT value from the pool, so a fall in value provably reduces
///         the financier's recoverable capital -- enforced by transfer, not by an
///         emitted number. This closes the peer-review CRITICAL on V1.
/// @dev    Fiqh basis: risk-sharing is the essence of musharakah; loss follows capital.
///         AAOIFI Shari'ah Standard on Musharakah, No. 12 [verify].
contract MusharakahMutanaqisahV2 {
    address public immutable bank;
    address public immutable client;
    IValuationOracle public immutable oracle;
    uint256 public constant BPS = 10_000;

    uint256 public bankShareBps;
    uint256 public clientShareBps;
    uint256 public initialAssetValue;
    uint256 public immutable rentPerPeriodPerBps;

    uint256 public pool;        // funds held in trust (the asset's monetary representation)
    uint256 public bankFunded;
    uint256 public clientFunded;
    bool public active;
    bool public settled;

    uint256 private _lock = 1;
    modifier nonReentrant() { require(_lock == 1, "reentrant"); _lock = 2; _; _lock = 1; }
    modifier onlyBank() { require(msg.sender == bank, "only bank"); _; }
    modifier onlyClient() { require(msg.sender == client, "only client"); _; }
    modifier whenActive() { require(active && !settled, "not active"); _; }

    event Funded(address who, uint256 amount);
    event Activated(uint256 assetValue);
    event SharePurchased(uint256 bps, uint256 price, uint256 newBankBps);
    event RentPaid(uint256 amount, uint256 onBankBps);
    event Settled(uint256 fairValue, uint256 bankPayout, uint256 clientPayout, uint256 impairedLoss);

    constructor(address _client, address _oracle, uint256 _bankShareBps, uint256 _rentPerPeriodPerBps) {
        require(_client != address(0) && _oracle != address(0), "zero addr");
        require(_bankShareBps > 0 && _bankShareBps < BPS, "bank bps range");
        bank = msg.sender;
        client = _client;
        oracle = IValuationOracle(_oracle);
        bankShareBps = _bankShareBps;
        clientShareBps = BPS - _bankShareBps;
        rentPerPeriodPerBps = _rentPerPeriodPerBps;
    }

    function fundBank() external payable onlyBank {
        require(!active, "already active");
        require(bankFunded == 0, "bank already funded");
        uint256 v0 = oracle.fairValue();
        require(msg.value == v0 * bankShareBps / BPS, "bank must fund its share");
        bankFunded = msg.value;
        pool += msg.value;
        emit Funded(bank, msg.value);
        _tryActivate(v0);
    }

    function fundClient() external payable onlyClient {
        require(!active, "already active");
        require(clientFunded == 0, "client already funded");
        uint256 v0 = oracle.fairValue();
        require(msg.value == v0 * clientShareBps / BPS, "client must fund its share");
        clientFunded = msg.value;
        pool += msg.value;
        emit Funded(client, msg.value);
        _tryActivate(v0);
    }

    function _tryActivate(uint256 v0) internal {
        if (bankFunded > 0 && clientFunded > 0) {
            require(bankFunded + clientFunded == v0, "funds must equal asset value");
            initialAssetValue = v0;
            active = true;
            emit Activated(v0);
        }
    }

    function rentDue() public view returns (uint256) {
        return rentPerPeriodPerBps * bankShareBps;
    }

    function payRent() external payable onlyClient whenActive nonReentrant {
        uint256 due = rentDue();
        require(msg.value == due, "rent != due");
        emit RentPaid(due, bankShareBps);
        (bool ok, ) = bank.call{value: msg.value}("");
        require(ok, "rent xfer");
    }

    /// @notice Client buys bank's bps at the oracle's current fair value. The payment
    ///         enters the pool and an equal amount of the bank's capital exits to the
    ///         bank, so the pool keeps representing the asset; ownership shifts.
    function buyShare(uint256 bps) external payable onlyClient whenActive nonReentrant {
        require(bps > 0 && bps <= bankShareBps, "bps range");
        uint256 f = oracle.fairValue();
        require(f > 0, "oracle value");
        uint256 price = f * bps / BPS;
        require(msg.value == price, "value != fair price");
        bankShareBps -= bps;
        clientShareBps += bps;
        pool += msg.value;
        require(pool >= price, "pool underflow");
        pool -= price;
        emit SharePurchased(bps, price, bankShareBps);
        (bool ok, ) = bank.call{value: price}("");
        require(ok, "buyout xfer");
    }

    /// @notice Dissolve at the oracle's current value: pay each partner its share of
    ///         the CURRENT value from the pool. If value fell, each receives less than
    ///         it funded -> loss borne proportionally, enforced by transfer. The
    ///         impaired remainder stays locked (the realised loss).
    function settle() external whenActive nonReentrant {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        uint256 f = oracle.fairValue();
        require(f > 0, "oracle value");
        uint256 distributable = f > pool ? pool : f;
        uint256 bankPayout = distributable * bankShareBps / BPS;
        uint256 clientPayout = distributable * clientShareBps / BPS;
        uint256 impaired = pool - bankPayout - clientPayout;
        settled = true;
        pool = impaired;
        if (bankPayout > 0) { (bool a, ) = bank.call{value: bankPayout}(""); require(a, "bank payout"); }
        if (clientPayout > 0) { (bool b, ) = client.call{value: clientPayout}(""); require(b, "client payout"); }
        emit Settled(f, bankPayout, clientPayout, impaired);
    }

    function fullyAcquired() external view returns (bool) {
        return bankShareBps == 0;
    }

    receive() external payable {
        revert("use fund functions");
    }
}
