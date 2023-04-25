use super::*;
use frame_support::{
	pallet_prelude::*,
	traits::tokens::{
		fungible::{Inspect, Transfer},
		DepositConsequence, WithdrawConsequence,
	},
};
use sp_runtime::traits::Bounded;

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
	type Balance = T::Balance;

	fn total_issuance() -> Self::Balance {
		let id = Self::llm_id();
		Assets::<T>::total_supply(id)
	}

	fn minimum_balance() -> Self::Balance {
		1u8.into()
	}

	fn balance(who: &T::AccountId) -> Self::Balance {
		Self::balance(who.clone())
	}

	fn reducible_balance(who: &T::AccountId, keep_alive: bool) -> Self::Balance {
		let balance = <Self as Inspect<T::AccountId>>::balance(who);
		if keep_alive {
			balance - Self::minimum_balance()
		} else {
			balance
		}
	}

	fn can_deposit(who: &T::AccountId, amount: Self::Balance, _mint: bool) -> DepositConsequence {
		if Self::Balance::max_value() - <Self as Inspect<T::AccountId>>::balance(who) > amount {
			DepositConsequence::Overflow
		} else {
			DepositConsequence::Success
		}
	}

	fn can_withdraw(who: &T::AccountId, amount: Self::Balance) -> WithdrawConsequence<Self::Balance> {
		if <Self as Inspect<T::AccountId>>::balance(who) < amount {
			WithdrawConsequence::NoFunds
		} else {
			WithdrawConsequence::Success
		}
	}
}

impl<T: Config> Transfer<T::AccountId> for Pallet<T> {
	fn transfer(
		source: &T::AccountId,
		dest: &T::AccountId,
		amount: Self::Balance,
		_keep_alive: bool,
	) -> Result<Self::Balance, DispatchError> {
		Pallet::<T>::transfer(source.clone(), dest.clone(), amount)?;
		Ok(amount)
	}

	fn deactivate(_: Self::Balance) {
		unimplemented!();
	}

	fn reactivate(_: Self::Balance) {
		unimplemented!();
	}
}
