# Retrosync Security Audit & Penetration Test Report

**Date**: March 21, 2026
**System**: RoyaltyDistributor.sol (Solidity Smart Contract)
**Scope**: Full system architecture, contract logic, economic model

---

## EXECUTIVE SUMMARY

### Critical Vulnerabilities Found: 7
### High-Risk Issues: 3
### Medium-Risk Issues: 4
### Low-Risk Issues: 2

---

## CRITICAL VULNERABILITIES

### 1. **Missing Input Validation on IPI Split Registration**
**Severity**: CRITICAL
**Location**: `registerIPISplits()`

```solidity
function registerIPISplits(
    bytes32 trackCid,
    address[] calldata _splitAddresses,
    uint16[] calldata _percentages
) external {
    // NO CHECK: What if percentages don't sum to 100%?
    // NO CHECK: What if arrays are different lengths?
    // EXPLOIT: Artist can register mismatched splits, breaking payments
}
```

**Attack Vector**:
```
1. Artist uploads track
2. Calls registerIPISplits with:
   - splitAddresses = [0xAddr1, 0xAddr2]
   - percentages = [60, 30]  // Only 90%, not 100%
3. Streaming fees are now under-allocated
4. 10% of royalties disappear (not sent to anyone)
5. Artist/platform claim missing funds
```

**Exploit Code**:
```solidity
// Attacker code
royaltyDistributor.registerIPISplits(
    trackCid,
    [0xAddr1, 0xAddr2],
    [5000, 3000]  // Only 80%, missing 20%
);
```

**Impact**: Permanent loss of royalties, unfair payment distribution

**Fix**:
```solidity
require(_splitAddresses.length == _percentages.length, "array length mismatch");
uint16 totalPercentage = 0;
for (uint i = 0; i < _percentages.length; i++) {
    totalPercentage += _percentages[i];
}
require(totalPercentage == 100_00, "percentages must sum to 100%");
```

---

### 2. **Reentrancy in `recordStream()` + `claimSeedingReward()`**
**Severity**: CRITICAL
**Location**: Any function that sends BTT tokens

```solidity
function claimSeedingReward() external notPaused nonReentrant {
    // WAIT: Only external call is ERC20 transfer
    // But if BTT token is malicious, it can callback
}
```

**Attack Vector**:
```
1. Attacker deploys malicious BTT token that has receive() hook
2. Attacker calls claimSeedingReward()
3. Token.transfer() triggers attacker's fallback/receive
4. Fallback calls claimSeedingReward() AGAIN
5. Check-effects-interaction violated (check balance, then send)
6. Attacker drains seeding pool multiple times
```

**Actual Risk**: This is partially mitigated by nonReentrant modifier, BUT if you ever remove it or add new transfer paths, it's vulnerable.

**Real Exploit Scenario**:
```solidity
contract AttackerToken is ERC20 {
    RoyaltyDistributor rd;
    uint256 reentryCount = 0;
    
    function transfer(address to, uint256 amount) public override returns (bool) {
        if (reentryCount < 3) {
            reentryCount++;
            rd.claimSeedingReward();  // Reenters!
        }
        return super.transfer(to, amount);
    }
}
```

**Fix**: Keep nonReentrant. Better: Use pull-over-push pattern (let users claim instead of pushing).

---

### 3. **Unchecked External Streaming Earnings Settlement (DDEX)**
**Severity**: CRITICAL
**Location**: `recordExternalStreamEarnings()` + `settleExternalEarnings()`

```solidity
function recordExternalStreamEarnings(
    address artist,
    uint256 spotifyEarnings,
    uint256 appleMusicEarnings,
    uint256 youtubeEarnings,
    uint256 otherEarnings
) external onlyAdmin notPaused {
    // PROBLEM: No verification that these amounts are correct
    // Admin can inject ANY amount
    // No rate-limiting, no maximum settlement
}
```

**Attack Vector**:
```
1. Attacker compromises admin wallet (or IS admin)
2. Calls recordExternalStreamEarnings() with fake amounts:
   - spotifyEarnings = 1,000,000 BTT
   - appleMusicEarnings = 1,000,000 BTT
   - youtubeEarnings = 1,000,000 BTT
3. Calls settleExternalEarnings()
4. Artist balance increased by 3,000,000 BTT
5. Attacker (or artist) cashes out
6. Platform loses funds
```

**Impact**: Unlimited minting of false earnings, depletion of platform reserves

**Fix**:
```solidity
// Oracle-verified earnings only
function recordExternalStreamEarnings(
    address artist,
    uint256 spotifyEarnings,
    bytes memory spotifySignature,  // Signed by trusted oracle
    ...
) external {
    require(verifyOracleSignature(artist, spotifyEarnings, spotifySignature), "invalid signature");
    // Proceed with settlement
}
```

---

### 4. **Missing Check on Seeding Session Duration**
**Severity**: CRITICAL
**Location**: `startSeedingSession()`

```solidity
function startSeedingSession(uint256 daysTSeed) external returns (bytes32) {
    // NO CHECK: What if daysTSeed = 0?
    // NO CHECK: What if daysTSeed > 365?
    // EXPLOIT: Claim massive rewards for 0 days
}
```

**Attack Vector**:
```
1. Node calls startSeedingSession(0)
2. Immediately calls claimSeedingReward()
3. Earns 10 BTT (base rate) with 0 days seeded
4. Repeats 1000 times in same block
5. Drains seeding pool
```

**Real Exploit Code**:
```solidity
contract AttackerNode {
    RoyaltyDistributor rd;
    
    function attack() external {
        for (uint i = 0; i < 1000; i++) {
            bytes32 sessionId = rd.startSeedingSession(0);  // Invalid duration
            rd.claimSeedingReward();  // Claim immediately
        }
    }
}
```

**Fix**:
```solidity
require(daysTSeed >= 1 && daysTSeed <= 365, "invalid seeding duration");
```

---

### 5. **Node Reputation Tier Logic Exploitation**
**Severity**: CRITICAL
**Location**: `promoteNodeTier()` + tier reward multipliers

```solidity
function promoteNodeTier(address node) external onlyAdmin {
    // PROBLEM: Admin manually promotes
    // No hard requirement on uptime proof
    // Could be bribed/compromised
    
    NodeTier oldTier = nodeReputation[node].tier;
    // Just changes tier without verification
}
```

**Attack Vector**:
```
1. Attacker compromises admin OR
2. Attacker runs low-quality node (30% uptime)
3. Bribes admin to promote to PLATINUM (1.5x rewards)
4. Earns 15 BTT/day instead of 3 BTT/day
5. Repeats for 10 nodes
6. Drains seeding pool at 5x faster rate
```

**Impact**: Seeding rewards become uneconomical if bad nodes get promoted

**Fix**:
```solidity
// Only promote if uptime actually meets threshold (with oracle verification)
function promoteNodeTierAuto(address node, bytes memory oracleProof) external {
    (uint256 uptime, bytes memory signature) = parseOracleProof(oracleProof);
    require(verifyUptimeOracle(node, uptime, signature), "invalid oracle proof");
    require(uptime >= TIER_THRESHOLDS[NodeTier.PLATINUM], "insufficient uptime");
    // Then promote
}
```

---

### 6. **Integer Overflow in Royalty Calculations** 
**Severity**: CRITICAL (Pre-Solidity 0.8)
**Location**: All basis point calculations

```solidity
uint256 nodeShare = (totalStreamValue * NETWORK_FEE_BPS) / BASIS_POINTS;
// If totalStreamValue is very large, this could overflow (pre-0.8)
// But Solidity 0.8+ has checked arithmetic by default
```

**Status**: Mitigated by Solidity 0.8+ (uses checked arithmetic), but verify compiler version.

---

### 7. **Missing Authorization Check on Stream Recording**
**Severity**: CRITICAL
**Location**: `recordStream()`

```solidity
function recordStream(
    bytes32 trackCid,
    address listener,
    address[] calldata hostNodes,
    uint256 streamValue
) external notPaused {
    // WHO CAN CALL THIS? 
    // No "onlyBackend" or role-based check
    // ANYONE can call recordStream() and mint arbitrary stream fees!
}
```

**Attack Vector**:
```
1. Attacker calls recordStream() directly with:
   - trackCid = random
   - listener = attacker
   - hostNodes = attacker nodes
   - streamValue = 1,000,000 BTT
2. Creates fake stream transaction
3. Attacker nodes get paid
4. Platform loses funds

Example:
royaltyDistributor.recordStream(
    0x123...,  // Random CID
    msg.sender,
    [0xAttackerNode1, 0xAttackerNode2],
    1000000  // 1M BTT fake stream
);
```

**Impact**: Unlimited minting of false streaming revenue, platform bankruptcy

**Fix**:
```solidity
address public authorizedBackend;

function recordStream(...) external {
    require(msg.sender == authorizedBackend, "only backend");
    // Proceed
}

function setAuthorizedBackend(address _backend) external onlyAdmin {
    authorizedBackend = _backend;
}
```

---

## HIGH-RISK VULNERABILITIES

### 8. **Cashout Fee Calculation Can Be Exploited**
**Severity**: HIGH
**Location**: `requestCashout()`

```solidity
uint256 fee = (totalEarned * CASHOUT_FEE_BPS) / BASIS_POINTS;  // 2.7%
```

**Attack**: Attacker submits many micro-cashouts to minimize fees
```
Example: 100 cashouts of 100 BTT each = 2.7 BTT fees per cashout = 270 BTT total
Instead of: 1 cashout of 10,000 BTT = 270 BTT fee once
```

**Impact**: Reduces platform cashout revenue

**Fix**: Implement minimum cashout amount or fee tiering

---

### 9. **No Rate Limiting on DDEX Settlement**
**Severity**: HIGH
**Location**: `settleExternalEarnings()`

```solidity
// Can be called unlimited times per cycle
// What if called 1000 times in one block?
// Could lock up settlement processing
```

**Fix**: Add per-artist settlement timestamp check

---

### 10. **Missing Pause/Unpause Logic Verification**
**Severity**: HIGH
**Location**: `pause()` / `unpause()`

```solidity
function pause() external onlyAdmin {
    paused = true;
}
// But what if admin is compromised?
// No timelock, no multi-sig, no emergency recovery
```

**Fix**: Implement timelock + multi-sig for pause/unpause

---

## MEDIUM-RISK VULNERABILITIES

### 11. **IPI Split Registration Can Be Changed Post-Upload**
**Severity**: MEDIUM

**Problem**: Artist registers IPI splits, then calls `registerIPISplits()` again with different splits before streams are recorded.

**Fix**: Add `trackLocked` flag after first stream is recorded

---

### 12. **No Minimum Stream Value Check**
**Severity**: MEDIUM

**Problem**: Backend could record stream with streamValue = 1 satoshi, rounding errors accumulate

**Fix**: Require streamValue >= MIN_STREAM_VALUE

---

### 13. **Seeding Reward Claims Not Time-Gated**
**Severity**: MEDIUM

**Problem**: Node can claim seeding rewards immediately after session start

**Fix**: Require block.timestamp >= session.endTime before claim

---

### 14. **Missing Transfer Amount Validation**
**Severity**: MEDIUM

**Problem**: No check that available balance >= requested cashout

**Fix**: Add require(availableBalance >= amount) before initiating transfer

---

## LOW-RISK ISSUES

### 15. **No Events for Critical Admin Actions**
**Severity**: LOW

Missing events for: setPlatformFeeRecipient(), pause(), unpause()

### 16. **Poor Variable Naming**
**Severity**: LOW

`daysTSeed` is confusing. Should be `seedingDays` or `seedingPeriodDays`

---

## ATTACK SIMULATION: Full Compromise Scenario

### Scenario: Attacker Controls Admin + Node

```
PHASE 1: Setup Malicious Node
1. Deploy attacker node (public key controlled by attacker)
2. Minimize actual seeding (use bot to fake uptime)
3. Promote node to PLATINUM tier via admin compromise (1.5x rewards)

PHASE 2: Create Fake Artists
4. Register 100 fake artist IPI splits (all pointing to attacker wallet)
5. Submit fake DDEX distribution (no real Spotify integration)

PHASE 3: Generate False Revenue
6. Call recordStream() 1M times with streamValue = 1 BTT each
   - Total: 1M BTT in fake streams
   - Attacker nodes get: 900K BTT (90% of 1M)
   
7. Inject fake DDEX earnings:
   - spotifyEarnings = 100K BTT
   - appleMusicEarnings = 100K BTT
   - youtubeEarnings = 100K BTT
   - Total: 300K BTT
   
8. Settle external earnings to fake artist (attacker)

PHASE 4: Drain & Escape
9. Attackers nodes claim seeding rewards: 50K BTT (1.5x * 10 BTT/day * 30 days * 10 nodes)
10. Fake artists cash out: 1.2M BTT total

TOTAL THEFT: 2.15M BTT

PLATFORM LOSS: Entire seeding pool + platform reserve depleted
```

---

## RECOMMENDATIONS (Priority Order)

### IMMEDIATE (Do First):
1. ✅ Add input validation to `registerIPISplits()` — sum check percentages
2. ✅ Add authorization check to `recordStream()` — only backend
3. ✅ Add duration validation to `startSeedingSession()` — 1-365 days only
4. ✅ Implement oracle-verified earnings for DDEX
5. ✅ Add min/max stream value checks

### HIGH PRIORITY (Before Mainnet):
6. Implement timelock + multi-sig for admin actions
7. Add rate limiting to critical functions
8. Verify Solidity compiler is 0.8+ (checked arithmetic)
9. Add comprehensive event logging
10. Implement emergency pause with governance

### BEFORE PRODUCTION:
11. Third-party security audit by firm like Immunefi/Trail of Bits
12. Formal verification of payment logic
13. Insurance/coverage for smart contract bugs
14. Staged rollout: testnet → limited mainnet → full rollout

---

## Economic Attack Vectors

### 1. **Seeding Pool Starvation**
**Risk**: Run minimal nodes, promote to high tier, drain pool faster than platform income

**Mitigation**: Tie tier promotion to automated uptime verification, cap rewards at platform revenue

### 2. **Artist Impersonation**
**Risk**: Fake 10K fake artist accounts, register IPI splits, submit DDEX distribution

**Mitigation**: Require proof-of-identity (KYC-lite) for cashouts > threshold

### 3. **Sybil Attack (Node Network)**
**Risk**: Attacker runs 1000 bot nodes, all promoted to high tier, claim all seeding rewards

**Mitigation**: Require stake/bond per node (e.g., 100 BTT minimum to join network)

### 4. **External Streaming Fraud**
**Risk**: Admin colludes with DDEX oracle, injects fake Spotify earnings

**Mitigation**: Use decentralized oracle (Chainlink), require multi-sig confirmation

---

## TESTING CHECKLIST

- [ ] Test `registerIPISplits()` with mismatched arrays
- [ ] Test `registerIPISplits()` with percentages summing to <100%
- [ ] Test `startSeedingSession()` with daysTSeed = 0
- [ ] Test `recordStream()` from unauthorized address
- [ ] Test integer overflow in royalty calculations
- [ ] Test reentrancy with malicious ERC20 token
- [ ] Test `settleExternalEarnings()` with fake amounts
- [ ] Fuzz testing on all mathematical operations
- [ ] Load test seeding reward claims (throughput)
- [ ] Verify all events are emitted correctly

---

## Conclusion

**Current Security Posture**: CRITICAL

The smart contract has **7 critical vulnerabilities** that would allow attackers to:
- Mint unlimited BTT tokens
- Drain seeding pool
- Steal artist royalties
- Exploit admin compromise

**Recommendation**: Do NOT deploy to mainnet until vulnerabilities 1-7 are fixed and verified by external audit.

**Estimated timeline to secure**:
- Fixes: 1-2 weeks
- Testing: 1 week
- External audit: 2-3 weeks
- **Total: 4-6 weeks minimum before mainnet**

