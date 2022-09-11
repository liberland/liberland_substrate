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
	use sp_runtime::{generic::BlockId, traits::StaticLookup, SaturatedConversion};
	use sp_std::vec::Vec;
	// Every year the pallet mints 0.9% of the total supply.
	//	use frame_support::traits::tokens::AssetId;
	//pub type AssetId = u64;
	use frame_system::Origin;
	use hex_literal::hex;
	use scale_info::prelude::vec;
	use sp_runtime::{print, traits::Saturating, AccountId32};

	//use sp_runtime::traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member, StaticLookup,
	// Zero};

	//	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	//	type AssetsWrapper: GetAsset<<Self as pallet_assets::Config>::AssetId>;
	//	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	//pub type AssetId = u64;//<T as pallet_assets::Config>::AssetId + u64; , u64 might to big?
	//	pub type Balance = u128;
	//pub type AssetId = <T as pallet_assets::Config>::AssetId;

	//	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
	//pub struct MetaData<u64> {
	//	reserve_am: u64,
	//	minted_am: u64,
	//	block_number: u64,
	//}
	//use pallet_assets::Asset::AssetId; // ::AssetId;
	//#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo)]
	//pub struct AssetInfo {
	//	name: Vec<u8>,
	//	symbol: Vec<u8>,
	//	decimals: u8,
	//}
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
	pub(super) type LLMBalance<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	#[pallet::storage] // allocated in politics
	#[pallet::getter(fn get_politics_balance)]
	pub(super) type LLMPolitics<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	#[pallet::storage] // LLM that are frozen in the politics queue
	#[pallet::getter(fn get_politics_lock)]
	pub(super) type LLMPoliticsLock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_locked_llm)] //locked llm used for voting
	pub(super) type LockedLLM<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_fluf)] //locked llm used for voting
	pub(super) type Withdrawlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>; // account and blocknumber

	#[pallet::storage]
	#[pallet::getter(fn minted_amount)]
	/// Keep track of the amount of minted llm
	pub(super) type MintedAmount<T: Config> = StorageValue<_, u64, ValueQuery>; // ValueQuery ,  OnEmpty = 0u64

	#[pallet::storage]
	#[pallet::getter(fn minted_when)]
	/// block number for next llm mint
	pub(super) type NextMint<T: Config> = StorageValue<_, u64, ValueQuery>; // ValueQuery ,  OnEmpty = 0u64

	// Guess we dont need this type of stuff since we use pallet-assets to keep track of things
	//	#[pallet::storage]
	//	#[pallet::getter(fn llm_balance_sheet)]
	//	pub(super) type llm_balance_sheet<T: Config> = StorageMap<_, T::AccountId, u64, ValueQuery>;
	// // ValueQuery ,  OnEmpty = 0u64

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
	pub const LLM_PALLET_ID: PalletId = PalletId(*b"llm/trsy"); // lets give llm a unique pallet id and it's own treasury

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
		InvalidAccount,
		InvalidTransfer,
		// can not transfer asset
		InvalidAssetMove,
		// do not unfreeze more then 10% of the users total supply
		Over10percentage,
		/// not allowed to withdraw llm
		Gottawait,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(b: T::BlockNumber) -> Weight {
			// convert blocknumber to u64
			//println!("LLM PAllet");
			log::info!("LLM Pallet Checking block");
			let blocknumber = b.saturated_into::<u64>();
			let _didmint = Self::try_mint(blocknumber); //.unwrap_or(false);
			log::info!("LLM Pallet Checked block");

			// todo: have a function that runs on blocknumber as input and checks if it can mint llm
			// based on blocknumber 	Self::mint_llm(origin);
			0
		}

		fn on_finalize(_n: T::BlockNumber) {}

		fn offchain_worker(_n: T::BlockNumber) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//	/*
		#[pallet::weight(10_000)] // debug function, used for testing, DO NOT USE IN PROD, function mints llm to a raw account
						  // based on input
		pub fn fake_send(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let receiver = to_account.clone();
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(
				receiver,
				amount.try_into().unwrap_or_default(),
			);

			pallet_assets::Pallet::<T>::mint_into(Self::llm_id().into(), &to_account, amount)
				.unwrap_or_default();
			Ok(())
		}
		//*/
		//type AssetId = T::AssetId;
		#[pallet::weight(10_000)] // change me
		pub fn send_llm(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: u64,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			let lookup_account = T::Lookup::unlookup(to_account.clone());

			let receiver = to_account;

			let amount_balance: T::Balance =
				amount.try_into().map_err(|_| Error::<T>::InvalidAmount)?;

			// check that balance can cover it
			ensure!(LLMBalance::<T>::get(&sender) >= amount_balance, Error::<T>::InvalidAmount);

			let sender_balance: T::Balance = LLMBalance::<T>::get(&sender) - amount_balance;
			pallet_assets::Pallet::<T>::transfer(
				origin.clone(),
				Self::llm_id().into(),
				lookup_account,
				amount_balance.clone(),
			)
			.unwrap_or_default(); //.map_err(|_| Error::<T>::InvalidTransfer);//unwrap_or_default();

			//	LLMBalance::<T>::mutate(sender, |balance| *balance =
			// balance.saturating_sub(amount_balance));
			LLMBalance::<T>::mutate_exists(&sender, |b| *b = Some(sender_balance));
			let receiver_balance: T::Balance = LLMBalance::<T>::get(&receiver) + amount_balance;

			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(receiver.clone(), receiver_balance); // overwrite each time, its a bit computational heavy, this should be replaced with a
																						 // match statement
																						 /*
																							 LLMBalance::<T>::mutate(&sender, |balance| -> Result<(), Error<T>> {
																									 *balance = amount_balance;//balance
																										 //.saturating_sub(amount_balance);
																										 //.ok_or(Error::<T>::InvalidAmount)?;
																									 Ok(())
																								 }).map_err(|_| Error::<T>::InvalidAccount)?;
																						 */

			Event::<T>::TransferedLLM(sender, receiver, amount);

			/*
				match LLMBalance::<T>::get(&receiver) {
					Some(balance) => {
						LLMBalance::<T>::mutate(receiver, |balance| *balance = balance.saturating_add(amount_balance));
						Ok(())
					}
					None => {
						LLMBalance::<T>::insert::<T::AccountId, T::Balance>(receiver, amount.try_into().unwrap_or_default());
						return Ok(());
					},
				}
			*/

			Ok(())
		}

		/// allocate LLM for politics
		#[pallet::weight(10_000)]
		pub fn politics_lock(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(LLMBalance::<T>::get(&sender) >= amount, Error::<T>::LowBalance);
			// insert into llm politics balance
			if LLMPolitics::<T>::contains_key::<T::AccountId>(sender.clone()) {
				//>= 0u64.try_into().unwrap_or(Default::default())

				LLMPolitics::<T>::mutate_exists(&sender, |b| {
					*b = Some(amount + LLMPolitics::<T>::get(&sender))
				}); // dont overwrite it, append to balance
			} else {
				LLMPolitics::<T>::insert::<T::AccountId, T::Balance>(sender.clone(), amount); // lock in the amount
			}

			let llm_balance: T::Balance = LLMBalance::<T>::get(&sender) - amount;

			// update llm balance storage map
			Self::update_user_balance(sender, llm_balance);

			// transfer to llm to llm trsy
			Self::deposit_political_llm(origin, amount);

			Ok(())
		}

		/// unlock LLM from politics
		#[pallet::weight(10_000)]
		pub fn politics_unlock(origin: OriginFor<T>) -> DispatchResult {
			let sender: T::AccountId = ensure_signed(origin.clone())?;
			//	let ten = Self::u64_to_balance(10u64);
			// check if we have political locked LLM
			log::info!("unlock called");
			ensure!(
				LLMPolitics::<T>::contains_key::<T::AccountId>(sender.clone()),
				Error::<T>::InvalidAccount
			);
			let current_block_number: u64 =
				<frame_system::Pallet<T>>::block_number().try_into().unwrap_or(0u64);
			ensure!(current_block_number > Withdrawlock::<T>::get(&sender), Error::<T>::Gottawait);
			let ten_percent: T::Balance =
				Self::get_10_percent(Self::get_politics_balance(sender.clone()));
			log::info!("releasing 10% {:?}", ten_percent.clone());
			//let did = Self::withdraw_political_llm(origin.clone());
			let timelimit: u64 = Self::get_future_block();

			LLMPolitics::<T>::mutate_exists(&sender, |llm_balance| {
				*llm_balance = Some(LLMPolitics::<T>::get(&sender) - ten_percent)
			}); // llm_balance.unwrap_or(0) - ten_percent;

			//update users llm account
			LLMBalance::<T>::mutate_exists(&sender, |b| {
				*b = Some(LLMBalance::<T>::get(&sender) + ten_percent)
			});

			Withdrawlock::<T>::insert(&sender, timelimit);
			//
			Ok(())
		}

		/*	- TEST FUNCTION
				#[pallet::weight(10_000)] // change me
				pub fn unfreeze_llm(
					origin: OriginFor<T>,
					//	account: T::AccountId,
					amount: T::Balance,
				) -> DispatchResult {
					//	let root_org: OriginFor<T> = frame_system::RawOrigin::Signed(LLM_PALLET_ID.clone());
					let sender = ensure_signed(origin.clone())?;
					//	let or--ws-externaligin_for_pallet = OriginFor<T>::Signed(Self::llm_id().into());
					ensure!(Self::get_locked_llm(&sender) >= amount, Error::<T>::InvalidAmount);
					//let lookup_account = T::Lookup::unlookup(account.clone());
					pallet_assets::Pallet::<T>::transfer(
						frame_system::RawOrigin::Signed(Self::get_llm_account()).into(),
						Self::llm_id().into(),
						T::Lookup::unlookup(sender.clone()), //
						amount.clone(),
					)
					.unwrap_or_default();

					//	LLMBalance::<T>::mutate(sender, |balance| *balance =
					LockedLLM::<T>::remove::<T::AccountId>(sender.clone());

					LLMBalance::<T>::mutate_exists(&sender, |b| {
						*b = Some(LLMBalance::<T>::get(&sender) + amount)
					});

					Ok(())
				}
		*/

		/// Freeze X amount of LLM for a certain account
		#[pallet::weight(10_000)] //change me
		pub fn freeze_llm(
			origin: OriginFor<T>,
			account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			//	let root_org = frame_system::RawOrigin::Signed(LLM_PALLET_ID.clone());
			// check if sender is the same as palletid
			//	ensure!(sender == Self::get_llm_account(), Error::<T>::InvalidAccount);
			// ensure that we have balance
			ensure!(LLMBalance::<T>::get(&sender) >= amount, Error::<T>::InvalidAmount);
			let sender_balance: T::Balance = LLMBalance::<T>::get(&sender) - amount;
			//check if LockedLLM contains the sender account
			//todo: replace with match statement

			if LockedLLM::<T>::contains_key::<T::AccountId>(sender.clone()) {
				//>= 0u64.try_into().unwrap_or(Default::default())

				LockedLLM::<T>::mutate_exists(&sender, |b| {
					*b = Some(amount + LockedLLM::<T>::get(&sender))
				}); // dont overwrite it, append to balance
			} else {
				LockedLLM::<T>::insert::<T::AccountId, T::Balance>(account.clone(), amount); // lock in the amount
			}

			pallet_assets::Pallet::<T>::transfer(
				origin.clone(),
				Self::llm_id().into(),
				T::Lookup::unlookup(Self::get_llm_account()), //
				amount.clone(),
			)
			.unwrap_or_default(); //.map_err(|_| Error::<T>::InvalidTransfer);//unwrap_or_default();

			//	LLMBalance::<T>::mutate(sender, |balance| *balance =
			// balance.saturating_sub(amount_balance));
			LLMBalance::<T>::mutate_exists(&sender, |b| *b = Some(sender_balance));

			Ok(())
		}

		//	#[pallet::weight(10_000)]
		//	pub fn send_locked_llm(
		//		origin: OriginFor<T>,
		//		amount: u64,
		//		to_account: T::AccountId,
		//	) -> Dispa(

		//)tchResult {
		//		todo!("send_locked_llm");
		//	}

		/// Request a transfer from the treasury llm account to a certain account.
		#[pallet::weight(10_000)]
		pub fn treasury_llm_transfer(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: u64,
		) -> DispatchResult {
			//

			//	let legits: Vec<T::AccountId> = Vec![
			//		hex!["061a7f0a43e35d16f330e64c1a4e5000db4ba064fc3630cc4a9e2027899a5a6f"].
			// unchecked_into(), 	];
			//let rootman = frame_system::RawOrigin::Root.into();
			//	let testac: T::AccountId =
			// hex!["ca84c08a24d96f8702e3940ea3ed7255a19ef11ac6d0fee490120edb9d9eb25d"].into();

			//	let account33: Vec<AccountId32> = vec![
			//			hex!["ca84c08a24d96f8702e3940ea3ed7255a19ef11ac6d0fee490120edb9d9eb25d"].into(),
			//		];
			//let xc: T::AccountId = account32.clone().unwrap();

			let account_map: Vec<T::AccountId> = vec![
				Self::account_id32_to_accountid(
					hex!["061a7f0a43e35d16f330e64c1a4e5000db4ba064fc3630cc4a9e2027899a5a6f"].into(),
				), //F
				Self::account_id32_to_accountid(
					hex!["ca84c08a24d96f8702e3940ea3ed7255a19ef11ac6d0fee490120edb9d9eb25d"].into(),
				), // Multisig N + F
				Self::account_id32_to_accountid(
					hex!["41166026871ac7d5606352428247a161e2c88fb67e48f9e0c6331dbe906405d8"].into(),
				), // Multisig F + ALICE + BOB */
			];
			//let sender_signed = ensure_signed(origin)?;
			//			let actest: T::AccountId =
			// Self::account_id32_to_accountid(hex!["
			// 061a7f0a43e35d16f330e64c1a4e5000db4ba064fc3630cc4a9e2027899a5a6f"].into());
			// 		let mut newac: AccountId32 =
			// hex!["ca84c08a24d96f8702e3940ea3ed7255a19ef11ac6d0fee490120edb9d9eb25d"].into();
			// 		let mut init_account32 = AccountId32::as_ref(&newac);
			//		let init_account: T::AccountId = T::AccountId::decode(&mut
			// init_account32).unwrap();//_or(Error::<T>::InvalidAccount.into());
			let sender: T::AccountId = ensure_signed(origin)?;
			//			let navid = "b86373a2dff0a7b5741fd7e1857de41353fca3b924f14eae5f4c70d69e949150";
			// //5GEUDCyZrzPy1A6Kn288pHZFDtVhfYWvYmU1iTUPMg6YSVTE 			let pub_key =
			// "061a7f0a43e35d16f330e64c1a4e5000db4ba064fc3630cc4a9e2027899a5a6f";	//
			// 5CCi1rPi7cphC6iE9mWkYvbLf57b9N233nFG8hM5zjvYZpLi 	let accounts: T::AccountId =
			// T::AccountId::decode(&mut &navid.encode()[..]).unwrap(); 	let fluff: T::AccountId =
			// T::AccountId::decode(&mut &pub_key.encode()[..]).unwrap();

			ensure!(account_map.contains(&sender), Error::<T>::InvalidAccount);

			let treasuy_account: T::AccountId = PalletId(*b"py/trsry").into_account();
			let treasury_balance = LLMBalance::<T>::get(&treasuy_account.clone());
			let amount_balance = Self::u64_to_balance(amount);
			ensure!(treasury_balance >= amount_balance, Error::<T>::LowBalance);
			let lookup_account = T::Lookup::unlookup(to_account.clone());

			//let rootorg = frame_system::RawOrigin::Root.into();
			let treasury_origin = PalletId(*b"py/trsry");
			let user_balance: T::Balance = LLMBalance::<T>::get(&to_account) + amount_balance;
			let new_treasury_balance: T::Balance = treasury_balance - amount_balance.clone();
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(treasuy_account.clone()).into(), /* root origin,
				                                                                  * change me later */
				Self::llm_id().into(),
				lookup_account,
				amount_balance.clone(),
			)
			.unwrap_or_default();

			// update user balance
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(to_account.clone(), user_balance);

			// update treasury balance
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(
				treasuy_account.clone(),
				new_treasury_balance,
			);

			//			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(to_account.clone(),
			// user_balance); // overwrite each time, its a bit computational heavy, this should be
			// replaced with a

			Event::<T>::TransferedLLM(treasuy_account, to_account, amount);

			//		let t_balance: u64 =
			//		pallet_assets::Pallet::<T>::balance(Self::llm_id().into(), &treasury.into())
			//			.try_into()
			//			.unwrap_or(0u64);

			Ok(())
			//todo!("treasury_llm");
		}

		/// Lock in LLM for
		//		#[pallet::weight(10_000)] // change me
		//		pub fn lock_llm(origin: OriginFor<T>, amount: u64) -> DispatchResult {
		//			todo!("lock_llm");
		// pallet freeze https://paritytech.github.io/substrate/master/pallet_assets/pallet/struct.Pallet.html#method.freeze
		//		}

		//	#[pallet::weight(10_000)] // change me
		//	pub fn unlock_llm(origin: OriginFor<T>) -> DispatchResult {
		//		todo!("unlock_llm"); // thaw
		//	}

		/// Create LLM manually plus mint it to the treasury
		#[pallet::weight(10_000)] // change me
		pub fn createllm(origin: OriginFor<T>) -> DispatchResult {
			//	T::AddOrigin::ensure_origin(origin)?;
			let sender = ensure_signed(origin.clone())?;
			Self::create_llm(origin)?;

			Ok(())
		}

		//		#[pallet::weight(10_000)] // change me
		//		pub fn delegated_transfer(
		//			origin: OriginFor<T>,
		//			to_account: T::AccountId,
		//			amount: u64,
		//		) -> DispatchResult {
		// the senate must approve this transfer
		//			let sender = ensure_signed(origin)?;
		//			let receiver = to_account;
		//	pallet_assets::Pallet::<T>::delegated_transfer(
		//		origin,
		//		assetid,
		//		delegate_account,

		//	)
		//			todo!("delegated_transfer");
		//		}

		//		#[pallet::weight(10_000)] // change me, remove me in prod
		//		pub fn add_balance(origin: OriginFor<T>, amount: u128) -> DispatchResult {
		//			let sender: T::AccountId = ensure_signed(origin)?;
		//	let my_account: T::AccountId = origin.ac
		//			todo!("added checks to verify and asset transfer and balance changes mutate");
		//			Self::load_alice(amount, sender);
		//			Ok(())
		//		}

		/// Allow the senate to approve transfers
		#[pallet::weight(10_000)]
		pub fn approve_transfer(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: u64,
		) -> DispatchResult {
			let _x = amount;
			let _y = to_account;
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
			let assetid: AssetId<T> = Self::llm_id();
			let minted_amount: u64 = <MintedAmount<T>>::get(); // Get the amount of llm minted so far
			let treasury: T::AccountId = PalletId(*b"py/trsry").into_account();
			let maxcap: u64 = T::Total_supply::get();
			let t_balance: u64 =
				pallet_assets::Pallet::<T>::balance(Self::llm_id().into(), &treasury.into())
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
												//	let test0 = assetid; //T::AssetId//::get(); //T::AssetsWrapper::id(0).ok_or(); //
												// T::AssetId::decode(&mut 0).unwrap();//_or(Default::default()); treasury account

			Ok(())
		}
		//	*/
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	//#[pallet::metadata(u64 = "Metadata")]
	pub enum Event<T: Config> {
		/// New llm has been minted
		MintedLLM(T::AccountId, u64),
		/// sender, receiver, amount
		TransferedLLM(T::AccountId, T::AccountId, u64),
		/// New LLM has been created
		LLMCreated(T::AccountId, u64), // acountid, amount
		/// X amount of LLM has been unlocked
		LLMPoliticsLocked(T::AccountId, u64),
		/// sent to user account, amount
		LLMPoliticsUnlocked(T::AccountId, u64),
		/// freeze llm for politics
		LLMPoliticsFreeze(T::AccountId, u64),
		/// unfreeze llm for politics
		LLMPoliticsUnfreeze(T::AccountId, u64),
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
		fn account_id32_to_accountid(accountid32: AccountId32) -> T::AccountId {
			let mut init_account32 = AccountId32::as_ref(&accountid32);
			let init_account: T::AccountId = T::AccountId::decode(&mut init_account32).unwrap();
			init_account
		}

		pub fn has_llm_politics(sender: T::AccountId) -> bool {
			LLMPolitics::<T>::contains_key::<T::AccountId>(sender)
		}
		// get 10% of the users balance as a u64
		fn get_10_percent(balance: T::Balance) -> T::Balance {
			balance / Self::u64_to_balance(10u64)
		}

		fn sync_llm_per_block() -> bool {
			//	let current_block = frame_system::Pallet::<Runtime>::block_number();
			true
		}

		/// deposit llm into political pool
		fn deposit_political_llm(origin: OriginFor<T>, amount_balance: T::Balance) -> bool {
			//		let assetid: AssetId<T> = Self::llm_id();

			//	let llmpallet: T::AccountId = PalletId(*b"py/trsry").into_account();
			// llm pallet origin
			//	let llmpallet_origin: T::Origin = T::Origin::from(llmpallet);
			//	let root_org: OriginFor<T> = frame_system::RawOrigin::Signed(LLM_PALLET_ID.clone());

			pallet_assets::Pallet::<T>::transfer(
				origin.clone(),
				Self::llm_id().into(),
				T::Lookup::unlookup(Self::get_llm_account()), // send to llm/trsy account
				amount_balance.clone(),
			)
			.unwrap_or_default();
			true
		}

		/// withdraw 10% of the users balance with pallet assets transfer and LLMBalance update and
		/// triggering onchain events
		fn withdraw_political_llm(origin: OriginFor<T>) -> bool {
			let sender = ensure_signed(origin).unwrap_or(return false);
			//	let llmpallet: T::AccountId = PalletId(*b"py/trsry").into_account();
			//	let llmpallet_origin: T::Origin = T::Origin::from(llmpallet);
			//		let root_org: OriginFor<T> =
			// frame_system::RawOrigin::Signed(LLM_PALLET_ID.clone()).into();
			let challenger_lookup: <T::Lookup as StaticLookup>::Source =
				T::Lookup::unlookup(sender);

			//	let current_balance: T::Balance = ;
			let ten_percent: T::Balance =
				Self::get_10_percent(Self::get_politics_balance(sender.clone()));

			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(Self::get_llm_account()).into(),
				Self::llm_id().into(),
				challenger_lookup, //send to the sender
				ten_percent.clone(),
			)
			.unwrap_or_default();

			// update the llm politics balance
			LLMPolitics::<T>::mutate_exists(&sender, |llm_balance| {
				*llm_balance = Some(LLMPolitics::<T>::get(&sender) - ten_percent)
			}); // llm_balance.unwrap_or(0) - ten_percent;

			//update users llm account
			LLMBalance::<T>::mutate_exists(&sender, |b| {
				*b = Some(LLMBalance::<T>::get(&sender) + ten_percent)
			});

			//trigger the onchain events to update that llm has been moved and unlocked
			Event::<T>::TransferedLLM(
				Self::get_llm_account(),
				sender.clone(),
				Self::balance_to_u64(ten_percent),
			); //sender, receiver, amount
			Event::<T>::LLMPoliticsUnlocked(sender.clone(), Self::balance_to_u64(ten_percent));

			true
		}

		//type Balance = <T as pallet::Config>::Balance;
		// could do like a OriginFor<SenateGroup> or X(Tech) committee
		fn create_llm(origin: OriginFor<T>) -> DispatchResult {
			// create asset with pallet assets
			//	ensure_signed(origin.clone())?; // bad practise

			let assetid: AssetId<T> = Self::llm_id(); //0u32.into();
										  // check if asset is created

			let owner: T::AccountId = PalletId(*b"py/trsry").into_account(); // treasury is the owner
			let challenger_lookup: <T::Lookup as StaticLookup>::Source =
				T::Lookup::unlookup(owner.clone());
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
			)
			.unwrap_or_default();
			let t_ac: T::AccountId = PalletId(*b"py/trsry").into_account();
			let new_balance: T::Balance =
				T::PreMintedAmount::get().try_into().unwrap_or(Default::default());

			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(t_ac.clone(), new_balance);
			let my_amount: u64 = min_balance.try_into().unwrap_or(0u64);
			Event::<T>::LLMCreated(t_ac.clone(), my_amount);
			//	LLMBalance::<T>::insert::<T::AccountId, T::Balance>(t_ac,
			// my_amount.try_into().unwrap_or_default()); set the asset's meta data
			pallet_assets::Pallet::<T>::force_set_metadata(
				origin.clone(),
				assetid.into(),
				name,
				symbol,
				decimals,
				false,
			)
			.unwrap_or_default();
			Self::mint_tokens(assetid, T::PreMintedAmount::get()); // mint the preminted amount
			Ok(())
			// pre mint amount and freeze it
		}
		//GET LLM ID
		fn llm_id() -> AssetId<T> {
			1u32.into()
		}

		fn get_llm_account() -> T::AccountId {
			PalletId(*b"llm/trsy").into_account()
		}

		fn balance_to_u64(balance: T::Balance) -> u64 {
			balance.try_into().unwrap_or(0u64)
		}

		fn u64_to_balance(amount: u64) -> T::Balance {
			amount.try_into().unwrap_or(Default::default())
		}

		fn get_future_block() -> u64 {
			let mut block: u64 = 0u64;
			let current_block_number: u64 =
				<frame_system::Pallet<T>>::block_number().try_into().unwrap_or(0u64);
			let blocks_per_second: u64 = 6u64; // 6 seconds per block
			let one_minute: u64 = 60u64 / blocks_per_second;
			let one_day: u64 = one_minute * 60u64 * 24u64;
			let one_year: u64 = one_day * 365u64; //365.24
			block = current_block_number + 2u64 * one_minute ; // 2 minutes
			block
		}

		/// get the 0.9% of the amount we are able to mint
		fn get_allowed_spending() -> u64 {
			let minted_amount: u64 = <MintedAmount<T>>::get(); // Get the amount of llm minted so far
			let maxcap: u64 = T::Total_supply::get();
			let hardlimit: f64 = 0.9;
			let allow_spend: f64 = maxcap as f64 - minted_amount as f64 * hardlimit; // 0.9% of the total supply minus the minted on is what we are allowed to spend per year
			allow_spend as u64
		}

		fn try_mint(block: u64) -> bool {
			log::info!("try_mint called");
			if block == 1u64 {
				log::info!("block is less than two");
				let rootorg = frame_system::RawOrigin::Root.into();
				Self::create_llm(rootorg).unwrap_or_default();
				let nextblock = Self::get_future_block();
				log::info!("setting nextmint to {:?}", nextblock);
				NextMint::<T>::put(nextblock);
				return true
			}

			if block < NextMint::<T>::get() {
				log::info!("returning false {:?}", block);
				return false
			}
			log::info!("second pass");
			log::info!("Next mint is: {:?}", NextMint::<T>::get());
			log::info!("Minting llm!!");
			//	let blocks_per_second: u64 = 6u64;// 6 seconds per block
			//	let one_minute: u64 = 60u64 / blocks_per_second;
			//	let mut nextblock: u64 = 2u64 * one_minute; // 2 minutes
			//	nextblock += block;
			NextMint::<T>::put(Self::get_future_block());
			let treasuy_account: T::AccountId = PalletId(*b"py/trsry").into_account();
			// check if the asset is created

			// mint 0.9%
			let zeronine: u64 = Self::get_allowed_spending();
			Self::mint_tokens(0.into(), zeronine);
			Event::<T>::MintedLLM(treasuy_account.into(), zeronine);

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
			log::info!("try_mint ran all the way");
			true
		}

		// get asset metadata from pallet-assets crate
		//	fn get_asset_metadata(assetid: AssetId<T>) -> Result<(Vec<u8>, Vec<u8>, u8), &'static
		// str> { 			todo!("get_asset_metadata");
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

		/// used for testing, DO NOT USE IN PRODUCTION
		fn load_alice(amount: u128, user: T::AccountId) {
			//let alice: T::AccountId = T::AccountId::from_raw([0u8; 32].into());

			//change balance of alice
			//	LLMBalance::<T>::try_mutate(account, |balance| {
			//		*balance = amount.try_into().unwrap_or(0u64);
			//		Ok(())
			//	  });

			// add check if alice exist
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(
				user,
				amount.try_into().unwrap_or_default(),
			);
		}

		fn update_user_balance(useraccount: T::AccountId, new_balance: T::Balance) {
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(useraccount, new_balance);
		}

		/// Mint tokens to the treasury account.
		fn mint_tokens(assetid: AssetId<T>, amount: u64) {
			let transfer_amount: T::Balance = amount.try_into().unwrap_or(Default::default());
			let treasury: T::AccountId = PalletId(*b"py/trsry").into_account();
			// update balance of the treasury account, balances should be u128 and not u64
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(
				treasury.clone(),
				LLMBalance::<T>::get(&treasury) + amount.try_into().unwrap_or_default(),
			);
			// add the amount that we have minted into MintedAmount to add allow_sped
			<MintedAmount<T>>::mutate(|minted_amount| *minted_amount += amount);

			pallet_assets::Pallet::<T>::mint_into(assetid.into(), &treasury, transfer_amount)
				.unwrap_or_default();
			//	Event::<T>::MintedLLM(treasury.into(), amount); // emit event
		}
	}
}
