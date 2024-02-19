// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

/// @title Interface with events emitted by the bridge
interface BridgeEvents {
    /// Emitted after burn, notifies relays that transfer is happening
    /// @param from Account that burned its tokens
    /// @param substrateRecipient Who should get tokens on substrate
    /// @param amount Amount of token burned
    event OutgoingReceipt(address indexed from, bytes32 indexed substrateRecipient, uint256 amount);

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
    event Vote(bytes32 indexed receiptId, address indexed relay, uint64 substrateBlockNumber);

    /// IncomingReceipt was completely processed - tokens were minted
    /// @param receiptId subject Receipt
    event Processed(bytes32 indexed receiptId);

    /// Bridge was emergency stopped by watcher - misbehavior by relay was
    /// detected
    event EmergencyStop();
}
