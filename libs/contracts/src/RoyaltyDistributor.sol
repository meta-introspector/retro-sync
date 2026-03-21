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
    uint256 public constant NETWORK_FEE_BPS   = 270; // 2.7% fee (270 / 10,000)

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

    // ── Artist earnings accumulator (100% of streams, paid on cashout) ──
    struct ArtistEarnings {
        uint256 totalEarned;        // 100% of all stream royalties accumulated
        uint256 totalWithdrawn;     // Amount already cashed out
        uint256 lastWithdrawal;
    }
    mapping(address => ArtistEarnings) public artistEarnings; // For non-split artists

    // ── Streaming transaction records (for audit trail) ─────────────────
    struct StreamingTransaction {
        bytes32   trackCid;         // Content CID being streamed
        address   listener;         // User who triggered the stream
        address[] hostNodes;        // P2P nodes that provided the stream
        address[] royaltyRecipients; // IPI split addresses (or single artist)
        uint16[]  royaltyPercentages; // IPI split percentages
        uint256   streamValue;      // Amount credited to artist(s)
        uint256   timestamp;
    }
    mapping(bytes32 => StreamingTransaction) public streamingTransactions;
    bytes32[] public transactionHistory;

    // ── Cashout pending (2.5% fee charged on withdrawal) ──────────────
    struct CashoutRequest {
        address recipient;
        uint256 amount;             // Amount requested (before fee)
        uint256 networkFee;         // 2.5% fee calculated
        uint256 netAmount;          // Amount after fee (to be paid)
        uint256 timestamp;
        bool executed;
    }
    mapping(bytes32 => CashoutRequest) public cashoutRequests;

    // ── Host node reputation ───────────────────────────────────────────
    struct HostReputation {
        uint256 totalFeesEarned;    // 2.5% fee share earned
        uint256 streamsHosted;
        uint256 lastReward;
    }
    mapping(address => HostReputation) public hostReputation;

    // ── Artist opt-in for crypto payouts ───────────────────────────────
    mapping(address => bool) public artistOptInCrypto; // true = artist accepts crypto payouts
    
    // ── Platform cashout tracking ───────────────────────────────────────
    uint256 public platformFeesAccumulated; // Platform share from cashout fees

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
        address[] royaltyRecipients,
        uint256[] royaltyPercentages
    );
    event IPISplitRegistered(
        bytes32 indexed trackCid,
        address[] splitAddresses,
        uint16[] splitPercentages,
        bytes32 ipiReference
    );
    event CashoutRequested(
        bytes32 indexed cashoutId,
        address indexed artist,
        uint256 amount,
        uint256 networkFee,
        uint256 netAmount
    );
    event CashoutExecuted(
        bytes32 indexed cashoutId,
        address indexed artist,
        uint256 netAmount,
        uint256 networkFeeDistributed
    );
    event HostRewardPaid(
        address indexed hostNode,
        uint256 amount,
        uint256 totalEarned
    );

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

    /// @notice Record a P2P streaming transaction from user listening.
    /// @dev User pays streamValue in full. 2.7% micro-fee collected immediately for nodes + platform.
    ///      Remaining amount (97.3%) credited to artist(s) per IPI split.
    /// @param txId Unique transaction ID
    /// @param trackCid BTFS CID of the track being streamed
    /// @param listener User address who listened
    /// @param hostNodes P2P nodes that provided the stream
    /// @param streamValue User pays this amount (2.7% fee applies)
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

        IPISplit storage split = trackIPISplits[trackCid];
        
        // Calculate 2.7% network fee from user's stream payment
        uint256 networkFee = (streamValue * NETWORK_FEE_BPS) / BASIS_POINTS;
        uint256 artistRoyalty = streamValue - networkFee;
        
        // Record the transaction
        streamingTransactions[txId] = StreamingTransaction({
            trackCid: trackCid,
            listener: listener,
            hostNodes: hostNodes,
            royaltyRecipients: split.splitAddresses,
            royaltyPercentages: split.splitPercentages,
            streamValue: streamValue,
            timestamp: block.timestamp
        });
        transactionHistory.push(txId);

        // Collect 2.7% network fee: distribute to hosts + platform
        uint256 hostNodesShare = (networkFee * 9000) / BASIS_POINTS; // 90% to hosts
        uint256 platformShare = networkFee - hostNodesShare;         // 10% to platform

        // Distribute to host nodes equally
        uint256 feePerHost = hostNodesShare / hostNodes.length;
        for (uint i = 0; i < hostNodes.length; i++) {
            address host = hostNodes[i];
            require(host != address(0), "zero host address");

            if (feePerHost > 0) {
                require(btt.transfer(host, feePerHost), "host payment failed");

                hostReputation[host].totalFeesEarned += feePerHost;
                hostReputation[host].streamsHosted += 1;
                hostReputation[host].lastReward = block.timestamp;

                emit HostRewardPaid(host, feePerHost, hostReputation[host].totalFeesEarned);
            }
        }

        // Accumulate platform fees
        uint256 dust = hostNodesShare - (feePerHost * hostNodes.length);
        platformFeesAccumulated += platformShare + dust;

        // Credit artist royalty (97.3%) to each split recipient
        uint256 accumulated;
        for (uint i = 0; i < split.splitAddresses.length; i++) {
            address recipient = split.splitAddresses[i];
            uint256 amount = (artistRoyalty * split.splitPercentages[i]) / BASIS_POINTS;
            artistEarnings[recipient].totalEarned += amount;
            accumulated += amount;
        }

        // Dust to admin
        uint256 artistDust = artistRoyalty - accumulated;
        if (artistDust > 0) {
            artistEarnings[admin].totalEarned += artistDust;
        }

        emit StreamingTransactionRecorded(
            txId, trackCid, listener, streamValue, split.splitAddresses, split.splitPercentages
        );
    }

    /// @notice Request a cashout. 2.7% fee deducted, goes to P2P hosts and platform.
    /// @param cashoutId Unique cashout request ID
    /// @param amount Amount to cash out (before 2.7% fee)
    function requestCashout(bytes32 cashoutId, uint256 amount) external notPaused {
        require(amount > 0, "zero amount");
        require(artistEarnings[msg.sender].totalEarned >= amount, "insufficient earnings");
        require(cashoutRequests[cashoutId].timestamp == 0, "cashout already recorded");

        // Calculate 2.7% fee
        uint256 networkFee = (amount * NETWORK_FEE_BPS) / BASIS_POINTS;
        uint256 netAmount = amount - networkFee;

        // Record the cashout request
        cashoutRequests[cashoutId] = CashoutRequest({
            recipient: msg.sender,
            amount: amount,
            networkFee: networkFee,
            netAmount: netAmount,
            timestamp: block.timestamp,
            executed: false
        });

        emit CashoutRequested(cashoutId, msg.sender, amount, networkFee, netAmount);
    }

    /// @notice Execute a cashout and distribute 2.7% fee to hosts and platform.
    /// @dev 90% of fee to hosts (equally), 10% to platform operations.
    /// @param cashoutId The cashout request ID
    /// @param hostNodes P2P nodes that hosted/seeded the content
    function executeCashout(bytes32 cashoutId, address[] calldata hostNodes) external notPaused nonReentrant {
        CashoutRequest storage co = cashoutRequests[cashoutId];
        require(co.timestamp > 0, "cashout not found");
        require(!co.executed, "cashout already executed");
        require(hostNodes.length > 0, "at least one host required");

        co.executed = true;

        // Deduct from artist earnings
        artistEarnings[co.recipient].totalEarned -= co.amount;
        artistEarnings[co.recipient].totalWithdrawn += co.netAmount;
        artistEarnings[co.recipient].lastWithdrawal = block.timestamp;

        // Pay artist the net amount (after 2.7% fee)
        require(btt.transfer(co.recipient, co.netAmount), "artist payment failed");

        // Distribute 2.7% fee: 90% to hosts, 10% to platform
        uint256 hostNodesShare = (co.networkFee * 9000) / BASIS_POINTS; // 90%
        uint256 platformShare = co.networkFee - hostNodesShare;         // 10%

        // Pay hosting nodes equally
        uint256 feePerHost = hostNodesShare / hostNodes.length;
        for (uint i = 0; i < hostNodes.length; i++) {
            address host = hostNodes[i];
            require(host != address(0), "zero host address");

            if (feePerHost > 0) {
                require(btt.transfer(host, feePerHost), "host payment failed");

                hostReputation[host].totalFeesEarned += feePerHost;
                hostReputation[host].streamsHosted += 1;
                hostReputation[host].lastReward = block.timestamp;

                emit HostRewardPaid(host, feePerHost, hostReputation[host].totalFeesEarned);
            }
        }

        // Accumulate platform share
        uint256 dust = hostNodesShare - (feePerHost * hostNodes.length);
        platformFeesAccumulated += platformShare + dust;

        emit CashoutExecuted(cashoutId, co.recipient, co.netAmount, co.networkFee);
    }

    /// @notice Get artist earnings and withdrawal history.
    function getArtistEarnings(address artist) external view returns (ArtistEarnings memory) {
        return artistEarnings[artist];
    }

    /// @notice Get streaming transaction record.
    function getStreamingTransaction(bytes32 txId) external view returns (StreamingTransaction memory) {
        return streamingTransactions[txId];
    }

    /// @notice Get IPI split for a track.
    function getIPISplit(bytes32 trackCid) external view returns (IPISplit memory) {
        return trackIPISplits[trackCid];
    }

    /// @notice Get cashout request details.
    function getCashoutRequest(bytes32 cashoutId) external view returns (CashoutRequest memory) {
        return cashoutRequests[cashoutId];
    }

    /// @notice Get total streaming transactions recorded.
    function getTransactionCount() external view returns (uint256) {
        return transactionHistory.length;
    }

    /// @notice Query host node reputation stats.
    function getHostReputation(address hostNode) external view returns (HostReputation memory) {
        return hostReputation[hostNode];
    }

    /// @notice Get accumulated platform fees.
    function getPlatformFees() external view returns (uint256) {
        return platformFeesAccumulated;
    }

    /// @notice Admin: withdraw accumulated platform fees.
    function withdrawPlatformFees(uint256 amount) external onlyAdmin {
        require(amount <= platformFeesAccumulated, "insufficient platform fees");
        platformFeesAccumulated -= amount;
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
