// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "openzeppelin-contracts/contracts/access/AccessControl.sol";
import "./WrappedToken.sol";
import "forge-std/console.sol";

struct ReceiptStruct {
	uint64 substrateBlockNumber;
	address ethRecipient;
	uint256 amount;
	uint approvedOn;
	uint processedOn;
}

struct RateLimitCounter {
	uint256 counter;
	uint lastUpdate;
}

struct RateLimitParameters {
	uint256 counterLimit;
	uint256 decayRate;
}

error BridgeInactive();
error AlreadyProcessed();
error NotApproved();
error TooSoon();
error InsufficientEther();
error RateLimited();
error AlreadyExists();
error InvalidWatcher();
error InvalidRelay();
error EthTransferFailed();
error Unauthorized();
error TooMuchSupply();
error InvalidArgument();

interface BridgeEvents {
	event OutgoingReceipt(uint256 amount, bytes32 substrateRecipient);
	event StateChanged(bool newBridgeState);
	event Approved(bytes32 indexed receiptId);
	event Vote(bytes32 receiptId, address relay);
	event Processed(bytes32 receiptId);
	event EmergencyStop(uint blockNumber, address voter, bytes32 receiptId);
}

contract Bridge is AccessControl, BridgeEvents {
	// 7613a25ecc738585a232ad50a301178f12b3ba8887d13e138b523c4269c47689
	bytes32 public constant SUPER_ADMIN_ROLE = keccak256("SUPER_ADMIN_ROLE");
	// a49807205ce4d355092ef5a8a18f56e8913cf4a201fbe287825b095693c21775
	bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
	// 077a1d526a4ce8a773632ab13b4fbbf1fcc954c3dab26cd27ea0e2a6750da5d7
	bytes32 public constant RELAY_ROLE = keccak256("RELAY_ROLE");
	// 2125d1e225cadc5c8296e2cc1f96ee607770bf4a4a16131e62f6819937437c89
	bytes32 public constant WATCHER_ROLE = keccak256("WATCHER_ROLE");
	// 189ab7a9244df0848122154315af71fe140f3db0fe014031783b0946b8c9d2e3
	bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");

	WrappedToken token;
	mapping(address => bool) relays;
	mapping(address => bool) watchers;
	uint32 votesRequired;
	uint256 fee;
	bool bridgeActive;
	mapping(bytes32 => ReceiptStruct) receipts;
	mapping(bytes32 => address[]) votes;
	RateLimitCounter mintCounter;
	RateLimitParameters rateLimit;
	uint mintDelay;
	mapping(address => uint256) pendingRewards;
	uint256 supplyLimit;

	constructor(WrappedToken token_, address superAdmin) {
		token = token_;

		_grantRole(SUPER_ADMIN_ROLE, superAdmin);
	}

	function grantRole(bytes32 role, address account) public override {
		bool authorized = false;

		// Only super admin can add super admins, relays and upgraders
		if (role == SUPER_ADMIN_ROLE || role == RELAY_ROLE || role == UPGRADER_ROLE)
			authorized = hasRole(SUPER_ADMIN_ROLE, msg.sender);

		// Super admin or admin can add admins and watchers
		if (role == ADMIN_ROLE || role == WATCHER_ROLE)
			authorized = hasRole(SUPER_ADMIN_ROLE, msg.sender) || hasRole(ADMIN_ROLE, msg.sender);

		if (!authorized) revert Unauthorized();
		_grantRole(role, account);
	}

	function revokeRole(bytes32 role, address account) public override {
		bool authorized = false;

		// Only super admin can remove super admins, watchers and upgraders
		if (role == SUPER_ADMIN_ROLE || role == WATCHER_ROLE || role == UPGRADER_ROLE)
			authorized = hasRole(SUPER_ADMIN_ROLE, msg.sender);

		// Super admin or admin can remove admins and relays
		if (role == ADMIN_ROLE || role == RELAY_ROLE)
			authorized = hasRole(SUPER_ADMIN_ROLE, msg.sender) || hasRole(ADMIN_ROLE, msg.sender);


		if (!authorized) revert Unauthorized();
		_revokeRole(role, account);
	}

	function burn(uint256 amount, bytes32 substrateRecipient) public {
		// CHECKS
		if (!bridgeActive) revert BridgeInactive();

		// EFFECTS
		emit OutgoingReceipt(amount, substrateRecipient);

		// INTERACTIONS
		token.burn(msg.sender, amount);
	}

	function voteMint(
		bytes32 receiptId,
		uint64 substrateBlockNumber,
		uint256 amount,
		address ethRecipient
	) public onlyRole(RELAY_ROLE) {
		if (!bridgeActive) revert BridgeInactive();
		if (substrateBlockNumber == 0) revert InvalidArgument();
		if (receipts[receiptId].processedOn > 0) revert AlreadyProcessed();

		// checks if already exists
		if (receipts[receiptId].substrateBlockNumber != 0) {
			if (!_checkReceiptMatches(receiptId, substrateBlockNumber, ethRecipient, amount)) {
				// someone lied, stop the bridge
				_setActive(false);
				return;
			}
		} else {
			receipts[receiptId].substrateBlockNumber = substrateBlockNumber;
			receipts[receiptId].ethRecipient = ethRecipient;
			receipts[receiptId].amount = amount;
		}

		// check if already voted
		if (_arrayContains(votes[receiptId], msg.sender)) return;

		votes[receiptId].push(msg.sender);

		if (receipts[receiptId].approvedOn == 0 && votes[receiptId].length >= votesRequired) {
			receipts[receiptId].approvedOn = block.number;
			emit Approved(receiptId);
		}

		emit Vote(receiptId, msg.sender);
	}

	function _checkReceiptMatches(
		bytes32 receiptId,
		uint64 substrateBlockNumber,
		address ethRecipient,
		uint256 amount
	) internal view returns (bool) {
		ReceiptStruct storage r = receipts[receiptId];
		if (r.substrateBlockNumber != substrateBlockNumber) return false;
		if (r.ethRecipient != ethRecipient) return false;
		if (r.amount != amount) return false;
		return true;
	}

	function _setActive(bool active) internal {
		if (active != bridgeActive) emit StateChanged(active);
		bridgeActive = active;
	}

	function _arrayContains(address[] storage arr, address needle) internal view returns (bool) {
		for (uint256 i = 0; i < arr.length; i++) if (arr[i] == needle) return true;
		return false;
	}

	function mint(bytes32 receiptId) public payable {
		// CHECKS
		if (!bridgeActive) revert BridgeInactive();

		ReceiptStruct storage receipt = receipts[receiptId];
		if (receipt.approvedOn == 0) revert NotApproved();

		if (receipt.processedOn != 0) revert AlreadyProcessed();

		if (block.number < receipt.approvedOn + mintDelay) revert TooSoon();

		if (token.totalSupply() + receipt.amount > supplyLimit) revert TooMuchSupply();

		// EFFECTS
		_takeFee(votes[receiptId]);
		_rateLimit(receipt.amount);
		emit Processed(receiptId);
		receipt.processedOn = block.number;

		// INTERACTIONS
		token.mint(receipt.ethRecipient, receipt.amount);
	}

	function _rateLimit(uint256 amount) internal {
		uint blocksElapsed = block.number - mintCounter.lastUpdate;
		uint256 decay = rateLimit.decayRate * blocksElapsed;
		uint256 counter;
		if (mintCounter.counter > decay) {
			unchecked {
				counter = mintCounter.counter - decay;
			}
		} else {
			counter = 0;
		}

		counter += amount;

		if (counter > rateLimit.counterLimit) revert RateLimited();

		mintCounter.counter = counter;
		mintCounter.lastUpdate = block.number;
	}

	function _takeFee(address[] storage receiptVotes) internal {
		if (msg.value < fee) revert InsufficientEther();
		uint256 perVoterFee = msg.value / receiptVotes.length;
		uint256 remainder = msg.value - perVoterFee * receiptVotes.length;

		uint256 totalPaid = 0;
		totalPaid += _giveReward(receiptVotes[0], perVoterFee + remainder);
		for (uint256 i = 1; i < receiptVotes.length; i++)
			totalPaid += _giveReward(receiptVotes[i], perVoterFee);

		assert(totalPaid == msg.value);
	}

	function _giveReward(address addr, uint256 amount) internal returns (uint256) {
		pendingRewards[addr] += amount;
		return amount;
	}

	function setFee(uint256 fee_) public onlyRole(ADMIN_ROLE) {
		fee = fee_;
	}
	
	function getFee() public view returns (uint256) {
		return fee;
	}

	function setVotesRequired(uint32 votesRequired_) public onlyRole(SUPER_ADMIN_ROLE) {
		votesRequired = votesRequired_;
	}

	function getVotesRequired() public view returns (uint32) {
		return votesRequired;
	}

	function addRelay(address addr) public onlyRole(SUPER_ADMIN_ROLE) {
		if (relays[addr]) revert AlreadyExists();

		relays[addr] = true;
	}

	function removeWatcher(address addr) public onlyRole(SUPER_ADMIN_ROLE) {
		if (!watchers[addr]) revert InvalidWatcher();

		delete watchers[addr];
	}

	function addWatcher(address addr) public onlyRole(ADMIN_ROLE) {
		if (watchers[addr]) revert AlreadyExists();

		watchers[addr] = true;
	}

	function removeRelay(address addr) public onlyRole(ADMIN_ROLE) {
		if (!relays[addr]) revert InvalidRelay();

		delete relays[addr];
	}

	function setActive(bool active) public onlyRole(ADMIN_ROLE) {
		_setActive(active);
	}

	function getActive() public view returns (bool) {
		return bridgeActive;
	}

	function emergencyStop(
		uint blockNumber,
		address voter,
		bytes32 receiptId
	) public onlyRole(WATCHER_ROLE) {
		emit EmergencyStop(blockNumber, voter, receiptId);
		_setActive(false);
	}

	function setMintDelay(uint delay) public onlyRole(SUPER_ADMIN_ROLE) {
		mintDelay = delay;
	}

	function claimReward() public {
		uint256 amount = pendingRewards[msg.sender];
		pendingRewards[msg.sender] = 0;
		(bool sent, ) = msg.sender.call{value: amount}("");
		if (!sent) revert EthTransferFailed();
	}

	function setRateLimit(
		uint256 counterLimit,
		uint256 decayRate
	) public onlyRole(SUPER_ADMIN_ROLE) {
		rateLimit.counterLimit = counterLimit;
		rateLimit.decayRate = decayRate;
	}

	function setSupplyLimit(uint256 supplyLimit_) public onlyRole(SUPER_ADMIN_ROLE) {
		supplyLimit = supplyLimit_;
	}

	function getReceipt(bytes32 receiptId) public view returns (ReceiptStruct memory) {
		return receipts[receiptId];
	}

	function getPendingReward(address addr) public view returns (uint256) {
		return pendingRewards[addr];
	}
}
