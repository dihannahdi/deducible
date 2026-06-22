// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IValuationOracle} from "./IValuationOracle.sol";

/// @title  ConsensusValuationOracle — a zero-trust, committee-attested fair value
/// @notice Replaces the single trusted valuer (the residual gharar locus) with a committee
///         of INDEPENDENT attestors. Each attestor signs (round, value); the oracle recovers
///         the signer cryptographically and checks committee membership, so the relayer who
///         submits the batch is not trusted at all. The fair value is the MEDIAN of the
///         attestations that fall within an agreed dispersion band; values outside the band are
///         rejected as outliers.
///
///         The fiqh insight made executable: a price is usable only if it is *ma'lum*
///         (determinable). If fewer than `quorum` independent attestors agree within the
///         `ghararBoundBps` band, the value is *majhul* — that is gharar — and `fairValue()`
///         REVERTS, so a contract cannot transact on an undeterminable value. The gharar
///         boundary thus becomes a quantity computed on-chain (the dispersion of independent
///         attestations), not a hidden assumption.
contract ConsensusValuationOracle is IValuationOracle {
    address public immutable admin;
    uint256 public immutable quorum;          // min attestations within the band for a value to be ma'lum
    uint256 public immutable ghararBoundBps;   // max allowed deviation from the median (bps)
    address[] public committee;
    mapping(address => bool) public isCommittee;

    uint256 public round;                 // latest submitted round
    uint256 public latestResolvedRound;   // latest round that produced a determinable value
    uint256 private _value;

    event CommitteeSet(uint256 size, uint256 quorum, uint256 ghararBoundBps);
    event RoundResolved(uint256 indexed round, uint256 fairValue, uint256 dispersionBps, uint256 inliers);
    event Undeterminable(uint256 indexed round, uint256 dispersionBps, uint256 inliers, string reason);

    constructor(address[] memory _committee, uint256 _quorum, uint256 _ghararBoundBps) {
        require(_quorum >= 1 && _committee.length >= _quorum, "quorum vs committee");
        require(_ghararBoundBps > 0 && _ghararBoundBps < 10_000, "bound range");
        admin = msg.sender;
        quorum = _quorum;
        ghararBoundBps = _ghararBoundBps;
        for (uint256 i = 0; i < _committee.length; i++) {
            address m = _committee[i];
            require(m != address(0) && !isCommittee[m], "bad/dup member");
            isCommittee[m] = true;
            committee.push(m);
        }
        emit CommitteeSet(_committee.length, _quorum, _ghararBoundBps);
    }

    function committeeSize() external view returns (uint256) {
        return committee.length;
    }

    /// @notice Submit a batch of independently-signed attestations for the next round. Anyone may
    ///         relay it; trust rests on the signatures, not the sender.
    function submitRound(uint256 roundId, uint256[] calldata values, bytes[] calldata sigs) external {
        require(roundId == round + 1, "round must be sequential");
        require(values.length == sigs.length, "length mismatch");
        require(values.length >= quorum, "fewer attestations than quorum");

        // 1) every attestation must be signed by a DISTINCT committee member
        address[] memory seen = new address[](values.length);
        for (uint256 i = 0; i < values.length; i++) {
            require(values[i] > 0, "zero value");
            address signer = _recover(_digest(roundId, values[i]), sigs[i]);
            require(isCommittee[signer], "signer not on committee");
            for (uint256 j = 0; j < i; j++) {
                require(seen[j] != signer, "double-sign by a member");
            }
            seen[i] = signer;
        }

        // 2) median of all attestations
        uint256[] memory sorted = _sortCopy(values);
        uint256 med = _median(sorted, sorted.length);

        // 3) keep only inliers within the gharar band around the median
        uint256 lo = med - (med * ghararBoundBps) / 10_000;
        uint256 hi = med + (med * ghararBoundBps) / 10_000;
        uint256 cnt;
        uint256[] memory inliers = new uint256[](sorted.length);
        for (uint256 i = 0; i < sorted.length; i++) {
            if (sorted[i] >= lo && sorted[i] <= hi) {
                inliers[cnt] = sorted[i];
                cnt++;
            }
        }

        uint256 dispBps = _dispersionBps(sorted, sorted.length, med);
        round = roundId;

        // 4) if too few independent attestors agree, the value is majhul (gharar)
        if (cnt < quorum) {
            emit Undeterminable(roundId, dispBps, cnt, "agreement below quorum within gharar bound");
            return; // value NOT updated; fairValue() will revert for this round
        }

        uint256 fair = _median(inliers, cnt);
        _value = fair;
        latestResolvedRound = roundId;
        emit RoundResolved(roundId, fair, _dispersionBps(inliers, cnt, fair), cnt);
    }

    /// @inheritdoc IValuationOracle
    function fairValue() external view returns (uint256) {
        require(round > 0, "no valuation round yet");
        require(latestResolvedRound == round, "gharar: latest valuation round is undeterminable");
        return _value;
    }

    // --- internals ---

    function _digest(uint256 roundId, uint256 value) internal view returns (bytes32) {
        return keccak256(abi.encodePacked(roundId, value, address(this), block.chainid));
    }

    function _recover(bytes32 digest, bytes memory sig) internal pure returns (address) {
        bytes32 ethHash = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", digest));
        require(sig.length == 65, "bad sig length");
        bytes32 r;
        bytes32 s;
        uint8 v;
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
        if (v < 27) v += 27;
        require(v == 27 || v == 28, "bad v");
        address signer = ecrecover(ethHash, v, r, s);
        require(signer != address(0), "bad signature");
        return signer;
    }

    function _sortCopy(uint256[] calldata src) internal pure returns (uint256[] memory) {
        uint256 n = src.length;
        uint256[] memory a = new uint256[](n);
        for (uint256 i = 0; i < n; i++) a[i] = src[i];
        for (uint256 i = 1; i < n; i++) {
            uint256 key = a[i];
            uint256 j = i;
            while (j > 0 && a[j - 1] > key) {
                a[j] = a[j - 1];
                j--;
            }
            a[j] = key;
        }
        return a;
    }

    function _median(uint256[] memory sorted, uint256 n) internal pure returns (uint256) {
        require(n > 0, "empty");
        if (n % 2 == 1) {
            return sorted[n / 2];
        }
        return (sorted[n / 2 - 1] + sorted[n / 2]) / 2;
    }

    /// dispersion = (max - min) / median, in bps, over the first n elements of a sorted array
    function _dispersionBps(uint256[] memory sorted, uint256 n, uint256 med) internal pure returns (uint256) {
        if (n == 0 || med == 0) return 0;
        uint256 spread = sorted[n - 1] - sorted[0];
        return (spread * 10_000) / med;
    }
}
