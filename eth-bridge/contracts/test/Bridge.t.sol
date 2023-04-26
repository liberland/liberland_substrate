// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "../src/WrappedToken.sol";
import "../src/Bridge.sol";

using stdStorage for StdStorage;

contract BridgeTest is Test, BridgeEvents {
    WrappedToken public token;
    Bridge public bridge;

    address alice = vm.addr(1);
    address bob = vm.addr(2);
    address charlie = vm.addr(3);
    address dave = vm.addr(4);

    bytes32 substrate1 = "12345678901234567890123456789012";
    bytes32 receipt1 = "12345678901234567890123456789012";
    bytes32 receipt2 = "22345678901234567890123456789012";
    bytes32 receipt3 = "32345678901234567890123456789012";

    event Transfer(address indexed from, address indexed to, uint256 value);

    function setUp() public {
        Bridge impl = new Bridge();
        token = new WrappedToken("Liberland Merits", "LLM");
        bridge = Bridge(
            address(
                new ERC1967Proxy(
                address(impl),
                abi.encodeCall(
                Bridge.initialize,
                (
                    token,
                    2,
                    10,
                    4,
                    1000,
                    10,
                    650
                )
                )
                )
            )
        );
        bridge.grantRole(bridge.ADMIN_ROLE(), dave);
        bridge.grantRole(bridge.RELAY_ROLE(), alice);
        bridge.grantRole(bridge.RELAY_ROLE(), bob);
        bridge.grantRole(bridge.RELAY_ROLE(), charlie);
        bridge.grantRole(bridge.WATCHER_ROLE(), alice);
        bridge.grantRole(bridge.WATCHER_ROLE(), dave);
        vm.deal(alice, 100);
        token.mint(alice, 100);
        token.mint(bob, 100);
        token.mint(charlie, 100);
        token.mint(dave, 100);
        token.mint(address(this), 100);
        token.transferOwnership(address(bridge));
        token.approve(address(bridge), 9999999);

        vm.prank(dave);
        bridge.setActive(true);
    }

    function testBurnRevertsOnStoppedBridge() public {
        vm.prank(dave);
        bridge.setActive(false);

        vm.expectRevert(BridgeInactive.selector);
        bridge.burn(100, substrate1);
    }

    function testBurnEmitsReceipt() public {
        vm.expectEmit(false, false, false, false);
        emit OutgoingReceipt(100, substrate1);
        bridge.burn(100, substrate1);
    }

    function testBurnTakesTokenFromCaller() public {
        bridge.burn(10, substrate1);
        assertEq(token.balanceOf(address(this)), 90);
        bridge.burn(10, substrate1);
        assertEq(token.balanceOf(address(this)), 80);
        bridge.burn(80, substrate1);
        assertEq(token.balanceOf(address(this)), 0);
    }

    function testBurnReducesTotalSupply() public {
        bridge.burn(10, substrate1);
        assertEq(token.totalSupply(), 490);
        bridge.burn(10, substrate1);
        assertEq(token.totalSupply(), 480);
        bridge.burn(80, substrate1);
        assertEq(token.totalSupply(), 400);
    }

    function testBurnFailsOnInsufficientFunds() public {
        vm.expectRevert(bytes("ERC20: burn amount exceeds balance"));
        bridge.burn(101, substrate1);
    }

    function testVoteFailsOnNonRelay() public {
        vm.expectRevert(
            "AccessControl: account 0x7fa9385be102ac3eac297483dd6233d62b3e1496 is missing role 0x077a1d526a4ce8a773632ab13b4fbbf1fcc954c3dab26cd27ea0e2a6750da5d7"
        );
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testVoteFailsOnStoppedBridge() public {
        vm.prank(dave);
        bridge.setActive(false);
        vm.prank(alice);
        vm.expectRevert(BridgeInactive.selector);
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testVoteSuccedsAfterReachingRequiredVotes() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(charlie);
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testVoteFailsOnProcessedReceipt() public {
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.roll(11);
        bridge.mint{value: 4}(receipt1);

        vm.stopPrank();
        vm.prank(charlie);
        vm.expectRevert(AlreadyProcessed.selector);
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testVoteSetsReceiptDetails() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);

        (uint64 substrateBlockNumber, address ethRecipient, uint256 amount,,) = bridge.incomingReceipts(receipt1);
        assertEq(substrateBlockNumber, 1);
        assertEq(ethRecipient, alice);
        assertEq(amount, 100);
    }

    function testVoteStopsBridgeOnMismatchedDetails() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.prank(bob);
        vm.expectEmit(false, false, false, true);
        emit StateChanged(false);
        bridge.voteMint(receipt1, 1, 101, alice);

        vm.prank(alice);
        vm.expectRevert(BridgeInactive.selector);
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testVotingRevertsOnDoubleVote() public {
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.expectRevert(AlreadyVoted.selector);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.stopPrank();

        vm.expectRevert(NotApproved.selector);
        bridge.mint(receipt1);

        vm.prank(bob);
        vm.expectEmit(true, false, false, false);
        emit Approved(receipt1);
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testVotingEmitsEvent() public {
        vm.prank(alice);
        vm.expectEmit(false, false, false, true);
        emit Vote(receipt1, alice);
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testVotingSetsApprovedOn() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);

        (,,, uint256 approvedOn,) = bridge.incomingReceipts(receipt1);
        assertEq(approvedOn, block.number);
    }

    function testMintFailsOnUnknownReceipt() public {
        vm.expectRevert(NotApproved.selector);
        bridge.mint(receipt1);
    }

    function testMintFailsOnInsufficientVotes() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.expectRevert(NotApproved.selector);
        bridge.mint(receipt1);
    }

    function testMintFailsOnStoppedBridge() public {
        vm.prank(dave);
        bridge.setActive(false);

        vm.expectRevert(BridgeInactive.selector);
        bridge.mint(receipt1);
    }

    function testMintEmitsEvent() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(11);
        vm.expectEmit(true, true, false, true);
        emit Transfer(address(0), alice, 100);
        bridge.mint{value: 4}(receipt1);
    }

    function testMintIncreasesTotalSupply() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(11);
        bridge.mint{value: 4}(receipt1);

        assertEq(token.totalSupply(), 600);
    }

    function testMintSendsTokensToRecipient() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(11);
        bridge.mint{value: 4}(receipt1);

        assertEq(token.balanceOf(alice), 200);
    }

    function testMintRespectsMaxIssuanceLimit() public {
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 149, alice);
        bridge.voteMint(receipt2, 1, 2, alice);
        bridge.voteMint(receipt3, 1, 1, alice);
        vm.stopPrank();
        vm.startPrank(bob);
        bridge.voteMint(receipt1, 1, 149, alice);
        bridge.voteMint(receipt2, 1, 2, alice);
        bridge.voteMint(receipt3, 1, 1, alice);
        vm.stopPrank();

        vm.roll(11);
        bridge.mint{value: 4}(receipt1);

        vm.expectRevert(TooMuchSupply.selector);
        bridge.mint{value: 4}(receipt2);

        bridge.mint{value: 4}(receipt3);
    }

    function testMintFailsOnInsufficientEther() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(11);
        vm.expectRevert(InsufficientEther.selector);
        bridge.mint{value: 3}(receipt1);
    }

    function testMintDistributesRewardsToVotingRelays1() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(11);
        bridge.mint{value: 16}(receipt1);

        assertEq(bridge.pendingRewards(alice), 11);
        assertEq(bridge.pendingRewards(bob), 5);
        assertEq(bridge.pendingRewards(charlie), 0);
    }

    function testMintDistributesRewardsToVotingRelays2() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(charlie);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(11);
        bridge.mint{value: 19}(receipt1);

        assertEq(bridge.pendingRewards(alice), 11);
        assertEq(bridge.pendingRewards(bob), 5);
        assertEq(bridge.pendingRewards(charlie), 3);
    }

    function testMintUpdatesReceiptStatus() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(11);
        bridge.mint{value: 4}(receipt1);

        (,,,, uint256 processedOn) = bridge.incomingReceipts(receipt1);
        assertEq(processedOn, block.number);
    }

    function testMintWorksOnlyOnce() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.roll(11);
        bridge.mint{value: 4}(receipt1);

        vm.expectRevert(AlreadyProcessed.selector);
        bridge.mint{value: 4}(receipt1);
    }

    function testMintRespectsMintDelay() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(10);
        vm.expectRevert(TooSoon.selector);
        bridge.mint{value: 4}(receipt1);

        vm.roll(11);
        bridge.mint{value: 4}(receipt1);
    }

    function testMintRespectsMintDelayWithVotesAfterApproval() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, alice);
        vm.prank(charlie);
        vm.roll(5);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(10);
        vm.expectRevert(TooSoon.selector);
        bridge.mint{value: 4}(receipt1);

        vm.roll(11);
        bridge.mint{value: 4}(receipt1);
    }

    function testEmergencyStopChecksPerms() public {
        vm.expectRevert(
            "AccessControl: account 0x7fa9385be102ac3eac297483dd6233d62b3e1496 is missing role 0x2125d1e225cadc5c8296e2cc1f96ee607770bf4a4a16131e62f6819937437c89"
        );
        bridge.emergencyStop();

        vm.prank(dave);
        bridge.emergencyStop();
    }

    function testEmergencyStopEmitsEvent() public {
        vm.prank(dave);
        vm.expectEmit(false, false, false, false);
        emit EmergencyStop();
        bridge.emergencyStop();
    }

    function testEmergencyStopWorks() public {
        vm.prank(dave);
        bridge.emergencyStop();

        vm.prank(alice);
        vm.expectRevert(BridgeInactive.selector);
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testSetFeeWorks() public {
        vm.startPrank(dave);
        bridge.setFee(1);
        assertEq(bridge.fee(), 1);
        bridge.setFee(10);
        assertEq(bridge.fee(), 10);
    }

    function testSetFeeRequiresAdmin() public {
        vm.prank(alice);
        vm.expectRevert(
            "AccessControl: account 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf is missing role 0xa49807205ce4d355092ef5a8a18f56e8913cf4a201fbe287825b095693c21775"
        );
        bridge.setFee(1);
    }

    function testSetVotesRequiredWorks() public {
        bridge.setVotesRequired(1);
        assertEq(bridge.votesRequired(), 1);
        bridge.setVotesRequired(10);
        assertEq(bridge.votesRequired(), 10);
    }

    function testSetVotesRequiredRequiresSuperAdmin() public {
        vm.prank(alice);
        vm.expectRevert(
            "AccessControl: account 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf is missing role 0x7613a25ecc738585a232ad50a301178f12b3ba8887d13e138b523c4269c47689"
        );
        bridge.setVotesRequired(1);
    }

    function testRemovingRelayRequiresAdmin() public {
        bytes32 relay = bridge.RELAY_ROLE();
        vm.prank(alice);
        vm.expectRevert(Unauthorized.selector);
        bridge.revokeRole(relay, alice);

        bridge.revokeRole(relay, alice);
    }

    function testRemovingRelayWorks() public {
        bytes32 relay = bridge.RELAY_ROLE();

        assertEq(bridge.hasRole(relay, alice), true);
        bridge.revokeRole(relay, alice);
        assertEq(bridge.hasRole(relay, alice), false);

        vm.prank(alice);
        vm.expectRevert(
            "AccessControl: account 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf is missing role 0x077a1d526a4ce8a773632ab13b4fbbf1fcc954c3dab26cd27ea0e2a6750da5d7"
        );
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testAddingWatcherRequiresAdmin() public {
        bytes32 watcher = bridge.WATCHER_ROLE();

        vm.prank(alice);
        vm.expectRevert(Unauthorized.selector);
        bridge.grantRole(watcher, alice);

        vm.prank(dave);
        bridge.grantRole(watcher, alice);

        bridge.grantRole(watcher, bob);
    }

    function testAddingWatcherWorks() public {
        bytes32 watcher = bridge.WATCHER_ROLE();

        assertEq(bridge.hasRole(watcher, bob), false);
        bridge.grantRole(watcher, bob);
        assertEq(bridge.hasRole(watcher, bob), true);

        vm.prank(bob);
        bridge.emergencyStop();
    }

    function testSetActiveRequiresAdmin() public {
        vm.startPrank(dave);
        bridge.setActive(true);
        bridge.setActive(false);
        vm.stopPrank();

        vm.startPrank(alice);
        vm.expectRevert(
            "AccessControl: account 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf is missing role 0xa49807205ce4d355092ef5a8a18f56e8913cf4a201fbe287825b095693c21775"
        );
        bridge.setActive(true);

        vm.expectRevert(
            "AccessControl: account 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf is missing role 0xa49807205ce4d355092ef5a8a18f56e8913cf4a201fbe287825b095693c21775"
        );
        bridge.setActive(false);
    }

    function testSetActivePausesToken() public {
        vm.startPrank(dave);

        bridge.setActive(false);
        assertEq(token.paused(), true);

        bridge.setActive(false);
        assertEq(token.paused(), true);

        bridge.setActive(true);
        assertEq(token.paused(), false);

        bridge.setActive(true);
        assertEq(token.paused(), false);
    }

    function testSetActiveWorks() public {
        vm.startPrank(dave);
        bridge.setActive(false);
        assertEq(bridge.bridgeActive(), false);
        bridge.setActive(true);
        assertEq(bridge.bridgeActive(), true);
        bridge.setActive(false);
        assertEq(bridge.bridgeActive(), false);
    }

    function testAddingAdminsRequiresAdmin() public {
        bytes32 admin = bridge.ADMIN_ROLE();

        assertEq(bridge.hasRole(admin, alice), false);
        bridge.grantRole(admin, alice);
        assertEq(bridge.hasRole(admin, alice), true);

        vm.prank(bob);
        vm.expectRevert(Unauthorized.selector);
        bridge.grantRole(admin, alice);

        vm.prank(dave);
        bridge.grantRole(admin, alice);
    }

    function testRemovingAdminsRequiresAdmin() public {
        bytes32 admin = bridge.ADMIN_ROLE();
        bridge.grantRole(admin, bob);

        vm.prank(charlie);
        vm.expectRevert(Unauthorized.selector);
        bridge.revokeRole(admin, bob);

        assertEq(bridge.hasRole(admin, bob), true);
        bridge.revokeRole(admin, bob);
        assertEq(bridge.hasRole(admin, bob), false);

        vm.prank(dave);
        bridge.revokeRole(admin, dave);
    }

    function testAddingSuperAdminsRequiresSuperAdmin() public {
        bytes32 superAdmin = bridge.SUPER_ADMIN_ROLE();

        assertEq(bridge.hasRole(superAdmin, alice), false);
        bridge.grantRole(superAdmin, alice);
        assertEq(bridge.hasRole(superAdmin, alice), true);

        vm.prank(bob);
        vm.expectRevert(Unauthorized.selector);
        bridge.grantRole(superAdmin, bob);

        vm.prank(dave);
        vm.expectRevert(Unauthorized.selector);
        bridge.grantRole(superAdmin, bob);
    }

    function testRemovingSuperAdminsRequiresSuperAdmin() public {
        bytes32 superAdmin = bridge.SUPER_ADMIN_ROLE();
        bridge.grantRole(superAdmin, alice);

        vm.prank(charlie);
        vm.expectRevert(Unauthorized.selector);
        bridge.revokeRole(superAdmin, alice);

        vm.prank(dave);
        vm.expectRevert(Unauthorized.selector);
        bridge.revokeRole(superAdmin, alice);

        assertEq(bridge.hasRole(superAdmin, alice), true);
        bridge.revokeRole(superAdmin, alice);
        assertEq(bridge.hasRole(superAdmin, alice), false);
    }

    function testRemovingWatcherRequiresSuperAdmin() public {
        bytes32 watcher = bridge.WATCHER_ROLE();
        vm.prank(alice);
        vm.expectRevert(Unauthorized.selector);
        bridge.revokeRole(watcher, alice);

        vm.prank(dave);
        vm.expectRevert(Unauthorized.selector);
        bridge.revokeRole(watcher, alice);

        bridge.revokeRole(watcher, alice);
    }

    function testRemovingWatcherWorks() public {
        bytes32 watcher = bridge.WATCHER_ROLE();

        assertEq(bridge.hasRole(watcher, alice), true);
        bridge.revokeRole(watcher, alice);
        assertEq(bridge.hasRole(watcher, alice), false);

        vm.prank(alice);
        vm.expectRevert(
            "AccessControl: account 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf is missing role 0x2125d1e225cadc5c8296e2cc1f96ee607770bf4a4a16131e62f6819937437c89"
        );
        bridge.emergencyStop();
    }

    function testAddingRelayRequiresSuperAdmin() public {
        bytes32 relay = bridge.RELAY_ROLE();

        vm.prank(alice);
        vm.expectRevert(Unauthorized.selector);
        bridge.grantRole(relay, dave);

        vm.prank(dave);
        vm.expectRevert(Unauthorized.selector);
        bridge.grantRole(relay, dave);

        bridge.grantRole(relay, dave);
    }

    function testAddingRelayWorks() public {
        bytes32 relay = bridge.RELAY_ROLE();

        assertEq(bridge.hasRole(relay, dave), false);
        bridge.grantRole(relay, dave);
        assertEq(bridge.hasRole(relay, dave), true);

        vm.prank(dave);
        bridge.voteMint(receipt1, 1, 100, alice);
    }

    function testRateLimitAllowsSingleTxAtLimit() public {
        bridge.setSupplyLimit(99999);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 1000, alice);
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 1000, alice);
        vm.roll(11);

        bridge.mint{value: 4}(receipt1);
    }

    function testRateLimitAllowsMultiTxAtLimit() public {
        bridge.setSupplyLimit(99999);
        vm.startPrank(bob);
        bridge.voteMint(receipt1, 1, 500, alice);
        bridge.voteMint(receipt2, 1, 500, alice);
        vm.stopPrank();
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 500, alice);
        bridge.voteMint(receipt2, 1, 500, alice);
        vm.stopPrank();
        vm.roll(11);

        bridge.mint{value: 4}(receipt1);
        bridge.mint{value: 4}(receipt2);
    }

    function testRateLimitAllowsUsingDecayedAmountRightAway() public {
        bridge.setSupplyLimit(99999);
        vm.startPrank(bob);
        bridge.voteMint(receipt1, 1, 1000, alice);
        bridge.voteMint(receipt2, 1, 10, alice);
        vm.stopPrank();
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 1000, alice);
        bridge.voteMint(receipt2, 1, 10, alice);
        vm.stopPrank();
        vm.roll(11);

        bridge.mint{value: 4}(receipt1);
        vm.roll(12);
        bridge.mint{value: 4}(receipt2);
    }

    function testRateLimitGoesToZeroAfterWholeWindow() public {
        bridge.setSupplyLimit(99999);
        vm.startPrank(bob);
        bridge.voteMint(receipt1, 1, 1000, alice);
        bridge.voteMint(receipt2, 1, 1000, alice);
        vm.stopPrank();
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 1000, alice);
        bridge.voteMint(receipt2, 1, 1000, alice);
        vm.stopPrank();
        vm.roll(11);

        bridge.mint{value: 4}(receipt1);
        vm.roll(111);
        bridge.mint{value: 4}(receipt2);
    }

    function testRateLimitPreventsSingleBigMint() public {
        bridge.setSupplyLimit(99999);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 1001, alice);
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 1001, alice);
        vm.roll(11);

        vm.expectRevert(RateLimited.selector);
        bridge.mint{value: 4}(receipt1);
    }

    function testRateLimitPreventsMultiBigMint() public {
        bridge.setSupplyLimit(99999);
        vm.startPrank(bob);
        bridge.voteMint(receipt1, 1, 501, alice);
        bridge.voteMint(receipt2, 1, 500, alice);
        vm.stopPrank();
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 501, alice);
        bridge.voteMint(receipt2, 1, 500, alice);
        vm.stopPrank();
        vm.roll(11);

        bridge.mint{value: 4}(receipt1);

        vm.expectRevert(RateLimited.selector);
        bridge.mint{value: 4}(receipt2);
    }

    function testRateLimitRespectsDecay() public {
        bridge.setSupplyLimit(99999);
        vm.startPrank(bob);
        bridge.voteMint(receipt1, 1, 1000, alice);
        bridge.voteMint(receipt2, 1, 500, alice);
        bridge.voteMint(receipt3, 1, 501, alice);
        vm.stopPrank();
        vm.startPrank(alice);
        bridge.voteMint(receipt1, 1, 1000, alice);
        bridge.voteMint(receipt2, 1, 500, alice);
        bridge.voteMint(receipt3, 1, 501, alice);
        vm.stopPrank();

        vm.roll(100);
        bridge.mint{value: 4}(receipt1);

        vm.roll(149);
        vm.expectRevert(RateLimited.selector);
        bridge.mint{value: 4}(receipt2);

        vm.roll(150);
        vm.expectRevert(RateLimited.selector);
        bridge.mint{value: 4}(receipt3);

        bridge.mint{value: 4}(receipt2);
    }

    function testMaxTotalSupplyIsRespectedAfterBurns() public {
        bridge.setSupplyLimit(500);
        bridge.setVotesRequired(1);

        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, alice);

        vm.roll(11);
        vm.expectRevert(TooMuchSupply.selector);
        bridge.mint(receipt1);

        bridge.burn(100, substrate1);
        bridge.mint{value: 4}(receipt1);
    }

    function testVotesRequiredCantBeZero() public {
        vm.expectRevert(InvalidArgument.selector);
        bridge.setVotesRequired(0);

        bridge.setVotesRequired(1);
    }

    function testOnlyUpgraderCanUpgrade() public {
        Bridge impl2 = new Bridge();

        vm.expectRevert(
            "AccessControl: account 0x7fa9385be102ac3eac297483dd6233d62b3e1496 is missing role 0x189ab7a9244df0848122154315af71fe140f3db0fe014031783b0946b8c9d2e3"
        );
        bridge.upgradeTo(address(impl2));

        bridge.grantRole(bridge.UPGRADER_ROLE(), address(this));
        bridge.upgradeTo(address(impl2));
    }

    function testOnlyUpgradesToUUPSCompatible() public {
        bridge.grantRole(bridge.UPGRADER_ROLE(), address(this));
        vm.expectRevert("ERC1967Upgrade: new implementation is not UUPS");
        bridge.upgradeTo(address(token));
    }

    function testIsUpgradeable() public {
        Bridge impl2 = new Bridge();
        bridge.grantRole(bridge.UPGRADER_ROLE(), address(this));
        bridge.upgradeTo(address(impl2));

        vm.expectCall(address(impl2), "");
        bridge.fee();
    }

    function testOnlySuperAdminCanTransferTokenOwnership() public {
        vm.prank(alice);
        vm.expectRevert(
            "AccessControl: account 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf is missing role 0x7613a25ecc738585a232ad50a301178f12b3ba8887d13e138b523c4269c47689"
        );
        bridge.transferTokenOwnership(alice);
    }

    function testTokenOwnershipIsTransferable() public {
        bridge.transferTokenOwnership(address(this));
        assertEq(token.owner(), address(this));
    }

    function testTokenTransferBricksBridge() public {
        bridge.transferTokenOwnership(address(this));
        assertEq(bridge.bridgeActive(), false);

        vm.prank(dave);
        vm.expectRevert(InvalidConfiguration.selector);
        bridge.setActive(true);
    }

    function testClaimRewardWorks() public {
        vm.prank(alice);
        bridge.voteMint(receipt1, 1, 100, dave);
        vm.prank(bob);
        bridge.voteMint(receipt1, 1, 100, dave);
        vm.prank(charlie);
        bridge.voteMint(receipt1, 1, 100, dave);

        vm.roll(11);
        bridge.mint{value: 19}(receipt1);

        assertEq(alice.balance, 100);

        vm.prank(alice);
        bridge.claimReward();
        assertEq(alice.balance, 111);
        assertEq(bob.balance, 0);
        assertEq(charlie.balance, 0);

        vm.prank(bob);
        bridge.claimReward();
        assertEq(alice.balance, 111);
        assertEq(bob.balance, 5);
        assertEq(charlie.balance, 0);

        vm.prank(charlie);
        bridge.claimReward();
        assertEq(alice.balance, 111);
        assertEq(bob.balance, 5);
        assertEq(charlie.balance, 3);
    }
}
