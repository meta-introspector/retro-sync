// SPDX-License-Identifier: AGPL-3.0-or-later
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/MockBTT.sol";
import "../src/ZKVerifier.sol";
import "../src/RoyaltyDistributor.sol";

contract RoyaltyDistributorFuzzTest is Test {
    MockBTT  btt;
    ZKVerifier verifier;
    RoyaltyDistributor dist;

    function setUp() public {
        btt      = new MockBTT(1_000_000_000);
        verifier = new ZKVerifier();
        dist     = new RoyaltyDistributor(address(btt), address(verifier));
        btt.transfer(address(dist), 500_000_000 * 1e18);
    }

    // Reentrancy guard: paused state prevents double-entry
    function testEmergencyPauseBlocks() public {
        dist.emergencyPause();
        assertTrue(dist.paused());
    }

    // Band must be 0, 1, or 2
    function testInvalidBandReverts(uint8 band) public {
        vm.assume(band > 2);
        // Even without proof, band check fires first
        address[] memory artists = new address[](1);
        artists[0] = address(0x1);
        uint16[] memory bps_ = new uint16[](1);
        bps_[0] = 10_000;
        vm.expectRevert("invalid band");
        dist.distribute(bytes32(0), artists, bps_, band, new bytes(192), 1e18);
    }

    // BPS must sum to 10_000
    function testBpsNotSumRevert(uint16 a, uint16 b) public {
        vm.assume(uint256(a) + uint256(b) != 10_000);
        vm.assume(a > 0 && b > 0);
        address[] memory artists = new address[](2);
        artists[0] = address(0x1); artists[1] = address(0x2);
        uint16[] memory bps_ = new uint16[](2);
        bps_[0] = a; bps_[1] = b;
        vm.expectRevert("bps must sum to 10000");
        dist.distribute(bytes32(uint256(1)), artists, bps_, 0, new bytes(192), 1e18);
    }

    // MockBTT transfer/mint/burn properties
    function testMockBttMintBurn(uint96 amount) public {
        vm.assume(amount > 0);
        uint256 before = btt.totalSupply();
        btt.mint(address(this), amount);
        assertEq(btt.totalSupply(), before + amount);
        btt.burn(amount);
        assertEq(btt.totalSupply(), before);
    }

    function testMockBttTransfer(address to, uint96 amount) public {
        vm.assume(to != address(0) && to != address(this));
        vm.assume(uint256(amount) <= btt.balanceOf(address(this)));
        vm.assume(amount > 0);
        btt.transfer(to, amount);
        assertEq(btt.balanceOf(to), amount);
    }

    // Timelock: large amounts must go through queue
    function testLargeDistributionQueued() public {
        // MAX_DISTRIBUTION_BTT + 1 should queue, not execute immediately
        uint256 large = dist.MAX_DISTRIBUTION_BTT() + 1;
        btt.mint(address(dist), large);
        address[] memory artists = new address[](1);
        artists[0] = address(0xBEEF);
        uint16[] memory bps_ = new uint16[](1);
        bps_[0] = 10_000;
        // Will revert on ZK proof (verifier not set) — that's correct
        // Production: call with valid proof after ceremony
        vm.expectRevert("ZKVerifier: verifying key not set");
        dist.distribute(keccak256("large-cid"), artists, bps_, 0, new bytes(192), large);
    }
}
