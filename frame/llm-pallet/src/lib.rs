#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

/// Liberland Merit Pallet
/*
decrease the total supply with 0.9 % per year
mint 0.9% per year to the treasury with pallet assets

*/



#[frame_support::pallet]
pub mod pallet {
	// Import various types used to declare pallet in scope.
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_system::ensure_signed;
	use frame_support::pallet_prelude::DispatchResult;
	use frame_support::PalletId;
	use frame_support::sp_runtime::traits::AccountIdConversion;
	// Every year the pallet mints 0.9% of the total supply.
//	use frame_support::traits::tokens::AssetId;
	//pub type AssetId = u64;

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
	pub struct MetaData<AccountId, Balance> {
    Reserve_am: Balance,
    Minted_am: u64,
    BlockNumber: AccountId,
	}
	//use pallet_assets::Asset::AssetId; // ::AssetId;

//	pub const CORE_ASSET_ID: AssetId = 0;

	#[pallet::storage]
	#[pallet::getter(fn minted_amount)]
	/// Keep track of the amount of minted llm
	pub(super) type MintedAmount<T: Config> = StorageValue<_, u64, ValueQuery>;


	#[pallet::config]
	pub trait Config: pallet_assets::Config + frame_system::Config { // include pallet asset config aswell

		type Total_supply: Get<u64>; // Pre defined the total supply in runtime
		type PreMintedAmount: Get<u64>; // Pre defined the total supply in runtime

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	//	type WeightInfo: WeightInfo;
	}


	#[pallet::error]
	pub enum Error<T> {
			/// Invalid amount
			InvalidAmount,
			/// Balance is less then sending amount
			LowBalance,
			LowAmount,
		}


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

		fn on_initialize(_n: T::BlockNumber) -> Weight {
			0
		}

		fn on_finalize(_n: T::BlockNumber) {
		}

		fn offchain_worker(_n: T::BlockNumber) {
		}

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)] // change me
		pub fn send_llm(origin: OriginFor<T>, to_account: T::AccountId, amount: T::Balance) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let receiver = to_account;
		//	let ai: T::AssetId = CORE_ASSET_ID;
			// Check account balance of llm for sender
			//ensure!(<pallet_assets::Pallet<T>>::balance(ai, &sender.into()) >= amount, Error::<T>::LowBalance);// ensure balance is more or the same as the amount
		//	<pallet_assets::Pallet<T>>:: force_transfer(origin, ai, sender, receiver, amount)?; // transfer llm
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn mint_llm(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			
			let minted_amount = <MintedAmount<T>>::get(); // Get the amount of llm minted so far
			//let allow_spend: u64 = T::Total_supply::get()-minted_amount*0.9; // 0.9% of the total supply minus the minted on is what we are allowed to spend per year 
			ensure!(minted_amount <= T::Total_supply::get(), Error::<T>::LowAmount); // ensure the amount of llm minted is less then the total supply
			todo!("Check the time limit");


			todo!("Mint using pallet assets");
		//	let min_balance: T::Balance = 0;
		//	let treasury: T::AccountId = PalletId(*b"py/trsry").into_account(); // treasury account
			//let asset_id: T::AssetId = CORE_ASSET_ID; // id of LLM 
			// if our asset doesnt exist yet, create it
			// Check the balance of the asset id 0, if it doesnt exist, create it
	
	/*
			let current_am = match <pallet_assets::Pallet<T>>::balance(asset_id, &treasury) {
				Some(balance) => {
				//	ensure!(balance >= min_balance, "Balance must be greater than or equal to the minimum balance");
					balance
				},
				None => {
					<pallet_assets::Pallet<T>>::force_create(origin,
						asset_id, // asset id
						treasury.into(), // treasury
						false, // is_sufficient
						min_balance, // minum amount that users need to have to send
						);
						<MintedAmount<T>>::mutate(|minted_amount| *minted_amount += min_balance);
						min_balance
				}
			};
*/
				// Mint allowed amount to the treasury
			//	<pallet_assets::Pallet<T>>::mint_into(asset_id, &treasury, allow_spend.into());
				// Add the amount that we have minted into MintedAmount to add allow_sped
			//	<MintedAmount<T>>::mutate(|minted_amount| *minted_amount += allow_spend);
		
			//	Self::deposit_event(RawEvent::MintLlm(sender, amount));
			Ok(())
		}

	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {

		MintedLLM(T::AccountId, T::Balance),
		TransferedLLM(T::AccountId, T::AccountId, T::Balance),
	}

	/*
	// The genesis config type.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
			Total_supply: T::balance,
	}

	// The default value for the genesis config type.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { minted_amount: Default::default(), Total_supply: Default::default() }
		}
	}

	// The build of genesis for the pallet.
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			
		//	<MintedAmount<T>>::put(&self.mint_llm::get()); //sync the minted amount
					
		}


	}
*/
}

//impl<T: Config> Pallet<T> {
	// Add public immutables and private mutables.
//	#[allow(dead_code)]
//	fn accumulate_foo(origin: T::Origin, increase_by: T::Balance) -> DispatchResult {
//		todo!("empty function");
//		Ok(())
//	}
//}


