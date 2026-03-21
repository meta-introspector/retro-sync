// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/RoyaltyDistributor.sol";
import "../src/MockBTT.sol";
import "../src/ZKVerifier.sol";

contract SandboxTest is Test {
    RoyaltyDistributor public distributor;
    MockBTT public token;
    ZKVerifier public verifier;

    function setUp() public {
        token = new MockBTT(1_000_000); // 1M tokens
        verifier = new ZKVerifier();
        distributor = new RoyaltyDistributor(address(token), address(verifier));
    }

    function testSandbox() public {
        console2.log("--- Foundry Sandbox Environment ---");
        console2.log("Experiment with contract interactions here.");
        
        uint256 amount = 1000 * 10**18;
        uint256 initialBalance = token.balanceOf(address(this));
        token.mint(address(this), amount);
        
        assertEq(token.balanceOf(address(this)), initialBalance + amount);
        console2.log("Minted extra tokens for testing:", amount / 1e18);
        console2.log("Total balance:", token.balanceOf(address(this)) / 1e18);
    }
}
