use super::{pallet::Config, *};
use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade};
use sp_std::vec::Vec;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

pub mod v1 {
	use super::*;

	pub type I = ();
	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	#[derive(Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(MaxContractsLen, MaxParties))]
	pub struct ContractDataStorage<
		MaxContractsLen: Get<u32>,
		MaxParties: Get<u32>,
		AccountId,
		Balance,
	> {
		pub data: BoundedVec<u8, MaxContractsLen>,
		pub parties: BoundedVec<AccountId, MaxParties>,
		pub creator: AccountId,
		pub deposit: Balance,
	}

	#[storage_alias]
	pub type PartiesSignatures<T: Config<I>> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		ContractIndex,
		Vec<<T as frame_system::Config>::AccountId>,
		OptionQuery,
	>;

	#[storage_alias]
	pub type JudgesSignatures<T: Config<I>> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		ContractIndex,
		Vec<<T as frame_system::Config>::AccountId>,
		OptionQuery,
	>;

	#[storage_alias]
	pub type Contracts<T: Config<I>, I: 'static = ()> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		ContractIndex,
		ContractDataStorage<
			<T as Config>::MaxContractContentLen,
			<T as Config>::MaxParties,
			<T as frame_system::Config>::AccountId,
			BalanceOf<T, I>,
		>,
		OptionQuery,
	>;
}

pub mod v2 {
	use super::*;

	const TARGET: &'static str = "runtime::contracts-registry::migration::v2";
	pub type I = ();

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for Migration<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			assert!(StorageVersion::get::<Pallet<T>>() == 1, "can only upgrade from version 1");
			let contracts_size = v1::Contracts::<T, I>::iter().count();
			log::warn!(
				target: TARGET,
				"Pre upgrade contracts size: {}",
				contracts_size
			);
			Ok((contracts_size as u32).encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let weight = T::DbWeight::get().reads(1);
			if StorageVersion::get::<Pallet<T>>() != 1 {
				log::warn!(
					target: TARGET,
					"skipping on_runtime_upgrade: executed on wrong storage version.\
				Expected version 1"
				);
				return weight;
			}

			let _ = v1::JudgesSignatures::<T>::clear(u32::MAX, None);
			let _ = v1::PartiesSignatures::<T>::clear(u32::MAX, None);

			Contracts::<T>::translate(
				|_,
				 contract_data: v1::ContractDataStorage<
					<T as Config<I>>::MaxContractContentLen,
					<T as Config<I>>::MaxParties,
					<T as frame_system::Config>::AccountId,
					BalanceOf<T, I>,
				>| {
					Some(ContractDataStorage {
						data: contract_data.data,
						parties: Some(contract_data.parties),
						creator: contract_data.creator,
						deposit: contract_data.deposit,
					})
				},
			);

			StorageVersion::new(2).put::<Pallet<T>>();
			weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 2, "must upgrade");
			let old_contracts_size: u32 =
				Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state");
			let contracts_size = Contracts::<T>::iter().count();
			log::warn!(
				target: TARGET,
				"Post upgrade contracts size: {}",
				contracts_size
			);
			assert_eq!(contracts_size, old_contracts_size as usize, "must migrate all contracts");
			Ok(())
		}
	}
}
