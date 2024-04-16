use super::*;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

type DbWeight = <Runtime as frame_system::Config>::DbWeight;

pub mod add_contracts_registry_pallet {
	use super::*;

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl OnRuntimeUpgrade for Migration<Runtime> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let mut weight = DbWeight::get().reads(1);

			if StorageVersion::get::<ContractsRegistry>() == 0 {
                StorageVersion::new(1).put::<ContractsRegistry>();
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

pub mod add_sora_bridge {
	use super::*;

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl OnRuntimeUpgrade for Migration<Runtime> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let mut weight = DbWeight::get().reads(1);

			if StorageVersion::get::<SubstrateBridgeInboundChannel>() == 0 {
                StorageVersion::new(1).put::<SubstrateBridgeInboundChannel>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }

			if StorageVersion::get::<SubstrateBridgeOutboundChannel>() == 0 {
                StorageVersion::new(1).put::<SubstrateBridgeOutboundChannel>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }

			if StorageVersion::get::<SubstrateDispatch>() == 0 {
                StorageVersion::new(1).put::<SubstrateDispatch>();
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