// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {AccessControlUpgradeable} from
    "openzeppelin-contracts-upgradeable/contracts/access/AccessControlUpgradeable.sol";
import {WrappedToken} from "./WrappedToken.sol";

/// Struct for representing Substrate -> ETH transfer
struct IncomingReceiptStruct {
    uint64 substrateBlockNumber;
    address ethRecipient;
    uint256 amount;
    uint256 approvedOn;
    uint256 processedOn;
}

/// Rate limit parameters. Limit how fast can tokens be minted.This is
/// implemented as The Leaky Bucket as a Meter algorithm -
/// https://en.wikipedia.org/wiki/Leaky_bucket.
/// `counterLimit` is the max counter (a.k.a. max burst, max single withdrawal)
/// `decayRate` is after reaching max, how much can be minted per block.
struct RateLimitParameters {
    uint256 counterLimit;
    uint256 decayRate;
}

/// Struct for keeping track of counters required to enforce rate limits
struct RateLimitCounter {
    uint256 counter;
    uint256 lastUpdate;
}

/// Bridge is deactivated - see `bridgeActive()` and `setActive(bool)`
error BridgeInactive();
/// IncomingReceipt was already processed and tokens were minted
error AlreadyProcessed();
/// IncomingReceipt wasn't approved for minting yet
error NotApproved();
/// IncomingReceipt was approved, but `mintDelay` blocks didn't pass yet
error TooSoon();
/// Insufficient ether was sent - see `fee()`
error InsufficientEther();
/// Too much is being minted by the bridge at the moment, try again later
error RateLimited();
/// Eth transfer failed when redeeming reward
error EthTransferFailed();
/// Caller is not authorized for this call
error Unauthorized();
/// Total supply limit was reached - try again later
error TooMuchSupply();
/// Invalid argument
error InvalidArgument();
/// Invalid bridge configuration
error InvalidConfiguration();
/// This relay already voted for this ReceiptId
error AlreadyVoted();

/// @title Interface with events emitted by the bridge
interface BridgeEvents {
    /// Emitted after burn, notifies relays that transfer is happening
    /// @param amount Amount of token burned
    /// @param substrateRecipient Who should get tokens on substrate
    event OutgoingReceipt(uint256 amount, bytes32 substrateRecipient);

    /// Bridge get activated or deactivated
    /// @param newBridgeState New bridge state
    event StateChanged(bool newBridgeState);

    /// An IncomingReceipt was approved for transfer. `mint(bytes32)`
    /// can now be called for this receipt after `mintDelay` blocks pass.
    /// @param receiptId Receipt that got approved
    event Approved(bytes32 indexed receiptId);

    /// Vote was cast to approve IncomingReceipt
    /// @param receiptId subject Receipt
    /// @param relay Relay that cast the vote
    event Vote(bytes32 receiptId, address relay);

    /// IncomingReceipt was completely processed - tokens were minted
    /// @param receiptId subject Receipt
    event Processed(bytes32 receiptId);

    /// Bridge was emergency stopped by watcher - misbehavior by relay was
    /// detected
    event EmergencyStop();
}

/// @title Substrate <-> ETH bridge for Liberland
/// @dev Must be used with ERC1967Proxy
contract Bridge is Initializable, AccessControlUpgradeable, UUPSUpgradeable, BridgeEvents {
    // 7613a25ecc738585a232ad50a301178f12b3ba8887d13e138b523c4269c47689
    /// Role that's allowed to:
    /// * grant and revoke all roles
    /// * setVotesRequired
    /// * setMintDelay
    /// * setRateLimit
    /// * setSupplyLimit
    /// * transferTokenOwnership
    bytes32 public constant SUPER_ADMIN_ROLE = keccak256("SUPER_ADMIN_ROLE");

    // a49807205ce4d355092ef5a8a18f56e8913cf4a201fbe287825b095693c21775
    /// Role that's allowed to:
    /// * add and remove admins
    /// * add watchers
    /// * remove relays
    /// * setFee
    /// * setActive
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    // 077a1d526a4ce8a773632ab13b4fbbf1fcc954c3dab26cd27ea0e2a6750da5d7
    /// Role that's allowed to voteMint
    bytes32 public constant RELAY_ROLE = keccak256("RELAY_ROLE");

    // 2125d1e225cadc5c8296e2cc1f96ee607770bf4a4a16131e62f6819937437c89
    /// Role thats allowed to emergencyStop
    bytes32 public constant WATCHER_ROLE = keccak256("WATCHER_ROLE");

    // 189ab7a9244df0848122154315af71fe140f3db0fe014031783b0946b8c9d2e3
    /// Role that's allowed to upgrade the contract
    bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");

    /// If false, burning, voting, minting is disabled
    bool public bridgeActive;
    /// Number of votes required to approve IncomingReceipt
    uint32 public votesRequired;
    /// Fee that must be sent on `mint()` and is distributed to voters
    uint256 public fee;
    /// Delay in blocks between approval and allowing `mint()`
    uint256 public mintDelay;
    /// Maximum amount of tokens in circulation
    uint256 public supplyLimit;
    /// Address of token that will be minted/burned
    WrappedToken public token;
    /// Incoming Receipts - Substrate -> ETH transfers
    mapping(bytes32 receiptId => IncomingReceiptStruct receipt) public incomingReceipts;
    /// Votes for approving given receipt
    mapping(bytes32 receiptId => address[] votes) public votes;
    /// Rewards for relays that can be claimed with `claimReward`
    mapping(address voter => uint256 pendingReward) public pendingRewards;
    /// Counter for enforcing rate limit
    RateLimitCounter public mintCounter; // 2x uint256
    /// Rate limit configuration
    RateLimitParameters public rateLimit; // 2x uint256

    constructor() {
        _disableInitializers();
    }

    /// Initializer, should be called in the same tx as deployment
    /// @param token_ Address of ERC20 token to manage
    /// @param votesRequired_ initial `votesRequired`
    /// @param mintDelay_ initial `votesRequired`
    /// @param fee_ initial `votesRequired`
    /// @param counterLimit initial `rateLimit.counterLimit`
    /// @param decayRate initial `rateLimit.decayRate`
    /// @param supplyLimit_ initial `supplyLimit`
    function initialize(
        WrappedToken token_,
        uint32 votesRequired_,
        uint256 mintDelay_,
        uint256 fee_,
        uint256 counterLimit,
        uint256 decayRate,
        uint256 supplyLimit_
    ) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();

        token = token_;
        rateLimit.counterLimit = counterLimit;
        rateLimit.decayRate = decayRate;
        // slither-disable-start events-maths
        votesRequired = votesRequired_;
        fee = fee_;
        mintDelay = mintDelay_;
        supplyLimit = supplyLimit_;
        // slither-disable-end events-maths
        _grantRole(SUPER_ADMIN_ROLE, msg.sender);
    }

    /// Adding special users. See role docs on info who can grant each role
    /// @param role Role to grant
    /// @param account Account to grant the role to
    /// @dev Reverts with `Unauthorized()` if caller doesn't have correct role for
    ///      action
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

    /// Removing special users. See role docs on info who can remove each role
    /// @param role Role to remove
    /// @param account Account to remove the role from
    /// @dev Reverts with `Unauthorized()` if caller doesn't have correct role for
    ///      action
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

    /// Burn token and get receipt in return.
    /// This is the entrypoint for users to start ETH -> Substrate transfer.
    /// @param amount Amount of token to burn (a.k.a. transfer to substrate).
    /// @param substrateRecipient AccountId on substrate side that will receive
    ///                           funds
    /// @dev Reverts with `BridgeInactive` if bridge is inactive
    /// @dev Reverts with if bridge is not approved to manage funds or if caller has
    ///      insufficient balance
    /// @dev Emits `OutgoingReceipt(amount, substrateRecipient)` on success
    /// @dev Interacts with `token` contract
    function burn(uint256 amount, bytes32 substrateRecipient) public {
        // CHECKS
        if (!bridgeActive) revert BridgeInactive();

        // EFFECTS
        emit OutgoingReceipt(amount, substrateRecipient);

        // INTERACTIONS
        token.burn(msg.sender, amount);
    }

    /// Vote for approval of given receipt.
    /// This is the call used by relays to relay Substrate -> ETH transfers.
    /// @param receiptId Incoming receipt id.
    /// @param substrateBlockNumber Substrate block number on which Receipt was
    ///                             created.
    /// @param amount Amount being transfered
    /// @param ethRecipient Ethereum address of token recipient
    /// @dev Only addresses with RELAY_ROLE can call this
    /// @dev Reverts with `BridgeInactive` if bridge is inactive
    /// @dev Reverts with `InvalidArgument` if `substrateBlockNumber` is 0
    /// @dev Reverts with `AlreadyProcessed` if receipt was already fully processed
    /// @dev Will stop the bridge if details for this receiptId don't match
    ///      details relayed by other relays
    /// @dev Reverts with `AlreadyVoted()` if this relay already voted.
    /// @dev Emits `Approved(receiptId)` if this vote caused reaching
    ///      `votesRequired`
    /// @dev Emits `Vote(receiptId, msg.sender)` on success
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
        if (voted(receiptId, msg.sender)) revert AlreadyVoted();

        votes[receiptId].push(msg.sender);

        if (incomingReceipts[receiptId].approvedOn == 0 && votes[receiptId].length >= votesRequired) {
            incomingReceipts[receiptId].approvedOn = block.number;
            emit Approved(receiptId);
        }

        emit Vote(receiptId, msg.sender);
    }

    /// Mint tokens according to the receipt.
    /// Receipt must already be relayed and approved by relays.
    /// This is payable and requires sending at least `fee()` ether.
    /// @param receiptId Receipt
    /// @dev Reverts with `BridgeInactive` if bridge is inactive
    /// @dev Reverts with `AlreadyProcessed` if receipt was already processed
    /// @dev Reverts with `NotApproved` if receipt isn't approved by relays
    /// @dev Reverts with `TooSoon` if `mintDelay` blocks didn't pass since
    ///      approval
    /// @dev Reverts with `TooMuchSupply` if there's too many tokens in
    ///      circulation
    /// @dev Reverts with `RateLimited` if there's too many tokens minted in
    ///      short time
    /// @dev Reverts with `InsufficientEther` if not enough ether was sent
    /// @dev Emits `Processed(receiptId)` on success
    /// @dev Interacts with `token` contract
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

    /// Set the minting fee
    /// @param fee_ New minting fee
    /// @dev Only addresses with ADMIN_ROLE can call this
    function setFee(uint256 fee_) public onlyRole(ADMIN_ROLE) {
        fee = fee_;
    }

    /// Set the number of votes required to approve minting
    /// @param votesRequired_ Number of votes required
    /// @dev Only addresses with SUPER_ADMIN_ROLE can call this
    /// @dev Reverts with `InvalidArgument` if `votesRequired_` is 0
    function setVotesRequired(uint32 votesRequired_) public onlyRole(SUPER_ADMIN_ROLE) {
        if (votesRequired_ == 0) revert InvalidArgument();
        votesRequired = votesRequired_;
    }

    /// Stop/start bridge. Will also pause/unpause the token accordingly.
    /// @param active True to start, False to stop
    /// @dev Only addresses with ADMIN_ROLE can call this
    /// @dev Reverts with `InvalidArgument` if `votesRequired_` is 0
    /// @dev Reverts with `InvalidConfiguration` if `token` is ` and `active` is
    ///      true
    /// @dev Emits `StateChanged(active)` if state actually changed
    /// @dev Interacts with `token` contract
    function setActive(bool active) public onlyRole(ADMIN_ROLE) {
        _setActive(active);
    }

    /// Emergency stop - stops the bridge
    /// @dev Only addresses with ADMIN_ROLE can call this
    /// @dev Emits `StateChanged(active)` if state actually changed
    /// @dev Always emits `EmergencyStop()`
    function emergencyStop() public onlyRole(WATCHER_ROLE) {
        emit EmergencyStop();
        _setActive(false);
    }

    /// Set the minting delay
    /// @param delay New minting delay
    /// @dev Only addresses with SUPER_ADMIN_ROLE can call this
    function setMintDelay(uint256 delay) public onlyRole(SUPER_ADMIN_ROLE) {
        mintDelay = delay;
    }

    /// Claim pending reward
    /// Used by relays to get ether collected from users as minting fees.
    /// @dev May revert with EthTransferFailed
    function claimReward() public {
        uint256 amount = pendingRewards[msg.sender];
        pendingRewards[msg.sender] = 0;
        // disabling check as this is the recommended way of transferring ether
        // https://solidity-by-example.org/sending-ether/
        // slither-disable-next-line low-level-calls
        (bool sent,) = msg.sender.call{value: amount}("");
        if (!sent) revert EthTransferFailed();
    }

    /// Set rate limit parameters
    /// @param counterLimit New counter limit (max burst mint)
    /// @param decayRate New decay rate (avg mint per block)
    /// @dev Only addresses with SUPER_ADMIN_ROLE can call this
    function setRateLimit(uint256 counterLimit, uint256 decayRate) public onlyRole(SUPER_ADMIN_ROLE) {
        rateLimit.counterLimit = counterLimit;
        rateLimit.decayRate = decayRate;
    }

    /// Set max circulating token supply
    /// @param supplyLimit_ new supply limit
    /// @dev Only addresses with SUPER_ADMIN_ROLE can call this
    function setSupplyLimit(uint256 supplyLimit_) public onlyRole(SUPER_ADMIN_ROLE) {
        supplyLimit = supplyLimit_;
    }

    /// Transfer ownership of underlying token contract.
    /// Will stop the bridge and set the token address to 0, effectively
    /// bricking the bridge.
    /// @param newOwner new owner
    /// @dev Only addresses with SUPER_ADMIN_ROLE can call this
    /// @dev Interacts with `token` contract
    function transferTokenOwnership(address newOwner) public onlyRole(SUPER_ADMIN_ROLE) {
        WrappedToken token_ = token;
        token = WrappedToken(address(0));
        _setActive(false);
        token_.transferOwnership(newOwner);
    }

    // solhint-disable-next-line no-empty-blocks
    function _authorizeUpgrade(address newImplementation) internal override onlyRole(UPGRADER_ROLE) {}

    function _setActive(bool active) internal {
        if (active && address(token) == address(0)) revert InvalidConfiguration();
        if (active != bridgeActive) emit StateChanged(active);
        bridgeActive = active;

        if (address(token) != address(0)) {
            bool tokenPaused = token.paused();
            if (active && tokenPaused) {
                token.unpause();
            } else if (!active && !tokenPaused) {
                token.pause();
            }
        }
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

        uint256 votesCount = receiptVotes.length;

        // first vote costs ~110k gas
        // standard vote costs ~30k gas
        // approving vote costs ~50k gas
        // this assumes that hw/transfer cost is negligible in comparison to
        // network fees
        uint256 firstWeight = 8; // + standardWeight = 11
        uint256 approverWeight = 2; // + standardWeight = 5
        uint256 standardWeight = 3;

        uint256 totalWeight = firstWeight + approverWeight + votesCount * standardWeight;

        // disabling slither rule as we're specifically adjusting for the
        // precision loss here
        // slither-disable-start divide-before-multiply
        uint256 perWeightFee = msg.value / totalWeight;
        uint256 remainder = msg.value - perWeightFee * totalWeight;
        // slither-disable-end divide-before-multiply

        for (uint256 i = 0; i < votesCount;) {
            uint256 weight = standardWeight;
            if (i == 0) weight += firstWeight;
            if (i == votesRequired - 1) weight += approverWeight;

            uint256 reward = perWeightFee * weight;
            if (i == 0) reward += remainder;

            pendingRewards[receiptVotes[i]] += reward;
            unchecked {
                ++i;
            }
        }
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
        uint256 len = arr.length;
        for (uint256 i = 0; i < len;) {
            if (arr[i] == needle) return true;
            unchecked {
                ++i;
            }
        }
        return false;
    }

    function voted(bytes32 receiptId, address voter) public view returns (bool) {
        return _arrayContains(votes[receiptId], voter);
    }
}
