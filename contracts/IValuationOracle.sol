// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title  Valuation Oracle interface
/// @notice The honest trust boundary. Valuation is attested by an INDEPENDENT
///         party, never self-reported by a contracting partner. In production
///         this is a licensed valuer or a decentralised oracle network; the
///         oracle's integrity is the gharar locus the paper must discuss openly.
interface IValuationOracle {
    /// @return the current attested fair value of the whole asset
    function fairValue() external view returns (uint256);
}
