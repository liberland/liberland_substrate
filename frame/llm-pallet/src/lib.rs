#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

/// Liberland Merit Pallet
/*
decrease the total supply with 0.9 % per year
mint 0.9% per year to the treasury with pallet assets

*/
//use frame_support::tokens::fungibles::AssetId;

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
	// Every year the pallet mints 0.9% of the total supply.
	//	use frame_support::traits::tokens::AssetId;
	//pub type AssetId = u64;
	use frame_system::Origin;

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	//	type AssetsWrapper: GetAsset<<Self as pallet_assets::Config>::AssetId>;
	//	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	//pub type AssetId = u64;//<T as pallet_assets::Config>::AssetId + u64; , u64 might to big?
	pub type Balance = u128;
	//pub type AssetId = <T as pallet_assets::Config>::AssetId;

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

	pub trait GetAsset<AssetId> {
		fn id(asset: &AssetId) -> Option<AssetId>;
	}

	#[pallet::config]
	pub trait Config: pallet_assets::Config + frame_system::Config {
		// include pallet asset config aswell

		type Total_supply: Get<u64>; // Pre defined the total supply in runtime
		type PreMintedAmount: Get<u64>; // Pre defined the total supply in runtime
		type Balances: Parameter
			//+ u128
			+ Get<u128>
			+ From<u128>;
		type AssetsWrapper: GetAsset<<Self as pallet_assets::Config>::AssetId>;
		type AssetId: IsType<<Self as pallet_assets::Config>::AssetId> + Parameter + Ord + Copy;
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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
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
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let receiver = to_account;
			//	let ai: T::AssetId = CORE_ASSET_ID;
			// Check account balance of llm for sender
			//ensure!(<pallet_assets::Pallet<T>>::balance(ai, &sender.into()) >= amount,
			// Error::<T>::LowBalance);// ensure balance is more or the same as the amount
			// 	<pallet_assets::Pallet<T>>:: force_transfer(origin, ai, sender, receiver, amount)?;
			// // transfer llm
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn mint_llm(origin: OriginFor<T>, assetid: AssetId<T>) -> DispatchResult
//	where <T as pallet_assets::Config>::Balance: From<i32>
		{
			let sender = ensure_signed(origin)?;

			let minted_amount: u64 = <MintedAmount<T>>::get(); // Get the amount of llm minted so far
			let treasury: T::AccountId = PalletId(*b"py/trsry").into_account();
			let maxcap: u64 = T::Total_supply::get();
			let t_balance: u64 =
				pallet_assets::Pallet::<T>::balance(assetid.into(), &treasury.into())
					.try_into()
					.unwrap_or(Default::default());
			let hardlimit: f64 = 0.9;
			let allow_spend: f64 = maxcap as f64 - minted_amount as f64 * hardlimit; // 0.9% of the total supply minus the minted on is what we are allowed to spend per year

			// ensure that we do not mint more tokens than the maxcap
			ensure!(t_balance < maxcap.into(), Error::<T>::MaxCap); // ensure the treasury balance is more or the same as the maxcap

			todo!("Check the time limit");
			ensure!(t_balance >= 1, "Treasury account does not have the asset in balance");
			let transfer_amount: T::Balance = 100u64.try_into().unwrap_or(Default::default()); // 100 llm? with u64 storage
			todo!("Mint using pallet assets");

			Self::mint_tokens(assetid, allow_spend as u64); // mint tokens with pallet assets

			// min_balance minus 500
			//let assit_id: T::AssetId =   //T::AssetId::decode(&mut
			// 0.into()).unwrap_or(Default::default()); set assetid to 100
			let test0 = assetid; //T::AssetId//::get(); //T::AssetsWrapper::id(0).ok_or(); // T::AssetId::decode(&mut
					 // 0).unwrap();//_or(Default::default()); treasury account

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
	impl<T: Config> Pallet<T> {
		// check if we are allowed to mint more llm
		fn check_limits(assetid: AssetId<T>) -> bool {
			let minted_amount: u64 = <MintedAmount<T>>::get(); // Get the amount of llm minted so far
												   // check if minted amount is less then the Total_supply
												   //	let allow_spend: u64 = maxcap-minted_amount; // 0.9% of the total supply minus the
												   // minted on is what we are allowed to spend per year

			//		let allow_spend: u64 = T::Total_supply::get()-minted_amount*0.9; // 0.9% of the total
			// supply minus the minted on is what we are allowed to spend per year 		if minted_amount
			// <= T::Total_supply::get() { 			return true;
			//		}
			//		else {
			return false
			//		}
		}

		/// Mint tokens to the treasury account.
		fn mint_tokens(assetid: AssetId<T>, amount: u64) {
			let transfer_amount: T::Balance = amount.try_into().unwrap_or(Default::default());
			let treasury: T::AccountId = PalletId(*b"py/trsry").into_account();

			// add the amount that we have minted into MintedAmount to add allow_sped
			<MintedAmount<T>>::mutate(|minted_amount| *minted_amount += amount);
			pallet_assets::Pallet::<T>::mint_into(assetid.into(), &treasury, transfer_amount)
				.unwrap();
			Event::<T>::MintedLLM(treasury, transfer_amount); // emit event
		}
	}
}
