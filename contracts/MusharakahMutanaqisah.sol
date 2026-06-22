// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IValuationOracle} from "./IValuationOracle.sol";

/// @title  Musharakah Mutanaqisah (Diminishing Partnership)
/// @notice Compliance-by-construction. The contract ENFORCES the conditions
///         separating a genuine diminishing partnership (shirkah al-milk +
///         ijarah + buyout at independently-attested fair value, with
///         proportional loss-sharing) from interest-bearing debt disguised
///         as partnership.
/// @dev    Design-science artifact for Hedera testnet evaluation.
///         Fiqh refs: AAOIFI Shari'ah Standard on Musharakah (No. 12);
///         Qur'an al-Baqarah 2:275 (prohibition of riba). Verify before publication.
contract MusharakahMutanaqisah {
    address public immutable bank;
    address public immutable client;
    IValuationOracle public immutable oracle;

    uint256 public constant BPS = 10_000;
    uint256 public bankShareBps;
    uint256 public clientShareBps;
    uint256 public assetValue;            // last synced attested fair value
    uint256 public immutable initialAssetValue;

    uint256 public immutable rentPerPeriodPerBps;
    bool public lossRecorded;

    event SharePurchased(uint256 bpsBought, uint256 priceAtFairValue, uint256 newBankShareBps);
    event RentPaid(uint256 amount, uint256 onBankShareBps);
    event Revalued(uint256 oldValue, uint256 newValue);
    event LossShared(uint256 newAssetValue, uint256 bankWriteDown, uint256 clientWriteDown);

    modifier onlyClient() { require(msg.sender == client, "only client"); _; }
    modifier onlyBank()   { require(msg.sender == bank, "only bank"); _; }

    constructor(
        address _client,
        address _oracle,
        uint256 _bankShareBps,
        uint256 _rentPerPeriodPerBps
    ) {
        require(_client != address(0), "client required");
        require(_oracle != address(0), "oracle required");
        require(_bankShareBps > 0 && _bankShareBps < BPS, "bank share in (0,100%)");
        bank = msg.sender;
        client = _client;
        oracle = IValuationOracle(_oracle);
        bankShareBps = _bankShareBps;
        clientShareBps = BPS - _bankShareBps;
        uint256 v = oracle.fairValue();
        require(v > 0, "oracle value required");
        assetValue = v;
        initialAssetValue = v;
        rentPerPeriodPerBps = _rentPerPeriodPerBps;
    }

    /// @dev INVARIANT 1: rent on the bank's CURRENT share only; falls as ownership transfers.
    function rentDue() public view returns (uint256) {
        return rentPerPeriodPerBps * bankShareBps;
    }

    function payRent() external payable onlyClient {
        uint256 due = rentDue();
        require(msg.value == due, "rent must equal due on bank share");
        emit RentPaid(due, bankShareBps);
        (bool ok, ) = bank.call{value: msg.value}("");
        require(ok, "rent transfer failed");
    }

    /// @notice Client buys a slice of the bank's share. The price is derived from
    ///         the INDEPENDENT oracle -- the buyer cannot name their own price and
    ///         no pre-fixed schedule guarantees the bank's capital.
    /// @dev    INVARIANT 2 (anti-riba): price tracks attested fair value; bank bears price risk.
    function buyShare(uint256 bpsToBuy) external payable onlyClient {
        require(bpsToBuy > 0 && bpsToBuy <= bankShareBps, "invalid bps");
        uint256 fairUnitValue = oracle.fairValue();
        require(fairUnitValue > 0, "oracle value required");
        uint256 price = (fairUnitValue * bpsToBuy) / BPS;
        require(msg.value == price, "value must equal fair price of bps bought");

        bankShareBps -= bpsToBuy;
        clientShareBps += bpsToBuy;
        assetValue = fairUnitValue;

        emit SharePurchased(bpsToBuy, price, bankShareBps);
        (bool ok, ) = bank.call{value: msg.value}("");
        require(ok, "buyout transfer failed");
    }

    /// @notice Sync to the oracle's latest attested value. A fall is shared by
    ///         current ownership ratio -- the bank cannot exit whole. Either
    ///         partner may call; neither self-reports the value.
    /// @dev    INVARIANT 3 (risk-sharing): proportional loss separates musharakah
    ///         from a guaranteed-capital loan.
    function syncValuation() external {
        require(msg.sender == bank || msg.sender == client, "only a partner");
        uint256 newValue = oracle.fairValue();
        require(newValue > 0, "oracle value required");
        uint256 old = assetValue;
        assetValue = newValue;
        emit Revalued(old, newValue);
        if (newValue < old) {
            uint256 loss = old - newValue;
            uint256 bankWriteDown = (loss * bankShareBps) / BPS;
            uint256 clientWriteDown = (loss * clientShareBps) / BPS;
            lossRecorded = true;
            emit LossShared(newValue, bankWriteDown, clientWriteDown);
        }
    }

    function fullyAcquired() external view returns (bool) {
        return bankShareBps == 0;
    }
}
