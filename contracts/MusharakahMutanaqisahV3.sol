// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IHederaTokenService {
    function associateToken(address account, address token) external returns (int64 responseCode);
    function transferToken(address token, address sender, address receiver, int64 amount) external returns (int64 responseCode);
}

interface IValuationOracle {
    function fairValue() external view returns (uint256);
}

/// @title  Musharakah Mutanaqisah V3 (HTS-native)
/// @notice A buyout is ONE atomic transaction: the contract transfers real HTS
///         ownership units (MMS) from its escrow to the client AND forwards the
///         hbar price to the bank. Ownership transfer and payment are inseparable —
///         no off-ledger promise. Price comes from the independent oracle (I2);
///         rent is charged only on the bank's remaining escrowed units (I1).
/// @dev    The HTS precompile (0x167) executes only on Hedera, not a local EVM;
///         V3 is verified on testnet.
contract MusharakahMutanaqisahV3 {
    address constant HTS = address(0x167);
    int64 constant HTS_SUCCESS = 22;

    address public immutable bank;
    address public immutable client;
    IValuationOracle public immutable oracle;
    address public immutable token;
    uint64 public immutable totalUnits;
    uint64 public bankUnits;        // bank's still-owned units, escrowed in this contract
    uint256 public immutable rentPerPeriodPerUnit;
    bool public associated;

    modifier onlyBank() { require(msg.sender == bank, "only bank"); _; }
    modifier onlyClient() { require(msg.sender == client, "only client"); _; }

    event Associated(address token, int64 code);
    event UnitsBought(uint64 units, uint256 price, uint64 bankUnitsLeft, int64 htsCode);

    constructor(
        address _client,
        address _oracle,
        address _token,
        uint64 _totalUnits,
        uint64 _bankUnits,
        uint256 _rentPerPeriodPerUnit
    ) {
        require(_client != address(0) && _oracle != address(0) && _token != address(0), "zero addr");
        require(_bankUnits > 0 && _bankUnits <= _totalUnits, "units range");
        bank = msg.sender;
        client = _client;
        oracle = IValuationOracle(_oracle);
        token = _token;
        totalUnits = _totalUnits;
        bankUnits = _bankUnits;
        rentPerPeriodPerUnit = _rentPerPeriodPerUnit;
    }

    function associate() external onlyBank {
        int64 code = IHederaTokenService(HTS).associateToken(address(this), token);
        require(code == HTS_SUCCESS, "associate failed");
        associated = true;
        emit Associated(token, code);
    }

    /// @dev I1: rent only on the bank's remaining units; falls as units transfer.
    function rentDue() public view returns (uint256) {
        return rentPerPeriodPerUnit * bankUnits;
    }

    /// @notice Atomic buyout: hbar (priced from the oracle) to the bank, and the
    ///         same number of real MMS ownership units from escrow to the client.
    function buyShare(uint64 units) external payable onlyClient {
        require(units > 0 && units <= bankUnits, "units");
        uint256 f = oracle.fairValue();
        require(f > 0, "oracle value");
        uint256 price = f * units / totalUnits;
        require(msg.value == price, "value != fair price");

        bankUnits -= units;
        int64 code = IHederaTokenService(HTS).transferToken(token, address(this), client, int64(uint64(units)));
        require(code == HTS_SUCCESS, "hts transfer failed");
        (bool ok, ) = bank.call{value: msg.value}("");
        require(ok, "hbar transfer failed");
        emit UnitsBought(units, price, bankUnits, code);
    }

    event Dissolved(uint64 bankUnitsReturned);

    /// @notice Wind up the partnership: return the bank's remaining escrowed ownership
    ///         units. Any loss has already been borne via lower buyout proceeds — once
    ///         the oracle marks the asset down, buyouts are priced at the fallen value,
    ///         so the bank receives proportionally less for the units it sells.
    function dissolve() external onlyBank {
        uint64 remaining = bankUnits;
        bankUnits = 0;
        if (remaining > 0) {
            int64 code = IHederaTokenService(HTS).transferToken(token, address(this), bank, int64(uint64(remaining)));
            require(code == HTS_SUCCESS, "dissolve transfer failed");
        }
        emit Dissolved(remaining);
    }

    function fullyAcquired() external view returns (bool) {
        return bankUnits == 0;
    }
}
