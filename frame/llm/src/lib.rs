#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
pub mod traits;


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
	use super::traits::LLM;
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		sp_runtime::traits::AccountIdConversion,
		traits::fungibles::Mutate,
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_runtime::{traits::StaticLookup, SaturatedConversion};
	use sp_std::vec::Vec;
	use hex_literal::hex;
	use scale_info::prelude::vec;
	use sp_runtime::AccountId32;

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
	#[pallet::getter(fn politi_pooled_amount)]
	/// Keep track of the amount of politi pooled llm
	pub(super) type PolitiPooledAmount<T: Config> = StorageValue<_, u64, ValueQuery>; // ValueQuery ,  OnEmpty = 0u64

	#[pallet::storage]
	#[pallet::getter(fn minted_when)]
	/// block number for next llm mint
	pub(super) type NextMint<T: Config> = StorageValue<_, u64, ValueQuery>; // ValueQuery ,  OnEmpty = 0u64

	#[pallet::storage]
	pub(super) type Electionlock<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>; // account and blocknumber

	#[pallet::config]
	pub trait Config: pallet_assets::Config + frame_system::Config + pallet_identity::Config {
		// include pallet asset config aswell

		type TotalSupply: Get<u64>; // Pre defined the total supply in runtime
		type PreMintedAmount: Get<u64>; // Pre defined the total supply in runtime

		type AssetId: IsType<<Self as pallet_assets::Config>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
		/// No politcal LLM allocated tokens
		NoPolLLM,
		/// Not a Citizen
		NonCitizen,
		/// Temporary locked after unpooling LLM
		Locked,
		InsufficientLLM,
		InvalidBalanceType,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(b: T::BlockNumber) -> Weight {
			log::info!("LLM Pallet Checking block");
			// convert blocknumber to u64
			let blocknumber = b.saturated_into::<u64>();
			let _didmint = Self::try_mint(blocknumber);
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
		#[pallet::weight(10_000)] // debug function, used for testing, DO NOT USE IN PROD, function mints llm to a raw account
						  // based on input
		pub fn fake_send(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let receiver = to_account.clone();
			//TODO FIXME this sets LLM, not add
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(
				receiver,
				amount.try_into().unwrap_or_default(),
			);

			pallet_assets::Pallet::<T>::mint_into(Self::llm_id().into(), &to_account, amount)
				.unwrap_or_default();
			Ok(())
		}

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
			.unwrap_or_default();

			LLMBalance::<T>::mutate_exists(&sender, |b| *b = Some(sender_balance));
			let receiver_balance: T::Balance = LLMBalance::<T>::get(&receiver) + amount_balance;

			// overwrite each time, its a bit computational heavy, this should be replaced with a match statement
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(receiver.clone(), receiver_balance);

			Event::<T>::TransferedLLM(sender, receiver, amount);

			Ok(())
		}

		/// allocate LLM for politics
		#[pallet::weight(10_000)]
		pub fn politics_lock(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(LLMBalance::<T>::get(&sender) >= amount, Error::<T>::LowBalance);
			// insert into llm politics balance
			if LLMPolitics::<T>::contains_key::<T::AccountId>(sender.clone()) {
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
			Self::add_politi_pooled_stats(amount.try_into().unwrap_or(0u64));

			Ok(())
		}

		/// unlock LLM from politics
		#[pallet::weight(10_000)]
		pub fn politics_unlock(origin: OriginFor<T>) -> DispatchResult {
			let sender: T::AccountId = ensure_signed(origin.clone())?;
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
			let timelimit: u64 = Self::get_future_block();

			LLMPolitics::<T>::mutate_exists(&sender, |llm_balance| {
				*llm_balance = Some(LLMPolitics::<T>::get(&sender) - ten_percent)
			});

			//update users llm account
			LLMBalance::<T>::mutate_exists(&sender, |b| {
				*b = Some(LLMBalance::<T>::get(&sender) + ten_percent)
			});

			Withdrawlock::<T>::insert(&sender, timelimit);
			Electionlock::<T>::insert(&sender, timelimit);
			Self::substract_politi_pooled_stats(ten_percent.try_into().unwrap_or(0u64));

			Ok(())
		}

		/// Freeze X amount of LLM for a certain account
		#[pallet::weight(10_000)] //change me
		pub fn freeze_llm(
			origin: OriginFor<T>,
			account: T::AccountId,
			amount: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			// ensure that we have balance
			ensure!(LLMBalance::<T>::get(&sender) >= amount, Error::<T>::InvalidAmount);
			let sender_balance: T::Balance = LLMBalance::<T>::get(&sender) - amount;
			//check if LockedLLM contains the sender account
			//todo: replace with match statement

			if LockedLLM::<T>::contains_key::<T::AccountId>(sender.clone()) {
				LockedLLM::<T>::mutate_exists(&sender, |b| {
					*b = Some(amount + LockedLLM::<T>::get(&sender))
				}); // dont overwrite it, append to balance
			} else {
				LockedLLM::<T>::insert::<T::AccountId, T::Balance>(account.clone(), amount); // lock in the amount
			}

			pallet_assets::Pallet::<T>::transfer(
				origin.clone(),
				Self::llm_id().into(),
				T::Lookup::unlookup(Self::get_llm_account()),
				amount.clone(),
			)
			.unwrap_or_default();

			LLMBalance::<T>::mutate_exists(&sender, |b| *b = Some(sender_balance));

			Ok(())
		}

		/// Request a transfer from the treasury llm account to a certain account.
		#[pallet::weight(10_000)]
		pub fn treasury_llm_transfer(
			origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: u64,
		) -> DispatchResult {
			let account_map: Vec<T::AccountId> = vec![
				Self::account_id32_to_accountid(
					hex!["db93a8bc25102cb5c7392cbcc1b0837ece2c5f24436124522feb9bd6010bf780"].into(),
				), //5H2cD1Q8ZkC5gwBWX2sViwtbE4yr3chSh84NeW4Hnz43VX76 , V + DEVKEY + N + M
				//	Self::account_id32_to_accountid(
				//		hex!["41166026871ac7d5606352428247a161e2c88fb67e48f9e0c6331dbe906405d8"].into(),
				//	), // Multisig F + ALICE + BOB */
			];
			let sender: T::AccountId = ensure_signed(origin)?;

			ensure!(account_map.contains(&sender), Error::<T>::InvalidAccount);

			let treasury_account: T::AccountId = PalletId(*b"py/trsry").into_account();
			let treasury_balance = LLMBalance::<T>::get(&treasury_account.clone());
			let amount_balance = Self::u64_to_balance(amount);
			ensure!(treasury_balance >= amount_balance, Error::<T>::LowBalance);
			let lookup_account = T::Lookup::unlookup(to_account.clone());

			let user_balance: T::Balance = LLMBalance::<T>::get(&to_account) + amount_balance;
			let new_treasury_balance: T::Balance = treasury_balance - amount_balance.clone();
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(treasury_account.clone()).into(), /* root origin,
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
				treasury_account.clone(),
				new_treasury_balance,
			);

			Event::<T>::TransferedLLM(treasury_account, to_account, amount);

			Ok(())
		}

		/// Create LLM manually plus mint it to the treasury
		#[pallet::weight(10_000)] // change me
		pub fn createllm(origin: OriginFor<T>) -> DispatchResult {
			//	T::AddOrigin::ensure_origin(origin)?;
			ensure_signed(origin.clone())?;
			Self::create_llm(origin)?;

			Ok(())
		}

		/// Allow the senate to approve transfers
		#[pallet::weight(10_000)]
		pub fn approve_transfer(
			_origin: OriginFor<T>,
			to_account: T::AccountId,
			amount: u64,
		) -> DispatchResult {
			let _x = amount;
			let _y = to_account;
			todo!("approve_transfer");
		}

		#[pallet::weight(10_000)]
		pub fn mint_llm(origin: OriginFor<T>) -> DispatchResult
		{
			ensure_signed(origin)?;
			let assetid: AssetId<T> = Self::llm_id();
			let minted_amount: u64 = <MintedAmount<T>>::get(); // Get the amount of llm minted so far
			let treasury: T::AccountId = PalletId(*b"py/trsry").into_account();
			let maxcap: u64 = T::TotalSupply::get();
			let t_balance: u64 =
				pallet_assets::Pallet::<T>::balance(Self::llm_id().into(), &treasury.into())
					.try_into()
					.unwrap_or(0u64);
			let hardlimit: f64 = 0.9;
			let allow_spend: f64 = maxcap as f64 - minted_amount as f64 * hardlimit; // 0.9% of the total supply minus the minted on is what we are allowed to spend per year

			// ensure that we do not mint more tokens than the maxcap
			ensure!(t_balance < maxcap.into(), Error::<T>::MaxCap); // ensure the treasury balance is more or the same as the maxcap

			// TODO Check the time limit
			ensure!(t_balance >= 1, "Treasury account does not have the asset in balance");

			Self::mint_tokens(assetid, allow_spend as u64); // mint tokens with pallet assets

			Ok(())
		}
	}

	#[pallet::event]
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

		/// deposit llm into political pool
		fn deposit_political_llm(origin: OriginFor<T>, amount_balance: T::Balance) -> bool {
			pallet_assets::Pallet::<T>::transfer(
				origin.clone(),
				Self::llm_id().into(),
				T::Lookup::unlookup(Self::get_llm_account()), // send to llm/trsy account
				amount_balance.clone(),
			)
			.unwrap_or_default();
			true
		}

		// could do like a OriginFor<SenateGroup> or X(Tech) committee
		fn create_llm(origin: OriginFor<T>) -> DispatchResult {
			// create asset with pallet assets

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
			pallet_assets::Pallet::<T>::force_set_metadata(
				origin.clone(),
				assetid.into(),
				name,
				symbol,
				decimals,
				false,
			)
			.unwrap_or_default();

			// Mint the rest of the tokens into the llm/vault
			let vaultac: T::AccountId = Self::get_llm_vault_account();

			let money_left: T::Balance = T::TotalSupply::get().try_into().unwrap_or(Default::default());
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(vaultac.clone(), money_left.clone());
			pallet_assets::Pallet::<T>::mint_into(assetid.into().clone(), &vaultac, money_left)?;


			Self::mint_tokens(assetid, T::PreMintedAmount::get()); // mint the preminted amount
			Ok(())
			// pre mint amount and freeze it
		}
		//GET LLM ID
		fn llm_id() -> AssetId<T> {
			1u32.into()
		}


		fn get_llm_vault_account() -> T::AccountId {
			PalletId(*b"llm/safe").into_account()
		}

		fn get_llm_account() -> T::AccountId {
			PalletId(*b"llm/trsy").into_account()
		}

		fn u64_to_balance(amount: u64) -> T::Balance {
			amount.try_into().unwrap_or(Default::default())
		}

		fn get_future_block() -> u64 {
			let current_block_number: u64 =
				<frame_system::Pallet<T>>::block_number().try_into().unwrap_or(0u64);
			let blocks_per_second: u64 = 6u64; // 6 seconds per block
			let one_minute: u64 = 60u64 / blocks_per_second;
			let one_day: u64 = one_minute * 60u64 * 24u64;
			let _one_year: u64 = one_day * 365u64; //365.24
			let block = current_block_number + 2u64 * one_minute ; // 2 minutes
			block
		}

		/// get the 0.9% of the amount we are able to mint
		fn get_allowed_spending() -> u64 {
			let minted_amount: u64 = <MintedAmount<T>>::get(); // Get the amount of llm minted so far
			let maxcap: u64 = T::TotalSupply::get();
			let hardlimit: f64 = 0.9;
			let allow_spend: f64 = maxcap as f64 - minted_amount as f64 * hardlimit; // 0.9% of the total supply minus the minted on is what we are allowed to spend per year
			allow_spend as u64
		}

		fn try_mint(block: u64) -> bool {
			if block == 1u64 {
				let rootorg = frame_system::RawOrigin::Root.into();
				Self::create_llm(rootorg).unwrap_or_default();
				let nextblock = Self::get_future_block();
				NextMint::<T>::put(nextblock);
				return true
			}

			if block < NextMint::<T>::get() {
				return false
			}
			NextMint::<T>::put(Self::get_future_block());
			let treasury_account: T::AccountId = PalletId(*b"py/trsry").into_account();

			// mint 0.9%
			let zeronine: u64 = Self::get_allowed_spending();
			Self::mint_tokens(0.into(), zeronine);
			Event::<T>::MintedLLM(treasury_account.into(), zeronine);

			log::info!("try_mint ran all the way");
			true
		}

		fn update_user_balance(useraccount: T::AccountId, new_balance: T::Balance) {
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(useraccount, new_balance);
		}

		/// Mint tokens to the treasury account. Sends tokens from the llm/vault to the treasury
		fn mint_tokens(_assetid: AssetId<T>, amount: u64) {
			let transfer_amount: T::Balance = amount.try_into().unwrap_or(Default::default());
			let treasury: T::AccountId = PalletId(*b"py/trsry").into_account();
			// update balance of the treasury account, balances should be u128 and not u64
			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(
				treasury.clone(),
				LLMBalance::<T>::get(&treasury) + amount.try_into().unwrap_or_default(),
			);

			// deduct from the vault

			LLMBalance::<T>::insert::<T::AccountId, T::Balance>(
				Self::get_llm_vault_account(),
				LLMBalance::<T>::get(&treasury) + amount.try_into().unwrap_or_default(),
			);

			// add the amount that we have minted into MintedAmount to add allow_sped
			<MintedAmount<T>>::mutate(|minted_amount| *minted_amount += amount);

			let vlookup: <T::Lookup as StaticLookup>::Source =
				T::Lookup::unlookup(Self::get_llm_vault_account());
			let rootorg: OriginFor<T> = frame_system::RawOrigin::Root.into();
			// transfer from the vault to the treasury
			pallet_assets::Pallet::<T>::transfer(
				rootorg.clone(),
				Self::llm_id().into(),
				vlookup,
				transfer_amount.clone(),
			)
			.unwrap_or_default();
		}

		fn add_politi_pooled_stats(amount: u64) {
			<PolitiPooledAmount<T>>::mutate(|politi_pooled_amount| *politi_pooled_amount += amount);
		}

		fn substract_politi_pooled_stats(amount: u64) {
			<PolitiPooledAmount<T>>::mutate(|politi_pooled_amount| *politi_pooled_amount -= amount);
		}
	}

	impl<T: Config> traits::LLM<T::AccountId, T::Balance> for Pallet<T> {
		fn check_pooled_llm(account: &T::AccountId) -> bool {
			LLMPolitics::<T>::contains_key(account)
		}

		fn is_election_unlocked(account: &T::AccountId) -> bool {
			if Electionlock::<T>::contains_key(account) {
				let current_block_number: u64 =
						<frame_system::Pallet<T>>::block_number().try_into().unwrap_or(0u64);
				let unlocked_on_block = Electionlock::<T>::get(account);
				return current_block_number >= unlocked_on_block;
			}
			true
		}

		fn get_politi_pooled_amount() -> u64 {
			PolitiPooledAmount::<T>::get()
		}

		fn get_llm_politics(account: &T::AccountId) -> T::Balance {
			LLMPolitics::<T>::get(account)
		}
	}

	impl<T: Config> Pallet<T> {
		fn is_known_good(account: &T::AccountId) -> bool {
			match pallet_identity::Pallet::<T>::identity(account) {
				Some(reg) => reg.info.citizen != pallet_identity::Data::None &&
							reg.judgements.contains(&(0u32, pallet_identity::Judgement::KnownGood)),
				None => false,
			}
		}
	}

	impl<T: Config> traits::CitizenshipChecker<T::AccountId> for Pallet<T> {

		fn ensure_democracy_allowed(account: &T::AccountId) -> Result<(), DispatchError> {
			ensure!(Self::is_known_good(account), Error::<T>::NonCitizen);
			ensure!(Self::is_election_unlocked(account), Error::<T>::Locked);
			ensure!(Self::check_pooled_llm(account), Error::<T>::NoPolLLM);
			Ok(())
		}

		fn ensure_elections_allowed(account: &T::AccountId) -> Result<(), DispatchError> {
			ensure!(Self::is_known_good(account), Error::<T>::NonCitizen);
			ensure!(Self::is_election_unlocked(account), Error::<T>::Locked);
			Ok(())
		}

	}
}
