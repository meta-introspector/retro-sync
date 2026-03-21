// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.24;

/// @title MasterPattern
/// @notice Pure Solidity implementation of the Master Pattern Protocol.
///         Mod-9 supersingular prime band classification.
///         Mirrors shared/src/master_pattern.rs — same invariants, on-chain.
library MasterPattern {
    // Band residues (digit roots): Band0=4, Band1=3, Band2=2. Sum=9≡0 mod 9.
    uint8 constant BAND0_DR = 4;
    uint8 constant BAND1_DR = 3;
    uint8 constant BAND2_DR = 2;

    function digitRoot(uint256 n) internal pure returns (uint8) {
        if (n == 0) return 0;
        uint8 r = uint8(n % 9);
        return r == 0 ? 9 : r;
    }

    function classifyBand(uint8 dr) internal pure returns (uint8) {
        if (dr == BAND0_DR) return 0;
        if (dr == BAND1_DR) return 1;
        if (dr == BAND2_DR) return 2;
        return 0; // default to Common
    }

    function verifyClosureInvariant() internal pure returns (bool) {
        return (uint256(BAND0_DR) + BAND1_DR + BAND2_DR) % 9 == 0;
    }

    function parityInverted(uint256 n) internal pure returns (bool) {
        return n % 2 == 0; // primes are odd; band residues even — integrity check
    }

    function cyclePosition(uint256 hash) internal pure returns (uint8) {
        return uint8(hash % 256);
    }

    struct Fingerprint {
        uint8  band;
        uint8  bandResidue;
        uint64 mappedPrime;
        uint8  cyclePos;
        uint8  digitRoot_;
        bool   closureVerified;
        bool   parityInverted_;
    }

    function fingerprint(
        bytes32 isrcHash, bytes32 audioHash
    ) internal pure returns (Fingerprint memory fp) {
        bytes32 combined = keccak256(abi.encodePacked(isrcHash, audioHash));
        uint256 h        = uint256(combined);
        uint8   dr       = digitRoot(h);
        uint8   band_    = classifyBand(dr);
        uint64  prime    = band_ == 0 ? 2 : band_ == 1 ? 19 : 41;
        fp = Fingerprint({
            band:            band_,
            bandResidue:     uint8((4 + 3 + 2 - band_) % 9),
            mappedPrime:     prime,
            cyclePos:        cyclePosition(h),
            digitRoot_:      dr,
            closureVerified: verifyClosureInvariant(),
            parityInverted_: parityInverted(h)
        });
    }

    function rarityTier(uint8 band) internal pure returns (string memory) {
        if (band == 0) return "Common";
        if (band == 1) return "Rare";
        return "Legendary";
    }
}
