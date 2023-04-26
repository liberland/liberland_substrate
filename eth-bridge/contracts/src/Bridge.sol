// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {AccessControl} from "openzeppelin-contracts/contracts/access/AccessControl.sol";
import {WrappedToken} from "./WrappedToken.sol";

struct IncomingReceiptStruct {
    uint64 substrateBlockNumber;
    address ethRecipient;
    uint256 amount;
    uint256 approvedOn;
    uint256 processedOn;
}

struct RateLimitCounter {
    uint256 counter;
    uint256 lastUpdate;
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
error EthTransferFailed();
error Unauthorized();
error TooMuchSupply();
error InvalidArgument();
error InvalidConfiguration();

interface BridgeEvents {
    event OutgoingReceipt(uint256 amount, bytes32 substrateRecipient);
    event StateChanged(bool newBridgeState);
    event Approved(bytes32 indexed receiptId);
    event Vote(bytes32 receiptId, address relay);
    event Processed(bytes32 receiptId);
    event EmergencyStop();
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

    WrappedToken public immutable token;
    uint32 public votesRequired;
    uint256 public fee;
    bool public bridgeActive;
    mapping(bytes32 receiptId => IncomingReceiptStruct receipt) public incomingReceipts;
    mapping(bytes32 receiptId => address[] voters) public votes;
    RateLimitCounter public mintCounter;
    RateLimitParameters public rateLimit;
    uint256 public mintDelay;
    mapping(address voter => uint256 pendingReward) public pendingRewards;
    uint256 public supplyLimit;

    constructor(WrappedToken token_, address superAdmin) {
        token = token_;
        votesRequired = 1;

        _grantRole(SUPER_ADMIN_ROLE, superAdmin);
    }

    function grantRole(bytes32 role, address account) public override {
        bool authorized = false;

        // Only super admin can add super admins, relays and upgraders
        if (role == SUPER_ADMIN_ROLE || role == RELAY_ROLE || role == UPGRADER_ROLE) {
            authorized = hasRole(SUPER_ADMIN_ROLE, msg.sender);
        }

        // Super admin or admin can add admins and watchers
        if (role == ADMIN_ROLE || role == WATCHER_ROLE) {
            authorized = hasRole(SUPER_ADMIN_ROLE, msg.sender) || hasRole(ADMIN_ROLE, msg.sender);
        }

        if (!authorized) revert Unauthorized();
        _grantRole(role, account);
    }

    function revokeRole(bytes32 role, address account) public override {
        bool authorized = false;

        // Only super admin can remove super admins, watchers and upgraders
        if (role == SUPER_ADMIN_ROLE || role == WATCHER_ROLE || role == UPGRADER_ROLE) {
            authorized = hasRole(SUPER_ADMIN_ROLE, msg.sender);
        }

        // Super admin or admin can remove admins and relays
        if (role == ADMIN_ROLE || role == RELAY_ROLE) {
            authorized = hasRole(SUPER_ADMIN_ROLE, msg.sender) || hasRole(ADMIN_ROLE, msg.sender);
        }

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

    function voteMint(bytes32 receiptId, uint64 substrateBlockNumber, uint256 amount, address ethRecipient)
        public
        onlyRole(RELAY_ROLE)
    {
        if (!bridgeActive) revert BridgeInactive();
        if (substrateBlockNumber == 0) revert InvalidArgument();
        if (incomingReceipts[receiptId].processedOn > 0) revert AlreadyProcessed();

        // checks if already exists
        if (incomingReceipts[receiptId].substrateBlockNumber != 0) {
            if (!_checkReceiptMatches(receiptId, substrateBlockNumber, ethRecipient, amount)) {
                // someone lied, stop the bridge
                _setActive(false);
                return;
            }
        } else {
            incomingReceipts[receiptId].substrateBlockNumber = substrateBlockNumber;
            incomingReceipts[receiptId].ethRecipient = ethRecipient;
            incomingReceipts[receiptId].amount = amount;
        }

        // check if already voted
        if (_arrayContains(votes[receiptId], msg.sender)) return;

        votes[receiptId].push(msg.sender);

        if (incomingReceipts[receiptId].approvedOn == 0 && votes[receiptId].length >= votesRequired) {
            incomingReceipts[receiptId].approvedOn = block.number;
            emit Approved(receiptId);
        }

        emit Vote(receiptId, msg.sender);
    }

    function mint(bytes32 receiptId) public payable {
        // CHECKS
        if (!bridgeActive) revert BridgeInactive();

        IncomingReceiptStruct storage receipt = incomingReceipts[receiptId];
        if (receipt.approvedOn == 0) revert NotApproved();

        if (receipt.processedOn != 0) revert AlreadyProcessed();

        if (block.number < receipt.approvedOn + mintDelay) revert TooSoon();

        if (token.totalSupply() + receipt.amount > supplyLimit) {
            revert TooMuchSupply();
        }

        // EFFECTS
        _takeFee(votes[receiptId]);
        _rateLimit(receipt.amount);
        emit Processed(receiptId);
        receipt.processedOn = block.number;

        // INTERACTIONS
        token.mint(receipt.ethRecipient, receipt.amount);
    }

    function setFee(uint256 fee_) public onlyRole(ADMIN_ROLE) {
        fee = fee_;
    }

    function setVotesRequired(uint32 votesRequired_) public onlyRole(SUPER_ADMIN_ROLE) {
        if (votesRequired_ == 0) revert InvalidConfiguration();
        votesRequired = votesRequired_;
    }

    function setActive(bool active) public onlyRole(ADMIN_ROLE) {
        _setActive(active);
    }

    function emergencyStop() public onlyRole(WATCHER_ROLE) {
        emit EmergencyStop();
        _setActive(false);
    }

    function setMintDelay(uint256 delay) public onlyRole(SUPER_ADMIN_ROLE) {
        mintDelay = delay;
    }

    function claimReward() public {
        uint256 amount = pendingRewards[msg.sender];
        pendingRewards[msg.sender] = 0;
        // disabling check as this is the recommended way of transferring ether
        // https://solidity-by-example.org/sending-ether/
        // slither-disable-next-line low-level-calls
        (bool sent,) = msg.sender.call{value: amount}("");
        if (!sent) revert EthTransferFailed();
    }

    function setRateLimit(uint256 counterLimit, uint256 decayRate) public onlyRole(SUPER_ADMIN_ROLE) {
        rateLimit.counterLimit = counterLimit;
        rateLimit.decayRate = decayRate;
    }

    function setSupplyLimit(uint256 supplyLimit_) public onlyRole(SUPER_ADMIN_ROLE) {
        supplyLimit = supplyLimit_;
    }

    function _setActive(bool active) internal {
        if (active != bridgeActive) emit StateChanged(active);
        bridgeActive = active;
    }

    function _rateLimit(uint256 amount) internal {
        uint256 blocksElapsed = block.number - mintCounter.lastUpdate;
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

        // first vote costs ~110k gas
        // standard vote costs ~30k gas
        // approving vote costs ~50k gas
        // this assumes that hw/transfer cost is negligible in comparison to
        // network fees
        uint256 firstWeight = 8; // + standardWeight = 11
        uint256 approverWeight = 2; // + standardWeight = 5
        uint256 standardWeight = 3;

        uint256 totalWeight = firstWeight + approverWeight + receiptVotes.length * standardWeight;

        // disabling slither rule as we're specifically adjusting for the
        // precision loss here
        // slither-disable-start divide-before-multiply
        uint256 perWeightFee = msg.value / totalWeight;
        uint256 remainder = msg.value - perWeightFee * totalWeight;
        // slither-disable-end divide-before-multiply

        uint256 totalPaid = 0;
        for (uint256 i = 0; i < receiptVotes.length; i++) {
            uint256 weight = standardWeight;
            if (i == 0) weight += firstWeight;
            if (i == votesRequired - 1) weight += approverWeight;

            uint256 reward = perWeightFee * weight;
            if (i == 0) reward += remainder;

            totalPaid += _giveReward(receiptVotes[i], reward);
        }

        assert(totalPaid == msg.value);
    }

    function _giveReward(address addr, uint256 amount) internal returns (uint256) {
        pendingRewards[addr] += amount;
        return amount;
    }

    function _checkReceiptMatches(bytes32 receiptId, uint64 substrateBlockNumber, address ethRecipient, uint256 amount)
        internal
        view
        returns (bool)
    {
        IncomingReceiptStruct storage r = incomingReceipts[receiptId];
        if (r.substrateBlockNumber != substrateBlockNumber) return false;
        if (r.ethRecipient != ethRecipient) return false;
        if (r.amount != amount) return false;
        return true;
    }

    function _arrayContains(address[] storage arr, address needle) internal view returns (bool) {
        for (uint256 i = 0; i < arr.length; i++) {
            if (arr[i] == needle) return true;
        }
        return false;
    }
}
