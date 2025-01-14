# Staking Contract

## Overview
The Staking Contract is a smart contract built using [Ink!](https://paritytech.github.io/ink/) for managing staking and rewards distribution. It allows:
- Users to **stake** tokens to earn rewards
- A team to **deposit rewards**, which are distributed to users proportionally based on their stakes
- Users to **unstake** their tokens and **claim rewards**

## Features
- **Stake Tokens**: Users can stake tokens to participate in the reward pool
- **Unstake Tokens**: Users can withdraw their staked tokens anytime
- **Reward Distribution**: The contract calculates rewards based on the proportion of the user's stake
- **Claim Rewards**: Users can claim their accumulated rewards anytime
- **Event Emission**: Emits events for staking, unstaking, reward deposits, and claims

## Technical Details
- **Language**: [Rust](https://www.rust-lang.org/)
- **Framework**: [Ink!](https://paritytech.github.io/ink/)
- **Dependencies**:
  - `ink`: Smart contract framework for Substrate
  - `scale`: Used for encoding/decoding data
  - `scale-info`: Metadata generation for the contract

## Functions

### User Functions
- **`stake`**: Stake tokens by transferring them to the contract.
- **`unstake(amount: u128)`**: Withdraw staked tokens.
- **`claim`**: Claim rewards accumulated based on the user's stake.

### View Functions
- **`get_staked_balance(staker: AccountId) -> u128`**: Returns the staked balance of a user.
- **`get_total_staked() -> u128`**: Returns the total staked amount in the contract.
- **`get_reward_pool() -> u128`**: Returns the available reward pool.

### Team Functions
- **`deposit_rewards`**: Deposit rewards into the contract. These rewards are distributed proportionally among all stakers.