#![recursion_limit = "256"]
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod benchmarking;
pub mod mock;
pub mod tests;
pub mod types;
pub mod weights;

pub use types::{ContractDataStorage, ContractIndex};
pub use weights::WeightInfo;

use frame_support::traits::{Currency, NamedReservableCurrency};

type BalanceOf<T, I> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;
	use sp_runtime::Saturating;

	type ReserveIdentifierOf<T, I> = <<T as Config<I>>::Currency as NamedReservableCurrency<
		<T as frame_system::Config>::AccountId,
	>>::ReserveIdentifier;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + Sized {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency for Deposits
		type Currency: NamedReservableCurrency<Self::AccountId>;

		/// Per Byte deposit for storing data
		#[pallet::constant]
		type ByteDeposit: Get<BalanceOf<Self, I>>;

		#[pallet::constant]
		/// Identifies reserves in Currency
		type ReserveIdentifier: Get<&'static ReserveIdentifierOf<Self, I>>;

		/// Base deposit for storing data
		#[pallet::constant]
		type BaseDeposit: Get<BalanceOf<Self, I>>;

		/// Maximum length of contract data.
		#[pallet::constant]
		type MaxContractContentLen: Get<u32>;

		/// Maximum signatures per contract
		#[pallet::constant]
		type MaxSignatures: Get<u32>;

		/// Origin from which a judge may be added
		type AddJudgeOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin from which the new typical action can be made.
		///
		/// The success variant is the account id of the submitter.
		type SubmitOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Maximum number of items reached.
		TooMany,
		/// Can not find contract with given id
		ContractNotFound,
		/// Contract is already signed by the caller
		AlreadySigned,
		/// Caller isn't a judge
		NotJudge,
		/// Caller isn't a party
		NotParty,
		/// Caller isn't creator
		NotCreator,
		/// Given contract is already in use
		ContractInUse,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A judge has been added.
		AddedJudge { judge: T::AccountId },
		/// A judge has been removed.
		RemovedJudge { judge: T::AccountId },
		/// Judge signed a contract
		JudgeSigned { contract_id: ContractIndex, signer: T::AccountId },
		/// Judge signed a contract
		PartySigned { contract_id: ContractIndex, signer: T::AccountId },
		/// Created new contract
		ContractCreated { contract_id: ContractIndex, creator: T::AccountId },
		/// Remove contract
		ContractRemoved { contract_id: ContractIndex },
	}

	/// Contracts storage containing a content of contracts
	///
	/// TWOX-NOTE: Safe, as increasing integer keys are safe.
	#[pallet::storage]
	#[pallet::getter(fn contracts)]
	pub type Contracts<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		ContractIndex,
		ContractDataStorage<
			T::MaxContractContentLen,
			T::MaxSignatures,
			T::AccountId,
			BalanceOf<T, I>,
		>,
		OptionQuery,
	>;

	/// Vec of parties signatures for each contract
	///
	/// TWOX-NOTE: Safe, as increasing integer keys are safe.
	#[pallet::storage]
	#[pallet::getter(fn parties_signatures)]
	pub type PartiesSignatures<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		ContractIndex,
		BoundedVec<T::AccountId, T::MaxSignatures>,
		OptionQuery,
	>;

	/// Vec of judge signatures for each contract
	///
	/// TWOX-NOTE: Safe, as increasing integer keys are safe.
	#[pallet::storage]
	#[pallet::getter(fn judge_signatures)]
	pub(super) type JudgesSignatures<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		ContractIndex,
		BoundedVec<T::AccountId, T::MaxSignatures>,
		OptionQuery,
	>;

	/// List of judges
	#[pallet::storage]
	#[pallet::getter(fn judges)]
	pub type Judges<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::AccountId, bool, ValueQuery>;

	/// The number of contracts that have been made so far.
	#[pallet::storage]
	#[pallet::getter(fn contracts_count)]
	pub type NextContractsId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, ContractIndex, ValueQuery>;

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Add judge
		///
		/// The dispatch origin of this call must be _Signed_
		///
		/// - `judge`: Account that will be added as one of a judges
		///
		/// Emits `AddedJudge`.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_judge())]
		pub fn add_judge(origin: OriginFor<T>, judge: T::AccountId) -> DispatchResult {
			T::AddJudgeOrigin::ensure_origin(origin)?;

			Judges::<T, I>::insert(&judge, true);

			Self::deposit_event(Event::AddedJudge { judge });
			Ok(())
		}

		/// Judge can sign contract
		///
		/// The dispatch origin of this call must be _Signed_
		///
		/// - `contract_id`: The `id` of a contract that should be signed
		///
		/// Emits `JudgeSigned`.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::judge_sign_contract())]
		pub fn judge_sign_contract(
			origin: OriginFor<T>,
			contract_id: ContractIndex,
		) -> DispatchResult {
			let who = T::SubmitOrigin::ensure_origin(origin)?;
			let is_judge = Judges::<T, I>::get(&who);
			ensure!(is_judge, Error::<T, I>::NotJudge);

			JudgesSignatures::<T, I>::try_mutate(
				contract_id,
				|signatures_list| -> DispatchResult { Self::sign_contract(&who, signatures_list) },
			)?;

			Self::deposit_event(Event::JudgeSigned { contract_id, signer: who });
			Ok(())
		}

		/// Any caller can create contract
		///
		/// The dispatch origin of this call must be _Signed_ and the sender must
		/// have funds to cover the deposit.
		///
		/// - `data`: A content of contract
		///
		/// Emits `ContractCreated`.
		///
		/// # <weight>
		/// - `O(S)`
		///   - where `S` data len
		/// # </weight>
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_contract(
			T::MaxContractContentLen::get() // S
		))]
		pub fn create_contract(
			origin: OriginFor<T>,
			data: BoundedVec<u8, T::MaxContractContentLen>,
			parties: BoundedVec<T::AccountId, T::MaxSignatures>,
		) -> DispatchResultWithPostInfo {
			let who = T::SubmitOrigin::ensure_origin(origin)?;

			let required_deposit = Self::calculate_deposit(&data);
			T::Currency::reserve_named(T::ReserveIdentifier::get(), &who, required_deposit)?;

			let index = Self::contracts_count();
			let data_len = data.len() as u32;
			Contracts::<T, I>::insert(
				index,
				ContractDataStorage::<
					T::MaxContractContentLen,
					T::MaxSignatures,
					T::AccountId,
					BalanceOf<T, I>,
				> {
					data,
					parties,
					creator: who.clone(),
					deposit: required_deposit,
				},
			);

			JudgesSignatures::<T, I>::insert(index, BoundedVec::default());
			PartiesSignatures::<T, I>::insert(index, BoundedVec::default());

			Self::deposit_event(Event::ContractCreated { contract_id: index, creator: who });
			Ok(Some(T::WeightInfo::create_contract(
				data_len, // S
			))
			.into())
		}

		/// Sign contract as party
		///
		/// The dispatch origin of this call must be _Signed_
		///
		/// - `contract_id`: The `id` of a contract that should be signed
		///
		/// Emits `PartySigned`.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::party_sign_contract())]
		pub fn party_sign_contract(
			origin: OriginFor<T>,
			contract_id: ContractIndex,
		) -> DispatchResult {
			let who = T::SubmitOrigin::ensure_origin(origin)?;

			let contracts =
				Contracts::<T, I>::get(contract_id).ok_or(Error::<T, I>::ContractNotFound)?;
			ensure!(contracts.parties.contains(&who), Error::<T, I>::NotParty);

			PartiesSignatures::<T, I>::try_mutate(
				contract_id,
				|signatures_list| -> DispatchResult { Self::sign_contract(&who, signatures_list) },
			)?;

			Self::deposit_event(Event::PartySigned { contract_id, signer: who });
			Ok(())
		}

		/// Remove judge
		///
		/// The dispatch origin of this call must be _Signed_
		///
		/// - `judge`: Account that will be added as one of a judges
		///
		/// Emits `RemovedJudge`.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::remove_judge())]
		pub fn remove_judge(origin: OriginFor<T>, judge: T::AccountId) -> DispatchResult {
			T::AddJudgeOrigin::ensure_origin(origin)?;

			Judges::<T, I>::remove(&judge);
			Self::deposit_event(Event::RemovedJudge { judge });
			Ok(())
		}

		/// Remove contract
		///
		/// The dispatch origin of this call must be _Signed_ and the sender
		/// will receive refunded deposit
		///
		/// - `contract_id`: ID of contract
		///
		/// Emits `ContractRemoved`.
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_contract())]
		pub fn remove_contract(origin: OriginFor<T>, contract_id: ContractIndex) -> DispatchResult {
			T::SubmitOrigin::ensure_origin(origin)?;

			let judges_signatures_len = JudgesSignatures::<T, I>::get(contract_id)
				.ok_or(Error::<T, I>::ContractNotFound)?
				.len();
			let parties_signatures_len = PartiesSignatures::<T, I>::get(contract_id)
				.ok_or(Error::<T, I>::ContractNotFound)?
				.len();
			let contract =
				Contracts::<T, I>::get(contract_id).ok_or(Error::<T, I>::ContractNotFound)?;

			ensure!(
				judges_signatures_len == 0 && parties_signatures_len == 0,
				Error::<T, I>::ContractInUse
			);

			T::Currency::unreserve_named(
				T::ReserveIdentifier::get(),
				&contract.creator,
				contract.deposit,
			);

			JudgesSignatures::<T, I>::remove(contract_id);
			PartiesSignatures::<T, I>::remove(contract_id);
			Contracts::<T, I>::remove(contract_id);

			Self::deposit_event(Event::ContractRemoved { contract_id });
			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn calculate_deposit(data: &BoundedVec<u8, T::MaxContractContentLen>) -> BalanceOf<T, I> {
			let data_len = data.encoded_size() as u32;
			let required_deposit = T::BaseDeposit::get()
				.saturating_add(T::ByteDeposit::get().saturating_mul(data_len.into()));
			required_deposit
		}

		fn sign_contract(
			who: &T::AccountId,
			signatures_list: &mut Option<BoundedVec<T::AccountId, T::MaxSignatures>>,
		) -> DispatchResult {
			let signatures = signatures_list.as_mut().ok_or(Error::<T, I>::ContractNotFound)?;
			let already_signed = signatures.contains(&who);
			ensure!(!already_signed, Error::<T, I>::AlreadySigned);

			signatures.try_push(who.clone()).map_err(|_| Error::<T, I>::TooMany)?;
			Ok(())
		}
	}
}
