// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// Minimal Hedera Token Service precompile interface (system contract at 0x167).
interface IHederaTokenService {
    function associateToken(address account, address token) external returns (int64 responseCode);
    function transferToken(address token, address sender, address receiver, int64 amount) external returns (int64 responseCode);
}

/// @title AssetTokenCustodian
/// @notice Proof that the enforcement contract can custody and move the REAL HTS
///         asset-share token (MMS): it associates itself with the token and
///         transfers held units to a party. This is the binding step toward an
///         HTS-native Musharakah where buyShare/settle move real ownership units.
/// @dev    The HTS precompile executes only on Hedera (not a local EVM).
contract AssetTokenCustodian {
    address constant HTS = address(0x167);
    int64 constant HTS_SUCCESS = 22; // HederaResponseCodes.SUCCESS
    address public immutable owner;
    address public token;

    event Associated(address token, int64 code);
    event Moved(address token, address to, int64 amount, int64 code);

    constructor() {
        owner = msg.sender;
    }

    /// @notice Associate this contract with the HTS token so it can hold units.
    function associate(address _token) external returns (int64 code) {
        require(msg.sender == owner, "only owner");
        token = _token;
        code = IHederaTokenService(HTS).associateToken(address(this), _token);
        require(code == HTS_SUCCESS, "associate failed");
        emit Associated(_token, code);
    }

    /// @notice Move `amount` units of the held asset token from this contract to `to`.
    ///         `to` must already be associated with the token.
    function transferShare(address to, int64 amount) external returns (int64 code) {
        require(msg.sender == owner, "only owner");
        require(amount > 0, "amount");
        code = IHederaTokenService(HTS).transferToken(token, address(this), to, amount);
        require(code == HTS_SUCCESS, "transfer failed");
        emit Moved(token, to, amount, code);
    }
}
