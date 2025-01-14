#![cfg_attr(not(feature = "std"), no_std)]

use ink::prelude::{string::String, vec::Vec};

/// Trait defining the interface for a staking contract.
#[ink::trait_definition]
pub trait Stakeable {
    /// Stakes the specified amount of tokens.
    #[ink(message, payable)]
    fn stake(&mut self);

    /// Unstakes the specified amount of tokens.
    #[ink(message)]
    fn unstake(&mut self, amount: u128);

    /// Claims rewards for the caller.
    #[ink(message)]
    fn claim(&mut self);

    /// Returns the staked balance of the given account.
    #[ink(message)]
    fn get_staked_balance(&self, staker: AccountId) -> u128;

    /// Returns the total staked amount in the contract.
    #[ink(message)]
    fn get_total_staked(&self) -> u128;

    /// Returns the total reward pool available in the contract.
    #[ink(message)]
    fn get_reward_pool(&self) -> u128;
}
