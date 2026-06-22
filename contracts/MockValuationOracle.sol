// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IValuationOracle} from "./IValuationOracle.sol";

/// @notice PoC oracle for testnet evaluation. In production this is replaced by a
///         licensed valuer or a decentralised oracle network. Access-controlled to
///         a single independent valuer -- no contracting partner can attest value.
contract MockValuationOracle is IValuationOracle {
    address public immutable valuer;
    uint256 private _value;

    event ValueAttested(uint256 value);

    constructor(uint256 initialValue) {
        require(initialValue > 0, "initial value required");
        valuer = msg.sender;
        _value = initialValue;
    }

    function attest(uint256 newValue) external {
        require(msg.sender == valuer, "only valuer");
        require(newValue > 0, "value required");
        _value = newValue;
        emit ValueAttested(newValue);
    }

    function fairValue() external view returns (uint256) {
        return _value;
    }
}
