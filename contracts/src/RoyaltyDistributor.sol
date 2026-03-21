// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.24;

import "./ZKVerifier.sol";
import "./MasterPattern.sol";

/// @title RoyaltyDistributor
/// @notice Distributes BTT royalties to artists with ZK-verified splits.
///
/// ╔══════════════════════════════════════════════════════════════════╗
/// ║  DEFI SECURITY: SIX PROTECTIONS IMPLEMENTED                     ║
/// ║                                                                  ║
/// ║  1. REENTRANCY GUARD — locked bool, CEI pattern                  ║
/// ║     Prevents malicious ERC-20 from re-entering distribute()      ║
/// ║                                                                  ║
/// ║  2. ZK PROOF WITH SPLIT COMMITMENT — verifyProof() on-chain      ║
/// ║     band, bps_sum, AND Σ(bps[i]*addr[i]) cryptographically bound ║
/// ║     Proof cannot be replayed against a different artist set       ║
/// ║                                                                  ║
/// ║  3. VALUE CAP — MAX_DISTRIBUTION_BTT per tx                       ║
/// ║     Limits blast radius of any single exploit                    ║
/// ║                                                                  ║
/// ║  4. TIMELOCK — large distributions queued, 48h delay             ║
/// ║     Anomalous txns catchable before execution                    ║
/// ║                                                                  ║
/// ║  5. IMMUTABLE PROXY — no upgradeability                           ║
/// ║     Upgrade paths are a primary DeFi exploit vector              ║
/// ║                                                                  ║
/// ║  6. TIMELOCK CANCELLATION — admin can cancel queued distributions ║
/// ║     Provides exploit-response capability for queued bad params   ║
/// ╚══════════════════════════════════════════════════════════════════╝

interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

contract RoyaltyDistributor {
    using MasterPattern for bytes32;

    // ── Constants ────────────────────────────────────────────────────
    uint256 public constant BASIS_POINTS      = 10_000;
    uint256 public constant MAX_ARTISTS       = 16;

    /// Max BTT distributable in a single non-timelocked transaction.
    /// Large distributions go through the timelock queue.
    uint256 public constant MAX_DISTRIBUTION_BTT = 1_000_000 * 1e18; // 1M BTT

    /// Timelock delay for distributions above MAX_DISTRIBUTION_BTT.
    uint256 public constant TIMELOCK_DELAY = 48 hours;

    // ── State ─────────────────────────────────────────────────────────
    IERC20          public immutable btt;       // BTT ERC-20 token
    ZKVerifier      public immutable verifier;  // Groth16 verifier
    address         public immutable admin;

    // ── Reentrancy guard (FIX 1) ─────────────────────────────────────
    bool private locked;
    modifier nonReentrant() {
        require(!locked, "RoyaltyDistributor: reentrant call");
        locked = true;
        _;
        locked = false;
    }

    // ── Timelock queue (FIX 4) ───────────────────────────────────────
    struct PendingDistribution {
        bytes32   cid;
        address[] artists;
        uint16[]  bps;
        uint8     band;
        bytes     proof;
        uint256   totalBtt;
        uint256   executeAfter;
        bool      executed;
        bool      cancelled;
    }
    mapping(bytes32 => PendingDistribution) public pendingDistributions;

    // ── Track records ─────────────────────────────────────────────────
    struct TrackRecord {
        bool    distributed;
        uint8   band;
        uint8   bandResidue;
        uint256 totalBttDistributed;
        uint256 timestamp;
    }
    mapping(bytes32 => TrackRecord) public trackRecords;

    // ── Events ────────────────────────────────────────────────────────
    event Distributed(
        bytes32 indexed cid,
        uint256 totalBtt,
        uint8   band,
        string  rarityTier
    );
    event DistributionQueued(bytes32 indexed cid, uint256 executeAfter);
    event TimelockExecuted(bytes32 indexed cid);
    event TimelockCancelled(bytes32 indexed cid, address indexed by);
    event EmergencyPause(address indexed by);

    // ── Emergency pause (for exploit response) ─────────────────────────
    bool public paused;
    modifier notPaused() { require(!paused, "RoyaltyDistributor: paused"); _; }
    modifier onlyAdmin()  { require(msg.sender == admin, "not admin"); _; }

    // ── Constructor (IMMUTABLE — no proxy, no upgrade, FIX 5) ─────────
    constructor(address _btt, address _verifier) {
        require(_btt != address(0),      "zero BTT address");
        require(_verifier != address(0), "zero verifier address");
        btt      = IERC20(_btt);
        verifier = ZKVerifier(_verifier);
        admin    = msg.sender;
    }

    /// @notice Compute the split commitment for a given artist/bps allocation.
    /// @dev    commitment = Σ (bps[i] * uint128(uint160(artists[i])))
    ///         This is the same formula used in the ZK circuit witness.
    ///         Both must agree for proof verification to succeed.
    function _splitCommitment(
        address[] calldata artists,
        uint16[]  calldata bps
    ) private pure returns (uint256 commitment) {
        for (uint i = 0; i < artists.length; i++) {
            commitment += uint256(bps[i]) * uint256(uint128(uint160(artists[i])));
        }
    }

    /// @notice Distribute BTT royalties to a set of artists.
    /// @param cid      BTFS content CID (SHA-256, 32 bytes)
    /// @param artists  Artist EVM addresses
    /// @param bps      Basis points per artist (Σ must equal 10_000)
    /// @param band     Master Pattern band (0=Common, 1=Rare, 2=Legendary)
    /// @param proof    192-byte Groth16 proof (band + splits + commitment)
    /// @param totalBtt Total BTT to distribute (in wei)
    function distribute(
        bytes32          cid,
        address[] calldata artists,
        uint16[]  calldata bps,
        uint8              band,
        bytes     calldata proof,
        uint256            totalBtt
    ) external notPaused nonReentrant {

        // ── Input validation (LangSec-style boundary checks) ─────────
        require(artists.length > 0,                  "no artists");
        require(artists.length <= MAX_ARTISTS,        "too many artists");
        require(artists.length == bps.length,        "length mismatch");
        require(band <= 2,                           "invalid band");
        require(!trackRecords[cid].distributed,      "already distributed");
        require(totalBtt > 0,                        "zero amount");

        // ── Basis points must sum to exactly 10,000 ───────────────────
        uint256 bpsSum;
        for (uint i = 0; i < bps.length; i++) {
            require(artists[i] != address(0), "zero artist address");
            bpsSum += bps[i];
        }
        require(bpsSum == BASIS_POINTS, "bps must sum to 10000");

        // ── ZK proof verification with split commitment (FIX 2) ───────
        // Compute commitment on-chain from the provided artists/bps.
        // The ZK circuit constrains the same formula — proof will fail
        // if the caller substitutes different artists or bps values.
        uint256 commitment = _splitCommitment(artists, bps);
        require(
            verifier.verifyProof(band, BASIS_POINTS, commitment, proof),
            "RoyaltyDistributor: invalid ZK proof"
        );

        // ── Value cap check (FIX 3) ───────────────────────────────────
        if (totalBtt > MAX_DISTRIBUTION_BTT) {
            // Queue for timelock instead of immediate execution
            _queueDistribution(cid, artists, bps, band, proof, totalBtt);
            return;
        }

        // ── Checks-Effects-Interactions (CEI pattern, reentrancy prevention) ──
        // EFFECTS first: record state before any external calls
        trackRecords[cid] = TrackRecord({
            distributed:          true,
            band:                 band,
            bandResidue:          uint8((4 + 3 + 2 - band) % 9),
            totalBttDistributed:  totalBtt,
            timestamp:            block.timestamp
        });

        MasterPattern.Fingerprint memory _fp = MasterPattern.fingerprint(cid, bytes32(totalBtt));
        string memory tier = MasterPattern.rarityTier(band);

        // suppress unused variable warning — fingerprint is computed for event emission
        // and future indexing; individual fields unused here.
        (_fp);

        emit Distributed(cid, totalBtt, band, tier);

        // INTERACTIONS last: external calls after all state changes
        uint256 distributed;
        for (uint i = 0; i < artists.length; i++) {
            uint256 amount = (totalBtt * bps[i]) / BASIS_POINTS;
            distributed += amount;
            require(
                btt.transfer(artists[i], amount),
                "RoyaltyDistributor: BTT transfer failed"
            );
        }
        // Dust from integer division goes to admin
        uint256 dust = totalBtt - distributed;
        if (dust > 0) {
            require(btt.transfer(admin, dust), "dust transfer failed");
        }
    }

    /// @notice Queue a large distribution (above MAX_DISTRIBUTION_BTT) for timelock.
    function _queueDistribution(
        bytes32 cid, address[] calldata artists, uint16[] calldata bps,
        uint8 band, bytes calldata proof, uint256 totalBtt
    ) private {
        uint256 executeAfter = block.timestamp + TIMELOCK_DELAY;
        pendingDistributions[cid] = PendingDistribution({
            cid: cid, artists: artists, bps: bps, band: band,
            proof: proof, totalBtt: totalBtt, executeAfter: executeAfter,
            executed: false, cancelled: false
        });
        emit DistributionQueued(cid, executeAfter);
    }

    /// @notice Execute a timelocked distribution after the delay has passed.
    function executeQueued(bytes32 cid) external notPaused nonReentrant {
        PendingDistribution storage pd = pendingDistributions[cid];
        require(!pd.executed,                  "already executed");
        require(!pd.cancelled,                 "distribution cancelled");
        require(pd.executeAfter > 0,           "not queued");
        require(block.timestamp >= pd.executeAfter, "timelock: too early");
        require(!trackRecords[cid].distributed, "already distributed");

        // Re-verify proof at execution time (not just queue time)
        // Re-compute commitment from stored artists/bps
        uint256 commitment;
        for (uint i = 0; i < pd.artists.length; i++) {
            commitment += uint256(pd.bps[i]) * uint256(uint128(uint160(pd.artists[i])));
        }
        require(
            verifier.verifyProof(pd.band, BASIS_POINTS, commitment, pd.proof),
            "ZK proof invalid at execution"
        );

        pd.executed = true;
        trackRecords[cid] = TrackRecord({
            distributed: true, band: pd.band,
            bandResidue: uint8((4 + 3 + 2 - pd.band) % 9),
            totalBttDistributed: pd.totalBtt, timestamp: block.timestamp
        });

        emit TimelockExecuted(cid);

        uint256 dist;
        for (uint i = 0; i < pd.artists.length; i++) {
            uint256 amount = (pd.totalBtt * pd.bps[i]) / BASIS_POINTS;
            dist += amount;
            require(btt.transfer(pd.artists[i], amount), "transfer failed");
        }
        uint256 dust = pd.totalBtt - dist;
        if (dust > 0) { require(btt.transfer(admin, dust), "dust failed"); }
    }

    /// @notice Cancel a pending timelocked distribution (FIX 6).
    /// @dev    Admin-only. Allows exploit-response before a queued bad-params
    ///         distribution executes. Once cancelled the CID is not marked as
    ///         distributed, so a corrected distribution can be submitted later.
    function cancelQueued(bytes32 cid) external onlyAdmin {
        PendingDistribution storage pd = pendingDistributions[cid];
        require(pd.executeAfter > 0,  "not queued");
        require(!pd.executed,         "already executed");
        require(!pd.cancelled,        "already cancelled");
        pd.cancelled = true;
        emit TimelockCancelled(cid, msg.sender);
    }

    /// @notice Emergency pause — halts all distributions (exploit response).
    function emergencyPause() external onlyAdmin {
        paused = true;
        emit EmergencyPause(msg.sender);
    }

    function unpause() external onlyAdmin { paused = false; }

    function getTrackRecord(bytes32 cid) external view returns (TrackRecord memory) {
        return trackRecords[cid];
    }
}
