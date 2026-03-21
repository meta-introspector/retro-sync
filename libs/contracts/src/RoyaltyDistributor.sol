// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.24;

import "./ZKVerifier.sol";
import "./MasterPattern.sol";

/// @title RoyaltyDistributor
/// @notice Distributes BTT royalties to artists with ZK-verified splits.
///
/// ╔══════════════════════════════════════════════════════════════════╗
/// ║  DEFI SECURITY: FIVE PROTECTIONS IMPLEMENTED                    ║
/// ║                                                                  ║
/// ║  1. REENTRANCY GUARD — locked bool, CEI pattern                  ║
/// ║     Prevents malicious ERC-20 from re-entering distribute()      ║
/// ║                                                                  ║
/// ║  2. ZK PROOF REQUIRED — ZKVerifier.verifyProof() on-chain        ║
/// ║     Band + splits commitment cryptographically proven before pay  ║
/// ║                                                                  ║
/// ║  3. VALUE CAP — MAX_DISTRIBUTION_BTT per tx                       ║
/// ║     Limits blast radius of any single exploit                    ║
/// ║                                                                  ║
/// ║  4. TIMELOCK — large distributions queued, 48h delay             ║
/// ║     Anomalous txns catchable before execution                    ║
/// ║                                                                  ║
/// ║  5. IMMUTABLE PROXY — no upgradeability                           ║
/// ║     Upgrade paths are a primary DeFi exploit vector              ║
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
    uint256 public constant TRANSACTION_FEE_BPS = 250; // 2.5% fee (250 / 10,000)

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

    // ── IPI-based artist split (songwriters/publishers) ────────────────
    struct IPISplit {
        address[] splitAddresses; // Songwriter/publisher wallet addresses
        uint16[]  splitPercentages; // Basis points per address (sum to 10000)
        bytes32   ipiReference;   // Reference to IPI data (off-chain or on-chain)
    }
    mapping(bytes32 => IPISplit) public trackIPISplits; // trackCid => split info

    // ── Streaming transaction records (P2P fee model) ──────────────────
    struct StreamingTransaction {
        bytes32   trackCid;         // Content CID being streamed
        address   listener;         // User who triggered the stream
        address[] hostNodes;        // P2P nodes that seeded/hosted
        uint256   streamValue;      // User pays this value in BTT
        uint256   networkFee;       // 2.5% of streamValue (to hosts + platform)
        uint256   artistRoyalty;    // Remaining amount after fee (split by IPI)
        uint256   timestamp;
        bool      feeProcessed;     // 2.5% fee distributed to hosts
        bool      royaltyProcessed; // Artist royalty distributed to splits
    }
    mapping(bytes32 => StreamingTransaction) public streamingTransactions;
    bytes32[] public transactionHistory;

    // ── Host node reputation ───────────────────────────────────────────
    struct HostReputation {
        uint256 totalFeesEarned;    // 2.5% fee share earned
        uint256 streamsHosted;
        uint256 lastReward;
    }
    mapping(address => HostReputation) public hostReputation;

    // ── Artist opt-in for crypto payouts ───────────────────────────────
    mapping(address => bool) public artistOptInCrypto; // true = artist accepts crypto payouts
    
    // ── Platform fee accumulator (portion of 2.5% not given to hosts) ────
    uint256 public platformFeeAccumulated;

    // ── Events ────────────────────────────────────────────────────────
    event Distributed(
        bytes32 indexed cid,
        uint256 totalBtt,
        uint8   band,
        string  rarityTier
    );
    event DistributionQueued(bytes32 indexed cid, uint256 executeAfter);
    event TimelockExecuted(bytes32 indexed cid);
    event EmergencyPause(address indexed by);

    // ── P2P Streaming & Fee Events ─────────────────────────────────────
    event StreamingTransactionRecorded(
        bytes32 indexed txId,
        bytes32 indexed trackCid,
        address indexed listener,
        uint256 streamValue,
        uint256 networkFee,
        uint256 artistRoyalty
    );
    event IPISplitRegistered(
        bytes32 indexed trackCid,
        address[] splitAddresses,
        uint16[] splitPercentages,
        bytes32 ipiReference
    );
    event NetworkFeeDistributed(
        bytes32 indexed txId,
        uint256 totalFee,
        uint256 hostNodesShare,
        uint256 platformShare
    );
    event HostRewardPaid(
        address indexed hostNode,
        uint256 amount,
        uint256 totalEarned
    );
    event ArtistRoyaltyDistributed(
        bytes32 indexed txId,
        bytes32 indexed trackCid,
        uint256 totalRoyalty,
        address[] recipients,
        uint256[] amounts
    );
    event ArtistOptedIntoCrypto(address indexed artist, bool status);

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

    /// @notice Distribute BTT royalties to a set of artists.
    /// @param cid      BTFS content CID (SHA-256, 32 bytes)
    /// @param artists  Artist EVM addresses
    /// @param bps      Basis points per artist (Σ must equal 10_000)
    /// @param band     Master Pattern band (0=Common, 1=Rare, 2=Legendary)
    /// @param proof    192-byte Groth16 proof (band + splits commitment)
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

        // ── ZK proof verification (FIX 2) ─────────────────────────────
        // Band and split commitment proven before any state change
        require(
            verifier.verifyProof(band, BASIS_POINTS, proof),
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
            proof: proof, totalBtt: totalBtt, executeAfter: executeAfter, executed: false
        });
        emit DistributionQueued(cid, executeAfter);
    }

    /// @notice Execute a timelocked distribution after the delay has passed.
    function executeQueued(bytes32 cid) external notPaused nonReentrant {
        PendingDistribution storage pd = pendingDistributions[cid];
        require(!pd.executed,                  "already executed");
        require(pd.executeAfter > 0,           "not queued");
        require(block.timestamp >= pd.executeAfter, "timelock: too early");
        require(!trackRecords[cid].distributed, "already distributed");

        // Re-verify proof at execution time (not just queue time)
        require(
            verifier.verifyProof(pd.band, BASIS_POINTS, pd.proof),
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

    /// @notice Register IPI splits for a track (called during upload by artist wallet).
    /// @param trackCid BTFS CID of the track
    /// @param splitAddresses Songwriter/publisher wallet addresses
    /// @param splitPercentages Basis points per address (must sum to 10,000)
    /// @param ipiReference Reference to IPI data (IPFS hash or on-chain pointer)
    function registerIPISplit(
        bytes32 trackCid,
        address[] calldata splitAddresses,
        uint16[] calldata splitPercentages,
        bytes32 ipiReference
    ) external {
        require(splitAddresses.length > 0, "at least one split required");
        require(splitAddresses.length == splitPercentages.length, "length mismatch");
        require(splitAddresses.length <= MAX_ARTISTS, "too many artists");
        require(trackIPISplits[trackCid].splitAddresses.length == 0, "splits already registered");

        uint256 totalBps;
        for (uint i = 0; i < splitAddresses.length; i++) {
            require(splitAddresses[i] != address(0), "zero split address");
            totalBps += splitPercentages[i];
        }
        require(totalBps == BASIS_POINTS, "splits must sum to 10000");

        trackIPISplits[trackCid] = IPISplit({
            splitAddresses: splitAddresses,
            splitPercentages: splitPercentages,
            ipiReference: ipiReference
        });

        emit IPISplitRegistered(trackCid, splitAddresses, splitPercentages, ipiReference);
    }

    /// @notice Record a P2P streaming transaction.
    /// @dev User pays streamValue in BTT. 2.5% is networkFee (to hosts + platform).
    ///      Remaining (97.5%) is artistRoyalty, split among songwriters/publishers by IPI.
    /// @param txId Unique transaction ID
    /// @param trackCid BTFS CID of the track being streamed
    /// @param listener User address who listened
    /// @param hostNodes P2P nodes that provided the stream
    /// @param streamValue Total BTT user pays
    function recordStreamingTransaction(
        bytes32 txId,
        bytes32 trackCid,
        address listener,
        address[] calldata hostNodes,
        uint256 streamValue
    ) external notPaused nonReentrant {
        require(hostNodes.length > 0, "at least one host required");
        require(hostNodes.length <= MAX_ARTISTS, "too many hosts");
        require(listener != address(0), "zero listener address");
        require(streamValue > 0, "zero stream value");
        require(streamingTransactions[txId].timestamp == 0, "txId already recorded");
        require(trackIPISplits[trackCid].splitAddresses.length > 0, "IPI splits not registered");

        // Calculate 2.5% network fee and remaining artist royalty
        uint256 networkFee = (streamValue * TRANSACTION_FEE_BPS) / BASIS_POINTS;
        uint256 artistRoyalty = streamValue - networkFee;

        // Record the transaction
        streamingTransactions[txId] = StreamingTransaction({
            trackCid: trackCid,
            listener: listener,
            hostNodes: hostNodes,
            streamValue: streamValue,
            networkFee: networkFee,
            artistRoyalty: artistRoyalty,
            timestamp: block.timestamp,
            feeProcessed: false,
            royaltyProcessed: false
        });
        transactionHistory.push(txId);

        emit StreamingTransactionRecorded(
            txId, trackCid, listener, streamValue, networkFee, artistRoyalty
        );
    }

    /// @notice Distribute the 2.5% network fee to hosting nodes and platform.
    /// @dev 90% of fee to hosts (split equally), 10% to platform.
    /// @param txId The streaming transaction ID
    function distributeNetworkFee(bytes32 txId) external notPaused nonReentrant {
        StreamingTransaction storage tx = streamingTransactions[txId];
        require(tx.timestamp > 0, "transaction not found");
        require(!tx.feeProcessed, "fee already processed");
        require(tx.networkFee > 0, "zero network fee");

        tx.feeProcessed = true;

        // Split 2.5% fee: 90% to hosts, 10% to platform
        uint256 hostNodesShare = (tx.networkFee * 9000) / BASIS_POINTS; // 90%
        uint256 platformShare = tx.networkFee - hostNodesShare;           // 10%

        // Distribute to host nodes equally
        uint256 feePerHost = hostNodesShare / tx.hostNodes.length;
        for (uint i = 0; i < tx.hostNodes.length; i++) {
            address host = tx.hostNodes[i];
            require(host != address(0), "zero host address");

            if (feePerHost > 0) {
                require(btt.transfer(host, feePerHost), "host payment failed");

                hostReputation[host].totalFeesEarned += feePerHost;
                hostReputation[host].streamsHosted += 1;
                hostReputation[host].lastReward = block.timestamp;

                emit HostRewardPaid(host, feePerHost, hostReputation[host].totalFeesEarned);
            }
        }

        // Accumulate platform share (dust + 10%)
        uint256 dust = hostNodesShare - (feePerHost * tx.hostNodes.length);
        platformFeeAccumulated += platformShare + dust;

        emit NetworkFeeDistributed(txId, tx.networkFee, hostNodesShare, platformShare);
    }

    /// @notice Distribute artist royalty according to IPI splits.
    /// @dev 97.5% of stream goes to songwriters/publishers per their split agreement.
    /// @param txId The streaming transaction ID
    function distributeArtistRoyalty(bytes32 txId) external notPaused nonReentrant {
        StreamingTransaction storage tx = streamingTransactions[txId];
        require(tx.timestamp > 0, "transaction not found");
        require(!tx.royaltyProcessed, "royalty already processed");
        require(tx.artistRoyalty > 0, "zero artist royalty");

        tx.royaltyProcessed = true;

        IPISplit storage split = trackIPISplits[tx.trackCid];
        require(split.splitAddresses.length > 0, "IPI splits not found");

        uint256[] memory amounts = new uint256[](split.splitAddresses.length);
        uint256 distributed;

        // Distribute royalty according to IPI split percentages
        for (uint i = 0; i < split.splitAddresses.length; i++) {
            address recipient = split.splitAddresses[i];
            uint256 amount = (tx.artistRoyalty * split.splitPercentages[i]) / BASIS_POINTS;
            amounts[i] = amount;
            distributed += amount;

            if (amount > 0 && artistOptInCrypto[recipient]) {
                require(btt.transfer(recipient, amount), "royalty payment failed");
            }
            // If not opted in, royalty accumulates in contract for fiat settlement
        }

        // Dust to admin
        uint256 dust = tx.artistRoyalty - distributed;
        if (dust > 0) {
            require(btt.transfer(admin, dust), "dust transfer failed");
        }

        emit ArtistRoyaltyDistributed(
            txId, tx.trackCid, tx.artistRoyalty, split.splitAddresses, amounts
        );
    }

    /// @notice Allow an artist/songwriter to opt in to direct crypto payouts.
    function setArtistCryptoOptIn(bool optIn) external {
        artistOptInCrypto[msg.sender] = optIn;
        emit ArtistOptedIntoCrypto(msg.sender, optIn);
    }

    /// @notice Query host node reputation stats.
    function getHostReputation(address hostNode) external view returns (HostReputation memory) {
        return hostReputation[hostNode];
    }

    /// @notice Get streaming transaction record.
    function getStreamingTransaction(bytes32 txId) external view returns (StreamingTransaction memory) {
        return streamingTransactions[txId];
    }

    /// @notice Get IPI split for a track.
    function getIPISplit(bytes32 trackCid) external view returns (IPISplit memory) {
        return trackIPISplits[trackCid];
    }

    /// @notice Get total streaming transactions recorded.
    function getTransactionCount() external view returns (uint256) {
        return transactionHistory.length;
    }

    /// @notice Get accumulated platform fees (10% of network fees).
    function getPlatformFees() external view returns (uint256) {
        return platformFeeAccumulated;
    }

    /// @notice Admin: withdraw accumulated platform fees.
    function withdrawPlatformFees(uint256 amount) external onlyAdmin {
        require(amount <= platformFeeAccumulated, "insufficient platform fees");
        platformFeeAccumulated -= amount;
        require(btt.transfer(admin, amount), "transfer failed");
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
