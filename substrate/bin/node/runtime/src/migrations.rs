use super::*;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

type DbWeight = <Runtime as frame_system::Config>::DbWeight;

pub mod add_ministry_of_finance_office_pallet {
	use super::*;

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl OnRuntimeUpgrade for Migration<Runtime> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let mut weight = DbWeight::get().reads(1);

			if StorageVersion::get::<MinistryOfFinanceOffice>() == 0 {
                StorageVersion::new(1).put::<MinistryOfFinanceOffice>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }

            weight
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			Ok(())
		}
	}
}