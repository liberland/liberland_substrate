#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  #[pallet::without_storage_info]
  pub struct Pallet<T>(_);

  #[pallet::config]
  pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
  }
  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    LawAdded { index1: u32, index2: u32 },
    LawRepealed { index1: u32, index2: u32 },
  }
  #[pallet::error]
  pub enum Error<T> {
    LawAlreadyExists,
  }
  #[pallet::storage]
  //metadata stored on centralized db in order for it to be available during proposal, referendum
  pub(super) type Laws<T: Config> = StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, u32, BoundedVec<u8, ConstU32<65536>>, ValueQuery>;
  #[pallet::call]
  impl<T: Config> Pallet<T> {

    #[pallet::weight(10_000)]
    pub fn add_law(origin: OriginFor<T>, index1: u32, index2:u32, law_content: BoundedVec<u8, ConstU32<65536>> ) -> DispatchResult {
    	ensure_root(origin)?;

    	ensure!(!Laws::<T>::contains_key(&index1, &index2), Error::<T>::LawAlreadyExists);

    	Laws::<T>::insert(&index1, &index2, &law_content);

    	Self::deposit_event(Event::LawAdded { index1, index2 });

    	Ok(())
    }

	#[pallet::weight(10_000)]
    pub fn repeal_law(origin: OriginFor<T>, index1: u32, index2:u32) -> DispatchResult {
        	ensure_root(origin)?;

        	Laws::<T>::remove(&index1, &index2);

        	Self::deposit_event(Event::LawRepealed { index1, index2 });

        	Ok(())
        }

  }
}
