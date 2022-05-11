#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

/// Liberland Merit Pallet
/*
decrease the total supply with 0.9 % per year
mint 0.9% per year to the treasury with pallet assets

Author: <filip@rustsyndi.cat>
Copyright © 2022 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#[frame_support::pallet]
pub mod pallet {
	// Import various types used to declare pallet in scope.
	use super::*;
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		sp_runtime::traits::AccountIdConversion,
		traits::fungibles::Mutate,
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_runtime::{generic::BlockId, SaturatedConversion};
	use sp_std::vec::Vec;
	use sp_runtime::traits::StaticLookup;
	// Every year the pallet mints 0.9% of the total supply.
	//	use frame_support::traits::tokens::AssetId;
	//pub type AssetId = u64;
	use frame_system::Origin;
	//use sp_runtime::traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member, StaticLookup,
	// Zero};

//	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	//	type AssetsWrapper: GetAsset<<Self as pallet_assets::Config>::AssetId>;
	//	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	//pub type AssetId = u64;//<T as pallet_assets::Config>::AssetId + u64; , u64 might to big?
	//	pub type Balance = u128;
	//pub type AssetId = <T as pallet_assets::Config>::AssetId;

	//	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
	pub struct MetaData<u64> {
		Reserve_am: u64,
		Minted_am: u64,
		BlockNumber: u64,
	}
	//use pallet_assets::Asset::AssetId; // ::AssetId;
	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo)]
	pub struct AssetInfo {
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	}
	//	pub const CORE_ASSET_ID: AssetId = 0;

// asset querys
	//	pub fn get_asset_meta_data(asset_id: AssetId) -> Option<MetaData<AssetId>> {
/*
		#[pallet::storage]
	#[pallet::getter(fn asset_store)]
	pub(super) type AssetStore<T: Config> = StorageMap<_, 
	Blake2_128Concat,
	u32, 
	AssetInfo, 
	ValueQuery>; // asset id, asset meta data
*/


	#[pallet::storage]
	#[pallet::getter(fn get_balance)]
	pub(super) type LLMBalance<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		ValueQuery
		>;

	#[pallet::storage]
	#[pallet::getter(fn minted_amount)]
	/// Keep track of the amount of minted llm
	pub(super) type MintedAmount<T: Config> = StorageValue<_, u64, ValueQuery>; // ValueQuery ,  OnEmpty = 0u64

// Guess we dont need this type of stuff since we use pallet-assets to keep track of things
//	#[pallet::storage]
//	#[pallet::getter(fn llm_balance_sheet)]
//	pub(super) type llm_balance_sheet<T: Config> = StorageMap<_, T::AccountId, u64, ValueQuery>; // ValueQuery ,  OnEmpty = 0u64

	//#[pallet::storage]
	//#[pallet::getter(fn meta_data)]
	/// Keep track of the block that we use to mint llm
	//pub(super) type MetaData<T: Config> = StorageValue<_, MetaData<T::Balance, BlockId>,
	// ValueQuery>;

//	pub trait GetAsset<AssetId> {
//		fn id(asset: &AssetId) -> Option<AssetId>;
//	}

	#[pallet::config]
	pub trait Config: pallet_assets::Config + frame_system::Config {
		// include pallet asset config aswell

		type Total_supply: Get<u64>; // Pre defined the total supply in runtime
		type PreMintedAmount: Get<u64>; // Pre defined the total supply in runtime

		//		type Balance: Member + Parameter + AtLeast32Bit + Default + Copy +
		// MaybeSerializeDeserialize;

		//		type AssetsWrapper: GetAsset<<Self as pallet_assets::Config>::AssetId>;
		type AssetId: IsType<<Self as pallet_assets::Config>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;
	//	type AccountId: Parameter + Member + Default + Copy; //AtLeast32Bit
		//type AssetId: IsType<<Self as pallet_assets::Config>::AssetId> + Parameter + Ord + Copy +
		// Into<u64> + From<u64> + Default; //GetAsset<<Self as pallet_assets::Config>::AssetId> +
		// type AssetId: IsType<<Self as pallet_assets::Config>::AssetId> + From<u64> + From<i32>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		//	type WeightInfo: WeightInfo;
	}

	pub type AssetId<T> = <T as Config>::AssetId;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid amount
		InvalidAmount,
		/// Balance is less then sending amount
		LowBalance,
		LowAmount,
		/// Not allowed to mint more
		MaxCap,
		/// Asset already created
		AssetExists,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			// convert blocknumber to u64
			//println!("LLM PAllet");
			
		//	let blocknumber = blocknumber.saturated_into::<u64>();
		//	Self::try_mint(blocknumber).unwrap_or_default();
			// todo: have a function that runs on blocknumber as input and checks if it can mint llm
			// based on blocknumber 	Self::mint_llm(origin);
			0
		}

		fn on_finalize(_n: T::BlockNumber) {}

		fn offchain_worker(_n: T::BlockNumber) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//type AssetId = T::AssetId;
		#[pallet::weight(10_000)] // change me
		pub fn send_llm(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: u64,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let receiver = to_account;
	//		pallet_assets::Pallet::<T>::transfer(
	//			origin, 
	//			llm_id,
	//			lookup_account,
	//			amount.into()
	//		)
			//	let ai: T::AssetId = CORE_ASSET_ID;
			// Check account balance of llm for sender
			//ensure!(<pallet_assets::Pallet<T>>::balance(ai, &sender.into()) >= amount,
			// Error::<T>::LowBalance);// ensure balance is more or the same as the amount
			// 	<pallet_assets::Pallet<T>>:: force_transfer(origin, ai, sender, receiver, amount)?;
			// // transfer llm
		//	<BalanceToAccount<T>>::insert(&to_account, amount);
			todo!("Transfer llm");
		}

		#[pallet::weight(10_000)] 
		pub fn send_locked_llm(origin: OriginFor<T>, amount: u64, to_account: T::AccountId) -> DispatchResult {
				todo!("send_locked_llm");
		}

		/// Lock in LLM for 
		#[pallet::weight(10_000)] // change me
		pub fn lock_llm(origin: OriginFor<T>, amount: u64) -> DispatchResult {
			todo!("lock_llm");
			// pallet freeze https://paritytech.github.io/substrate/master/pallet_assets/pallet/struct.Pallet.html#method.freeze
		}

		#[pallet::weight(10_000)] // change me
		pub fn unlock_llm(origin: OriginFor<T>) -> DispatchResult {
				todo!("unlock_llm"); // thaw
		}

		#[pallet::weight(10_000)] // change me
		pub fn createllm(origin: OriginFor<T>) -> DispatchResult {
		//	T::AddOrigin::ensure_origin(origin)?;
			let sender = ensure_signed(origin.clone())?;
			Self::create_llm(origin)?;
			
			Ok(())
		}


		#[pallet::weight(10_000)] // change me
		pub fn delegated_transfer(origin: OriginFor<T>, to_account: T::AccountId, amount: u64) -> DispatchResult {
// the senate must approve this transfer
			let sender = ensure_signed(origin)?;
			let receiver = to_account;
		//	pallet_assets::Pallet::<T>::delegated_transfer(
		//		origin,
		//		assetid,
		//		delegate_account,

		//	)
				todo!("delegated_transfer");
		}

		/// Allow the senate to approve transfers
		#[pallet::weight(10_000)] 
		pub fn approve_transfer(origin: OriginFor<T>, to_account: T::AccountId, amount: u64) -> DispatchResult {
				todo!("approve_transfer");
		}

		/// Display the minted amount of llm
//		#[pallet::weight(10_000)] // change me
//		pub fn llm_circulation(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
//			let sender = ensure_signed(origin)?;
//			let minted_amount: u64 = <MintedAmount<T>>::get();
//			Ok(())
//		}

		#[pallet::weight(10_000)]
		pub fn mint_llm(origin: OriginFor<T>) -> DispatchResult
//	where <T as pallet_assets::Config>::Balance: From<i32>
		{
			let sender = ensure_signed(origin)?;
			let assetid: AssetId<T> = 1u32.into();
			let minted_amount: u64 = <MintedAmount<T>>::get(); // Get the amount of llm minted so far
			let treasury: T::AccountId = PalletId(*b"py/trsry").into_account();
			let maxcap: u64 = T::Total_supply::get();
			let t_balance: u64 =
				pallet_assets::Pallet::<T>::balance(assetid.into(), &treasury.into())
					.try_into()
					.unwrap_or(0u64);
			let hardlimit: f64 = 0.9;
			let allow_spend: f64 = maxcap as f64 - minted_amount as f64 * hardlimit; // 0.9% of the total supply minus the minted on is what we are allowed to spend per year

			// ensure that we do not mint more tokens than the maxcap
			ensure!(t_balance < maxcap.into(), Error::<T>::MaxCap); // ensure the treasury balance is more or the same as the maxcap

			//todo!("Check the time limit");
			ensure!(t_balance >= 1, "Treasury account does not have the asset in balance");
			//	let transfer_amount: T::Balance = 100u64.try_into().unwrap_or(Default::default()); //
			// 100 llm? with u64 storage
			//	todo!("Mint using pallet assets");

			Self::mint_tokens(assetid, allow_spend as u64); // mint tokens with pallet assets
	//		<BalanceToAccount<T>>::insert(&sender, amount);
			// min_balance minus 500
			//let assit_id: T::AssetId =   //T::AssetId::decode(&mut
			// 0.into()).unwrap_or(Default::default()); set assetid to 100
		//	let test0 = assetid; //T::AssetId//::get(); //T::AssetsWrapper::id(0).ok_or(); // T::AssetId::decode(&mut
					 // 0).unwrap();//_or(Default::default()); treasury account

			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	//#[pallet::metadata(u64 = "Metadata")]
	pub enum Event<T: Config> {
		MintedLLM(T::AccountId, u64),
		TransferedLLM(T::AccountId, T::AccountId, u64),
		LLMCreated(T::AccountId, u64), // acountid, amount
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
	impl<T: Config> Pallet<T> {

		
		//type Balance = <T as pallet::Config>::Balance;
		// could do like a OriginFor<SenateGroup> or X(Tech) committee
		fn create_llm(origin: OriginFor<T>) -> DispatchResult {
			// create asset with pallet assets
			//	ensure_signed(origin.clone())?; // bad practise

			let assetid: AssetId<T> = 0u32.into();
			// check if asset is created

			let owner: T::AccountId = PalletId(*b"py/trsry").into_account(); // treasury is the owner
			let challenger_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(owner.clone());
			let t_ac2: T::AccountId = PalletId(*b"py/trsry").into_account();
			let asset_balance: u128 = pallet_assets::Pallet::<T>::balance(assetid.into(), t_ac2)
				.try_into()
				.unwrap_or(0u128);
			let minted_amount: u64 = <MintedAmount<T>>::get();
			ensure!(minted_amount == 0u64, Error::<T>::AssetExists); //minted should be zero
			ensure!(asset_balance == 0, Error::<T>::AssetExists); // if the asset balance is zero == that means it is not been created and we can create
													  // it set minum balance to 0
			let min_balance: T::Balance = 0u64.try_into().unwrap_or(Default::default()); // 0 llm
			let name: Vec<u8> = "Liberland Merit".into();
			let symbol: Vec<u8> = "LLM".into();
			let decimals: u8 = 12u8; //12 decimals 0.01 llm
			pallet_assets::Pallet::<T>::force_create(
				origin.clone(),
				assetid.into(),
				challenger_lookup,
				true,
				min_balance,
			).unwrap_or_default();
			let t_ac: T::AccountId = PalletId(*b"py/trsry").into_account();
			let my_amount: u64 = min_balance.try_into().unwrap_or(0u64);
			Event::<T>::LLMCreated(t_ac.clone(),my_amount);
		//	LLMBalance::<T>::insert::<T::AccountId, T::Balance>(t_ac, my_amount.try_into().unwrap_or_default());
			// set the asset's meta data
			pallet_assets::Pallet::<T>::force_set_metadata(
				origin.clone(),
				assetid.into(),
				name,
				symbol,
				decimals,
				false,
			).unwrap_or_default();
			Self::mint_tokens(assetid, T::PreMintedAmount::get()); // mint the preminted amount
			Ok(())
			// pre mint amount and freeze it
		}

		fn try_mint(block: u64) -> DispatchResult {
			// check if the asset is created
			/*
						let asset_created = pallet_assets::Pallet::<T>::is_created(assetid.into());
						ensure!(asset_created, Error::<T>::AssetNotCreated);
						// check if the asset is not frozen
						let asset_frozen = pallet_assets::Pallet::<T>::is_frozen(assetid.into());
						ensure!(!asset_frozen, Error::<T>::AssetFrozen);
						// check if the asset is not minted
						let asset_minted = pallet_assets::Pallet::<T>::is_minted(assetid.into());
						ensure!(!asset_minted, Error::<T>::AssetMinted);
						// check if the asset is not locked
						let asset_locked = pallet_assets::Pallet::<T>::is_locked(assetid.into());
						ensure!(!asset_locked, Error::<T>::AssetLocked);
						// check if the asset is not paused
						let asset_paused = pallet_assets::Pallet::<T>::is_paused(assetid.into());
						ensure!(!asset_paused, Error::<T>::AssetPaused);
						// check if the asset is not paused
						let asset_paused = pallet_assets::Pallet::<T>::is_paused(assetid.into());
						ensure!(!asset_paused, Error::<T>::AssetPaused);
						// check if the asset is not paused
						let asset_paused = pallet_assets::Pallet::<T>::is_paused(assetid.into());
						ensure!(!asset_paused, Error::<T>::AssetPaused);
			*/
			Ok(())
		}

		// get asset metadata from pallet-assets crate
	//	fn get_asset_metadata(assetid: AssetId<T>) -> Result<(Vec<u8>, Vec<u8>, u8), &'static str> {
	//			todo!("get_asset_metadata");
	//			let asset_metadata = pallet_assets::Pallet::<T>::get_metadata(assetid.into());
	//	}

	//	fn get_asset_meta_data(asset_id: AssetId<T>) -> AssetMetaData<T> {
	//		let asset_meta_data = <AssetMetaData<T>>::get(asset_id);
	//		todo!("get_asset_meta_data");
	//	}

		/// amount of blocks until date 
		fn timestamp(blockid: u64) -> u64 {
			//let block_date = <system::Module<T>>::block_number_to_date(blockid);
			todo!("timestamp");
		}

		/// Mint tokens to the treasury account.
		fn mint_tokens(assetid: AssetId<T>, amount: u64) {
			let transfer_amount: T::Balance = amount.try_into().unwrap_or(Default::default());
			let treasury: T::AccountId = PalletId(*b"py/trsry").into_account();
			// update balance of the treasury account, balances should be u128 and not u64
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(treasury.clone(), amount.try_into().unwrap_or_default());
			// add the amount that we have minted into MintedAmount to add allow_sped
			<MintedAmount<T>>::mutate(|minted_amount| *minted_amount += amount);
			
			pallet_assets::Pallet::<T>::mint_into(assetid.into(), &treasury, transfer_amount)
				.unwrap_or_default();
		//	Event::<T>::MintedLLM(treasury.into(), amount); // emit event
		}
	}
}
