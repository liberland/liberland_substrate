#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod staking_lld {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;

    #[ink(event)]
    pub struct Staked {
        #[ink(topic)]
        staker: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Unstaked {
        #[ink(topic)]
        staker: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct RewardDeposited {
        #[ink(topic)]
        team: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct RewardClaimed {
        #[ink(topic)]
        staker: AccountId,
        amount: u128,
    }

    #[ink(storage)]
    pub struct Staking {
        total_staked: u128,
        reward_pool: u128,
        staked_balances: Mapping<AccountId, u128>,
        reward_debt: Mapping<AccountId, u128>,
        acc_reward_per_share: u128,
        multiplier: u128,
    }

    impl Default for Staking {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Staking {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                total_staked: 0,
                reward_pool: 0,
                staked_balances: Mapping::default(),
                reward_debt: Mapping::default(),
                acc_reward_per_share: 0,
                multiplier: 1_000_000_000_000,
            }
        }

        #[ink(message, payable)]
        pub fn stake(&mut self) {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();

            assert!(amount > 0, "Stake amount must be greater than zero");

            self.update_rewards();

            let current_balance = self.staked_balances.get(caller).unwrap_or(0);
            let new_balance = current_balance
                .checked_add(amount)
                .expect("Overflow in staking");

            self.staked_balances.insert(caller, &new_balance);
            self.total_staked = self.total_staked
                .checked_add(amount)
                .expect("Overflow in total staked");

            let reward_debt = new_balance
                .checked_mul(self.acc_reward_per_share)
                .unwrap_or(0)
                / self.multiplier;
            self.reward_debt.insert(caller, &reward_debt);

            self.env().emit_event(Staked { staker: caller, amount });
        }

        #[ink(message)]
        pub fn unstake(&mut self, amount: u128) {
            let caller = self.env().caller();
            let current_balance = self.staked_balances.get(caller).unwrap_or(0);

            assert!(amount > 0, "Unstake amount must be greater than zero");
            assert!(current_balance >= amount, "Insufficient staked balance");

            self.update_rewards();

            let new_balance = current_balance
                .checked_sub(amount)
                .expect("Underflow in unstaking");

            self.staked_balances.insert(caller, &new_balance);
            self.total_staked = self.total_staked
                .checked_sub(amount)
                .expect("Underflow in total staked");

            let reward_debt = new_balance
                .checked_mul(self.acc_reward_per_share)
                .unwrap_or(0)
                / self.multiplier;
            self.reward_debt.insert(caller, &reward_debt);

            assert!(
                self.env().transfer(caller, amount).is_ok(),
                "Unstake transfer failed"
            );

            self.env().emit_event(Unstaked { staker: caller, amount });
        }

        #[ink(message, payable)]
        pub fn deposit_rewards(&mut self) {
            let amount = self.env().transferred_value();

            assert!(amount > 0, "Deposit amount must be greater than zero");

            self.update_rewards();

            self.reward_pool = self.reward_pool
                .checked_add(amount)
                .expect("Overflow in reward pool");
            self.env().emit_event(RewardDeposited {
                team: self.env().caller(),
                amount,
            });
        }

        #[ink(message)]
        pub fn claim(&mut self) {
            let caller = self.env().caller();
            self.update_rewards();

            let staked_balance = self.staked_balances.get(caller).unwrap_or(0);
            let reward_debt: u128 = self.reward_debt.get(caller).unwrap_or(0);

            let pending_reward = staked_balance
                .checked_mul(self.acc_reward_per_share)
                .unwrap_or(0)
                / self.multiplier
                .checked_sub(reward_debt)
                .unwrap_or(0);

            assert!(pending_reward > 0, "No rewards to claim");

            self.reward_debt.insert(
                caller,
                &(staked_balance
                    .checked_mul(self.acc_reward_per_share)
                    .unwrap_or(0)
                    / self.multiplier),
            );

            self.reward_pool = self.reward_pool
                .checked_sub(pending_reward)
                .expect("Underflow in reward pool");
            assert!(
                self.env().transfer(caller, pending_reward).is_ok(),
                "Reward transfer failed"
            );

            self.env().emit_event(RewardClaimed {
                staker: caller,
                amount: pending_reward,
            });
        }

        fn update_rewards(&mut self) {
            if self.total_staked > 0 {
                let reward_per_share = self.reward_pool
                    .checked_mul(self.multiplier)
                    .unwrap_or(0)
                    / self.total_staked;

                self.acc_reward_per_share = self.acc_reward_per_share
                    .checked_add(reward_per_share)
                    .expect("Overflow in reward per share");
                self.reward_pool = 0;
            }
        }
    }
}