use super::{pallet::Config, *};
use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::vec::Vec;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

pub mod v2 {
	use super::*;

	const TARGET: &'static str = "runtime::llm::migration::v2";
    pub type I = ();

	pub struct Migration<T, OldInterval>(sp_std::marker::PhantomData<(T, OldInterval)>);

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

	impl<T: Config, OldInterval: Get<BlockNumberFor<T>>> OnRuntimeUpgrade
		for Migration<T, OldInterval>
	{
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			assert!(StorageVersion::get::<Pallet<T>>() == 1, "can only upgrade from version 1");

			Ok(().encode())
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

			let _ = v2::JudgesSignatures::<T>::clear(u32::MAX, None);
			let _ = v2::PartiesSignatures::<T>::clear(u32::MAX, None);

            Contracts::<T>::translate(|_, maybe_contract_data: Option<
                    OldContractDataStorage<
                        <T as Config<I>>::MaxContractContentLen, 
                        <T as Config<I>>::MaxParties, 
                        <T as frame_system::Config>::AccountId, 
                        BalanceOf<T, I>
                    >
                >| {
                if let Some(contract_data) = maybe_contract_data {
                    return Some(ContractDataStorage {
                         data: contract_data.data,
                         parties: Some(contract_data.parties),
                         creator: contract_data.creator,
                         deposit: contract_data.deposit,
                    })
                }
                None
            });
            
            StorageVersion::new(2).put::<Pallet<T>>();
			weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 2, "must upgrade");
			Ok(())
		}
	}
}

#[derive(Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(MaxContractsLen, MaxParties))]
pub struct OldContractDataStorage<MaxContractsLen: Get<u32>, MaxParties: Get<u32>, AccountId, Balance>
{
	pub data: BoundedVec<u8, MaxContractsLen>,
	pub parties: BoundedVec<AccountId, MaxParties>,
	pub creator: AccountId,
	pub deposit: Balance,
}
