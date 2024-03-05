use super::*;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

type DbWeight = <Runtime as frame_system::Config>::DbWeight;

pub mod add_pallets {
	use super::*;

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl OnRuntimeUpgrade for Migration<Runtime> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			Ok(().encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let mut weight = DbWeight::get().reads(1);

			if StorageVersion::get::<EthLLDBridge>() == 0 {
                StorageVersion::new(1).put::<EthLLDBridge>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<EthLLMBridge>() == 0 {
                StorageVersion::new(1).put::<EthLLMBridge>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<CouncilAccount>() == 0 {
                StorageVersion::new(1).put::<CouncilAccount>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<PoolAssets>() == 0 {
                StorageVersion::new(1).put::<PoolAssets>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<CompanyRegistry>() == 0 {
                StorageVersion::new(1).put::<CompanyRegistry>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<IdentityOffice>() == 0 {
                StorageVersion::new(1).put::<IdentityOffice>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<CompanyRegistryOffice>() == 0 {
                StorageVersion::new(1).put::<CompanyRegistryOffice>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<LandRegistryOffice>() == 0 {
                StorageVersion::new(1).put::<LandRegistryOffice>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<MetaverseLandRegistryOffice>() == 0 {
                StorageVersion::new(1).put::<MetaverseLandRegistryOffice>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<AssetRegistryOffice>() == 0 {
                StorageVersion::new(1).put::<AssetRegistryOffice>();
                weight = weight.saturating_add(DbWeight::get().reads_writes(1, 1));
            }
			if StorageVersion::get::<Senate>() == 0 {
                StorageVersion::new(4).put::<Senate>();
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


pub mod society_to_v2 {
	use super::*;
	use pallet_society::migrations::VersionUncheckedMigrateToV2;
	use frame_support::migrations::StoreCurrentStorageVersion;

	type SocietyMigration = VersionUncheckedMigrateToV2<Runtime, (), PastPayouts>;

	pub struct Migration<T>(sp_std::marker::PhantomData<T>);

	impl OnRuntimeUpgrade for Migration<Runtime> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			SocietyMigration::pre_upgrade()
		}

		fn on_runtime_upgrade() -> Weight {
			let mut weight = DbWeight::get().reads(1);
			weight = weight.saturating_add(SocietyMigration::on_runtime_upgrade());
			weight = weight.saturating_add(DbWeight::get().reads_writes(0, 1));
			<StorageVersion as StoreCurrentStorageVersion<Society>>::store_current_storage_version();
            weight
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
			SocietyMigration::post_upgrade(state)
		}
	}
}
