// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.24;

/// @title ZKVerifier
/// @notice Groth16 BN254 verifier for RoyaltySplitCircuit.
///         The verifying key (alpha, beta, gamma, delta, IC) is set at deploy time
///         from the output of `cargo run --bin setup_ceremony`.
///
///         PUBLIC INPUTS: [band, basis_points_sum, split_commitment]
///           band              must be in {0,1,2} — enforced by circuit constraint.
///           basis_points_sum  must equal 10_000.
///           split_commitment  = Σ (bps[i] * uint128(uint160(artists[i])))
///                               Binds the proof to the specific (artist, bps) allocation.
///                               Computed on-chain by RoyaltyDistributor and verified here.
///
///         SECURITY NOTE: The verifying key MUST be set before any distribution
///         is processed. An unset VK means verifyProof() always returns false,
///         which causes RoyaltyDistributor to revert — protecting against
///         unverified distributions until the ceremony is complete.
///
///         SECURITY NOTE: split_commitment prevents proof replay attacks.
///         A valid proof for one (artist, bps) allocation cannot be submitted
///         with a different allocation — the commitment will not match.
contract ZKVerifier {
    // BN254 field modulus
    uint256 constant FIELD_MODULUS =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;

    struct VerifyingKey {
        uint256[2] alpha;
        uint256[2][2] beta;
        uint256[2][2] gamma;
        uint256[2][2] delta;
        uint256[2][]  ic;   // one per public input + 1 = 4 points (band, bps, commitment)
        bool set;
    }

    struct Proof {
        uint256[2]   a;
        uint256[2][2] b;
        uint256[2]   c;
    }

    VerifyingKey private vk;
    address public immutable admin;

    event VerifyingKeySet(uint256 timestamp);

    modifier onlyAdmin() {
        require(msg.sender == admin, "ZKVerifier: only admin");
        _;
    }

    constructor() { admin = msg.sender; }

    /// Set the Groth16 verifying key from the trusted setup ceremony output.
    /// Can only be called once. After this, verifyProof() is active.
    /// Requires exactly 4 IC points: ic[0] + ic[band] + ic[bps] + ic[commitment].
    function setVerifyingKey(
        uint256[2]    calldata alpha,
        uint256[2][2] calldata beta,
        uint256[2][2] calldata gamma,
        uint256[2][2] calldata delta,
        uint256[2][]  calldata ic
    ) external onlyAdmin {
        require(!vk.set, "ZKVerifier: key already set");
        require(ic.length == 4, "ZKVerifier: need 4 IC points (1 + 3 inputs: band, bps, commitment)");
        vk.alpha = alpha;
        vk.beta  = beta;
        vk.gamma = gamma;
        vk.delta = delta;
        delete vk.ic;
        for (uint i = 0; i < ic.length; i++) { vk.ic.push(ic[i]); }
        vk.set = true;
        emit VerifyingKeySet(block.timestamp);
    }

    /// Verify a Groth16 proof.
    /// @param band              Band public input (0=Common, 1=Rare, 2=Legendary)
    /// @param basisPointsSum    Must equal 10_000
    /// @param splitCommitment   Σ (bps[i] * uint128(uint160(artists[i])))
    ///                          Computed by RoyaltyDistributor from the calldata arrays.
    /// @param proof             192-byte encoded Groth16 proof
    function verifyProof(
        uint8   band,
        uint256 basisPointsSum,
        uint256 splitCommitment,
        bytes   calldata proof
    ) external view returns (bool) {
        require(vk.set, "ZKVerifier: verifying key not set");
        require(band <= 2, "ZKVerifier: band out of range");
        require(basisPointsSum == 10_000, "ZKVerifier: bps must equal 10000");

        if (proof.length != 192) return false;
        Proof memory p = _decodeProof(proof);

        uint256[3] memory publicInputs;
        publicInputs[0] = uint256(band);
        publicInputs[1] = basisPointsSum;
        publicInputs[2] = splitCommitment;

        return _groth16Verify(p, publicInputs);
    }

    function _decodeProof(bytes calldata data) private pure returns (Proof memory p) {
        require(data.length == 192, "invalid proof length");
        (p.a[0], p.a[1])          = abi.decode(data[ 0: 64], (uint256, uint256));
        (p.b[0][0], p.b[0][1],
         p.b[1][0], p.b[1][1])    = abi.decode(data[64:128], (uint256,uint256,uint256,uint256));
        (p.c[0], p.c[1])          = abi.decode(data[128:192],(uint256, uint256));
    }

    function _groth16Verify(
        Proof memory proof, uint256[3] memory inputs
    ) private view returns (bool) {
        // Compute linear combination of IC points with public inputs
        uint256[2] memory acc;
        acc[0] = vk.ic[0][0];
        acc[1] = vk.ic[0][1];
        for (uint i = 0; i < inputs.length; i++) {
            require(inputs[i] < FIELD_MODULUS, "input out of field");
            (uint256 x, uint256 y) = _ecMul(vk.ic[i+1][0], vk.ic[i+1][1], inputs[i]);
            (acc[0], acc[1]) = _ecAdd(acc[0], acc[1], x, y);
        }
        // Pairing check: e(A,B) = e(alpha,beta) · e(acc,gamma) · e(C,delta)
        return _pairingCheck(
            proof.a, proof.b,
            vk.alpha, vk.beta,
            [acc[0], acc[1]], vk.gamma,
            proof.c, vk.delta
        );
    }

    // ECC precompile wrappers
    function _ecAdd(uint256 x1, uint256 y1, uint256 x2, uint256 y2)
        private view returns (uint256 rx, uint256 ry)
    {
        (bool ok, bytes memory res) = address(6).staticcall(
            abi.encode(x1, y1, x2, y2));
        require(ok, "ecAdd failed");
        (rx, ry) = abi.decode(res, (uint256, uint256));
    }

    function _ecMul(uint256 x, uint256 y, uint256 scalar)
        private view returns (uint256 rx, uint256 ry)
    {
        (bool ok, bytes memory res) = address(7).staticcall(
            abi.encode(x, y, scalar));
        require(ok, "ecMul failed");
        (rx, ry) = abi.decode(res, (uint256, uint256));
    }

    function _pairingCheck(
        uint256[2] memory a,   uint256[2][2] memory b,
        uint256[2] memory al,  uint256[2][2] memory be,
        uint256[2] memory acc, uint256[2][2] memory ga,
        uint256[2] memory c,   uint256[2][2] memory de
    ) private view returns (bool) {
        uint256[24] memory input;
        input[0]=a[0];   input[1]=a[1];
        input[2]=b[0][0];input[3]=b[0][1];input[4]=b[1][0];input[5]=b[1][1];
        input[6]=al[0];  input[7]=al[1];
        input[8]=be[0][0];input[9]=be[0][1];input[10]=be[1][0];input[11]=be[1][1];
        input[12]=acc[0];input[13]=acc[1];
        input[14]=ga[0][0];input[15]=ga[0][1];input[16]=ga[1][0];input[17]=ga[1][1];
        input[18]=c[0];  input[19]=c[1];
        input[20]=de[0][0];input[21]=de[0][1];input[22]=de[1][0];input[23]=de[1][1];
        (bool ok, bytes memory res) = address(8).staticcall(abi.encode(input));
        require(ok, "pairing failed");
        return abi.decode(res, (bool));
    }
}
