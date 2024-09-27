use super::*;
use frame_support::{
	pallet_prelude::*,
	traits::tokens::{
		fungible::{Dust, Inspect, Mutate, Unbalanced},
		fungibles::{
			Dust as FungiblesDust, Inspect as FungiblesInspect, Unbalanced as FungiblesUnbalanced,
		},
		DepositConsequence, Fortitude, Precision, Preservation, Provenance, WithdrawConsequence,
	},
};

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
	type Balance = BalanceOfAssets<T>;

	fn total_issuance() -> Self::Balance {
		let id = Self::llm_id();
		Assets::<T>::total_issuance(id)
	}

	fn minimum_balance() -> Self::Balance {
		let id = Self::llm_id();
		Assets::<T>::minimum_balance(id)
	}

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		let id = Self::llm_id();
		Assets::<T>::total_balance(id, who)
	}

	fn balance(who: &T::AccountId) -> Self::Balance {
		let id = Self::llm_id();
		Assets::<T>::balance(id, who)
	}

	fn reducible_balance(
		who: &T::AccountId,
		preservation: Preservation,
		force: Fortitude,
	) -> Self::Balance {
		let id = Self::llm_id();
		Assets::<T>::reducible_balance(id, who, preservation, force)
	}

	fn can_deposit(
		who: &T::AccountId,
		amount: Self::Balance,
		provenance: Provenance,
	) -> DepositConsequence {
		let id = Self::llm_id();
		Assets::<T>::can_deposit(id, who, amount, provenance)
	}

	fn can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		let id = Self::llm_id();
		Assets::<T>::can_withdraw(id, who, amount)
	}
}

impl<T: Config> Unbalanced<T::AccountId> for Pallet<T> {
	fn handle_dust(dust: Dust<T::AccountId, Self>) {
		let asset_id = Self::llm_id().into();
		let dust = FungiblesDust(asset_id, dust.0);
		Assets::<T>::handle_dust(dust);
	}

	fn write_balance(
		who: &T::AccountId,
		amount: Self::Balance,
	) -> Result<Option<Self::Balance>, DispatchError> {
		let asset_id = Self::llm_id().into();
		Assets::<T>::write_balance(asset_id, who, amount)
	}

	fn set_total_issuance(amount: Self::Balance) {
		let asset_id = Self::llm_id().into();
		Assets::<T>::set_total_issuance(asset_id, amount)
	}

	fn decrease_balance(
		who: &T::AccountId,
		amount: Self::Balance,
		precision: Precision,
		preservation: Preservation,
		fortitude: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		let asset_id = Self::llm_id().into();
		Assets::<T>::decrease_balance(asset_id, who, amount, precision, preservation, fortitude)
	}

	fn increase_balance(
		who: &T::AccountId,
		amount: Self::Balance,
		precision: Precision,
	) -> Result<Self::Balance, DispatchError> {
		let asset_id = Self::llm_id().into();
		Assets::<T>::increase_balance(asset_id, who, amount, precision)
	}
}

impl<T: Config> Mutate<T::AccountId> for Pallet<T> {}
