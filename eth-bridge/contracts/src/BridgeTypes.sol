// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

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
