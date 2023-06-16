//! # Liberland Federated Bridge Pallet
//!
//! ## Overview
//!
//! Federated Bridge Pallet is a part of Substrate <-> ETH bridge designed for
//! bridging LLM and LLD tokens to Ethereum network.
//!
//! ## Terminology
//!
//! * Bridge - complete system for transferring LLD/LLM to/from ETH, consisting of the bridge
//!   pallet, relays, watchers and bridge contract.
//! * Bridge pallet - part of Substrate chain - handles locking and unlocking
//! * Bridge contract - contract deployed on ETH - handles minting and burning
//! * Mint/burn - create/destroy wrapped token on ETH side
//! * Lock/unlock - lock LLD/LLM on substrate side while it's wrapped in tokens on ETH side.
//! * Stop - state in which bridge stops all actions
//! * Federation - set of relays, capable of minting and unlocking.
//! * Relay - offchain software that monitors locks on substrate and burns on eth and relays them to
//!   the other chain
//! * Watcher - monitors if relays don't misbehave - has right to stop bridge at any time
//! * Admin - someone with rights to stop/resume bridge
//! * Superadmin - account with rights to add/remove relay rights
//!
//! ## Bridge configuration
//!
//! ### Option 1: genesis / chainspec
//!
//! If you're starting a new chain, preconfigure the bridge using chain spec.
//! See the GenesisConfig struct for details.
//!
//! ### Option 2: using ForceOrigin
//!
//! If you're chain is running, but you have some trusted chain authority (like sudo key):
//! * set the ForceOrigin in runtime config
//! * using ForceOrigin call `set_admin` and `set_super_admin`
//! * using SuperAdmin call `add_relay` for all your relays
//! * using SuperAdmin call `set_votes_required`
//! * using SuperAdmin or Admin call `add_watcher` for all your watchers
//! * using SuperAdmin or Admin call `set_fee`
//! * using SuperAdmin or Admin call `set_state(Active)`
//!
//! ### Option 3: migration
//!
//! Write migration that will run on Runtime Upgrade that set everythin you
//! need. See GenesisBuild implementation for example.
//!
//! ## Typical ETH -> Substrate transfer flow
//!
//! 1. User deposits wrapped tokens to eth contract, a Receipt is issued as an event
//! 2. Relays see the Receipt event and relay it to bridge using `vote_withdraw` call
//! 3. User waits until `VotesRequired` votes are cast
//! 4. User calls `withdraw` on the bridge.
//!
//! ## Typical Substrate -> ETH transfer flow
//!
//! 1. User deposits native LLM/LLD to to Bridge pallet using `deposit` call, a
//!    Receipt is issued as an event
//! 2. Relays see the Receipt event and relay it to Ethereum contract
//! 3. User waits until required number of votes are cast by relays on Ethereum side
//! 4. User calls `withdraw` on the etherum side.
//!
//! ## Economics
//!
//! Running relays requires paying for resource use (CPU, storage, network) and
//! both Substrate and Ethereum network fees. As such, they're rewarded on each
//! successful withdrawal by user.
//!
//! The fee charged to user on withdrawal is set with `set_fee` call and can be
//! queried from pallet's storage.
//!
//! This fee is divided equally between relays that actually cast vote on given
//! transfer.
//!
//! ## Security
//!
//! Following security features are implemented substrate-side:
//! * bridge is stopped if 2 relays claim different details about given Receipt (different amount,
//!   recipient etc)
//! * bridge can be stopped by a watcher at any time
//! * bridge doesn't mint any new funds - only funds stored in bridge (a.k.a. existing as wrapped
//!   tokens on Eth side) are at risk
//! * bridge enforces rate-limit on withdrawals
//! * bridge limits how many tokens can be locked (bridged) at the same time
//! * there's a delay between approval of withdrawal and actually allowing it
//!
//! ## Pallet Config
//!
//! * `Currency` - Currency in which Fee will be charged (should be the same as curreny of fees for
//!   extrinsics)
//! * `Token` - fungible implementing Transfer trait that is bridged between networks
//! * `PalletId` - used to derive AccountId for bridge wallet
//! * `MaxRelays`
//! * `MaxWatchers`
//! * `MaxTotalLocked` - maximum amount of tokens that can be stored in bridges wallet. Deposits
//!   will start failing after this is reached.
//! * `WithdrawalDelay` - number of blocks between transfer approval and actually allowing it
//! * `WithdrawalRateLimit` - rate limit parameters
//! * `ForceOrigin` - origin that's authorized to set admin and super admin
//!
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `deposit`: Entrypoint for starting substrate -> eth transfer
//! * `vote_withdraw`: Call for relays to vote on approving eth -> substrate transfers
//! * `withdraw`: Call for users to finish approved eth -> substrate transfer
//! * `set_fee`: Set withdrawal fee
//! * `set_votes_required`: Set number of votes required to approve eth -> substrate transfer
//! * `add_relay`: Adds new relay
//! * `remove_relay`: Removes relay
//! * `add_watcher`: Adds new watcher
//! * `remove_watcher`: Removes watcher
//! * `set_state`: Allows stopping/resuming bridge
//! * `emergency_stop`: Call for Watchers to stop bridge in case of detected abuse
//! * `set_admin`: Sets new admin
//! * `set_super_admin`: Sets new super admin
//!
//!
//! License: MIT
/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::Currency, PalletId};
use frame_system::pallet_prelude::*;
pub use pallet::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

mod mock;
mod tests;

pub type EthAddress = [u8; 20];
pub type ReceiptId = [u8; 32];
pub type EthBlockNumber = u64;

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
/// Struct holding information about ETH -> Substrate transfer
pub struct IncomingReceipt<AccountId, Balance> {
	/// Eth block number at which Receipt event was emitted
	pub eth_block_number: EthBlockNumber,
	/// Account on Substrate side that should receive tokens
	pub substrate_recipient: AccountId,
	/// Amount of tokens transferred through bridge
	pub amount: Balance,
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
/// Status of ETH -> Substrate transfer being handled
pub enum IncomingReceiptStatus<BlockNumber> {
	/// Relays can vote on releasing this to recipient
	Voting,
	/// Relays casted required number of votes on block number
	Approved(BlockNumber),
	/// Funds already withdrawn to recipient on block number
	Processed(BlockNumber),
}

impl<T> Default for IncomingReceiptStatus<T> {
	fn default() -> Self {
		Self::Voting
	}
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
/// Bridge state:
/// * Stopped - emergency state, block all actions pending manual review
/// * Active - business as usual
pub enum BridgeState {
	Stopped,
	Active,
}

impl Default for BridgeState {
	fn default() -> Self {
		Self::Stopped
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		sp_runtime::{traits::AccountIdConversion, Saturating},
		traits::{
			tokens::fungible::{Inspect, Transfer},
			ExistenceRequirement,
		},
	};
	type BalanceOf<T, I = ()> =
		<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type BalanceOfToken<T, I = ()> =
		<<T as Config<I>>::Token as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type for collecting fees
		type Currency: Currency<Self::AccountId>;

		/// Type that is actually bridged
		type Token: Transfer<Self::AccountId>;

		#[pallet::constant]
		/// PalletId used to derive bridge's AccountId (wallet)
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		/// Maximum number of relays
		type MaxRelays: Get<u32>;

		#[pallet::constant]
		/// Maximum number of watchers
		type MaxWatchers: Get<u32>;

		#[pallet::constant]
		/// Maximum number of tokens that can be locked in pallet (a.k.a.
		/// bridged to ETH)
		type MaxTotalLocked: Get<BalanceOfToken<Self, I>>;

		#[pallet::constant]
		/// Delay between getting approval from relays and actually unlocking
		/// withdrawal for eth -> substrate transfer.
		/// Gives watchers time to stop bridge.
		type WithdrawalDelay: Get<Self::BlockNumber>;

		#[pallet::constant]
		/// Rate limit parameters This is implemented as The Leaky Bucket as a
		/// Meter algorithm - https://en.wikipedia.org/wiki/Leaky_bucket.:w
		/// First parameter is the max counter (a.k.a. max burst, max single
		/// withdrawal)
		/// Second parameter is the decay rate (a.k.a. after reaching max, how
		/// much can be withdrawn per block)
		type WithdrawalRateLimit: Get<(BalanceOfToken<Self, I>, BalanceOfToken<Self, I>)>;

		/// Origin that's authorized to set Admin and SuperAdmin
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Relay/Watcher already exists
		AlreadyExists,
		/// Incoming Receipt already processed and funds withdrawn
		AlreadyProcessed,
		/// Bridge is stopped
		BridgeStopped,
		/// This incoming receipt id is unknown
		UnknownReceiptId,
		/// Invalid relay
		InvalidRelay,
		/// Invalid watcher
		InvalidWatcher,
		/// Incoming receipt not approved for processing yet
		NotApproved,
		/// Too many votes
		TooManyVotes,
		/// Too many watchers
		TooManyWatchers,
		/// Too many relays
		TooManyRelays,
		/// Caller is unauthorized for this action
		Unauthorized,
		/// Not enough time passed since incoming receipt approval
		TooSoon,
		/// Too many tokens withdrawn in short time from bridge, try again later
		RateLimited,
		/// Too much locked in pallet already
		TooMuchLocked,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// Receipt for substrate -> eth transfer
		OutgoingReceipt {
			/// amount transferred
			amount: BalanceOfToken<T, I>,
			/// recipient on eth side
			eth_recipient: EthAddress,
		},
		/// Relay voted on approving given eth -> substrate receipt for processing
		Vote {
			/// voter
			relay: T::AccountId,
			/// Incoming Receipt ID
			receipt_id: ReceiptId,
			/// Ethereum block number at which receipt was created
			block_number: EthBlockNumber,
		},
		/// Incoming Receipt got approved for withdrawal
		Approved(ReceiptId),
		/// Incoming Receipt was processed, eth -> substrate transfer complete
		Processed(ReceiptId),
		/// Bridge state was changed by watcher, admin or superadmin
		StateChanged(BridgeState),
		/// Bridge was stopped by watcher after detecting invalid vote by relay
		EmergencyStop,
	}

	#[pallet::storage]
	/// List of accounts that act as Relays
	pub(super) type Relays<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxRelays>, ValueQuery>;

	#[pallet::storage]
	/// List of accounts that act as Watchers
	pub(super) type Watchers<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxWatchers>, ValueQuery>;

	#[pallet::storage]
	/// Number of Relay votes required to approve withdrawal
	pub(super) type VotesRequired<T: Config<I>, I: 'static = ()> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	/// Fee taken on withdrawal from caller and distributed to Voters
	pub(super) type Fee<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BalanceOf<T, I>, ValueQuery>;

	#[pallet::storage]
	/// Bridge state
	pub(super) type State<T: Config<I>, I: 'static = ()> = StorageValue<_, BridgeState, ValueQuery>;

	#[pallet::storage]
	/// Incoming Receipts - details on eth -> substrate transfers
	pub(super) type IncomingReceipts<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		ReceiptId,
		IncomingReceipt<T::AccountId, BalanceOfToken<T, I>>,
		OptionQuery,
	>;

	#[pallet::storage]
	/// Status of incoming receipts - eth -> substrate transfers
	pub(super) type StatusOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		ReceiptId,
		IncomingReceiptStatus<T::BlockNumber>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// List of relays that voted for approval of given receipt
	pub(super) type Voting<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		ReceiptId,
		BoundedVec<T::AccountId, T::MaxRelays>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// Account that can use calls that potentially lower bridge security
	pub(super) type SuperAdmin<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	/// Account that can use calls that changes config and restarts bridge
	pub(super) type Admin<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	/// Counter used to track rate limiting
	/// Decays linearly each block
	pub(super) type WithdrawalCounter<T: Config<I>, I: 'static = ()> =
		StorageValue<_, (BalanceOfToken<T, I>, T::BlockNumber), ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub relays: BoundedVec<T::AccountId, T::MaxRelays>,
		pub watchers: BoundedVec<T::AccountId, T::MaxWatchers>,
		pub votes_required: u32,
		pub fee: BalanceOf<T, I>,
		pub state: BridgeState,
		pub admin: Option<T::AccountId>,
		pub super_admin: Option<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self {
				relays: Default::default(),
				watchers: Default::default(),
				votes_required: Default::default(),
				fee: Default::default(),
				state: Default::default(),
				admin: Default::default(),
				super_admin: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			Relays::<T, I>::put(&self.relays);
			Watchers::<T, I>::put(&self.watchers);
			VotesRequired::<T, I>::put(&self.votes_required);
			Fee::<T, I>::put(&self.fee);
			State::<T, I>::put(&self.state);
			Admin::<T, I>::set(self.admin.clone());
			SuperAdmin::<T, I>::set(self.super_admin.clone());
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Entrypoint for Substrate -> ETH transfer.
		/// Takes `amount` of tokens (`T::Token`) from caller and issues a receipt for
		/// transferring them to `eth_recipient` on ETH side.
		///
		/// Deposits `OutgoingReceipt` event on success.
		///
		/// No fees other than transaction fees are taken at this point.
		///
		/// Can be called by any Signed origin.
		///
		/// Fails if bridge is stopped or caller has insufficient funds.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn deposit(
			origin: OriginFor<T>,
			amount: BalanceOfToken<T, I>,
			eth_recipient: EthAddress,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(State::<T, I>::get() == BridgeState::Active, Error::<T, I>::BridgeStopped);

			let total_locked = <T::Token as Inspect<T::AccountId>>::balance(&Self::account_id());
			ensure!(
				total_locked.saturating_add(amount) <= T::MaxTotalLocked::get(),
				Error::<T, I>::TooMuchLocked
			);

			<T::Token as Transfer<T::AccountId>>::transfer(
				&who,
				&Self::account_id(),
				amount,
				false,
			)?;
			Self::deposit_event(Event::OutgoingReceipt { amount, eth_recipient });

			Ok(())
		}

		/// Cast vote for approving funds withdrawal for given Substrate -> ETH
		/// transfer receipt.
		///
		/// Can only be called by relays.
		///
		/// Fails if:
		/// * bridge is stopped
		/// * caller isn't an authorized relay
		/// * receipt was already processed and funds were withdrawn to recipient
		/// * there's already `T::MaxRelays` number of votes casted
		///
		/// In case the `receipt` doesn't match previous `receipt` for given
		/// `receipt_id`, whole bridge will be immediately stopped and
		/// `StateChanged` event will be deposited.
		///
		/// Noop if caller already voted for given `receipt_id`.
		///
		/// Deposits `Vote` event on successful vote.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn vote_withdraw(
			origin: OriginFor<T>,
			receipt_id: ReceiptId,
			receipt: IncomingReceipt<T::AccountId, BalanceOfToken<T, I>>,
		) -> DispatchResult {
			let relay = ensure_signed(origin)?;

			let relays = Relays::<T, I>::get();
			let state = State::<T, I>::get();
			let status = StatusOf::<T, I>::get(receipt_id);
			ensure!(state == BridgeState::Active, Error::<T, I>::BridgeStopped);
			ensure!(relays.contains(&relay), Error::<T, I>::Unauthorized);
			ensure!(
				status == IncomingReceiptStatus::Voting ||
					matches!(status, IncomingReceiptStatus::Approved(_)),
				Error::<T, I>::AlreadyProcessed
			);

			if let Some(stored_receipt) = IncomingReceipts::<T, I>::get(receipt_id) {
				// verify that this relay has the same receipt for this id as previous one
				if stored_receipt != receipt {
					// someone lied, stop the bridge
					Self::do_set_state(BridgeState::Stopped);
					// we return Ok, as we DONT want to revert stopping bridge
					return Ok(())
				}
			} else {
				// first vote
				IncomingReceipts::<T, I>::insert(receipt_id, &receipt);
			}

			let mut votes = Voting::<T, I>::get(receipt_id);
			if !votes.contains(&relay) {
				votes.try_push(relay.clone()).map_err(|_| Error::<T, I>::TooManyVotes)?;
				Voting::<T, I>::insert(receipt_id, &votes);
				let block_number = frame_system::Pallet::<T>::block_number();
				if status == IncomingReceiptStatus::Voting {
					let votes_required = VotesRequired::<T, I>::get();
					if votes.len() >= votes_required as usize {
						StatusOf::<T, I>::insert(
							receipt_id,
							IncomingReceiptStatus::Approved(block_number),
						);
						Self::deposit_event(Event::Approved(receipt_id))
					}
				}

				Self::deposit_event(Event::Vote {
					relay,
					receipt_id,
					block_number: receipt.eth_block_number,
				})
			}
			Ok(())
		}

		/// Claim tokens (`T::Token`) from approved ETH -> Substrate transfer.
		///
		/// Any Signed origin can call this, tokens will always be transferred
		/// to recipient specified by Ethereum side in Incoming Receipt.
		///
		/// Takes fee (in `T::Currency`) from caller and proportionally distributes to relays that
		/// casted vote on this receipt.
		///
		/// Will fail if:
		/// * bridge is stopped
		/// * receipt was already withdrawn earlier
		/// * receipt wasn't yet approved by relays
		/// * receipt is unknown on substrate yet
		/// * caller has insufficient funds to cover the fee
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn withdraw(origin: OriginFor<T>, receipt_id: ReceiptId) -> DispatchResult {
			// FIXME do we want rate-limiting on substrate side?
			let caller = ensure_signed(origin)?;
			let voters = Voting::<T, I>::get(receipt_id);
			let state = State::<T, I>::get();
			let status = StatusOf::<T, I>::get(receipt_id);

			ensure!(state == BridgeState::Active, Error::<T, I>::BridgeStopped);
			ensure!(
				!matches!(status, IncomingReceiptStatus::Processed(_)),
				Error::<T, I>::AlreadyProcessed
			);

			if let Some(receipt) = IncomingReceipts::<T, I>::get(receipt_id) {
				if let IncomingReceiptStatus::Approved(approved_on) = status {
					let current_block_number = frame_system::Pallet::<T>::block_number();
					ensure!(
						current_block_number >= approved_on + T::WithdrawalDelay::get(),
						Error::<T, I>::TooSoon
					);
					Self::rate_limit(receipt.amount)?;
					Self::take_fee(voters, caller)?;
					T::Token::transfer(
						&Self::account_id(),
						&receipt.substrate_recipient,
						receipt.amount,
						false,
					)?;
					StatusOf::<T, I>::insert(
						receipt_id,
						IncomingReceiptStatus::Processed(frame_system::Pallet::<T>::block_number()),
					);
					Self::deposit_event(Event::<T, I>::Processed(receipt_id));
					Ok(())
				} else {
					Err(Error::<T, I>::NotApproved.into())
				}
			} else {
				Err(Error::<T, I>::UnknownReceiptId.into())
			}
		}

		/// Sets the withdrawal fee.
		///
		/// Can be called by Admin and SuperAdmin.
		///
		/// Should be set high enough to cover running costs for all relays.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn set_fee(origin: OriginFor<T>, amount: BalanceOf<T, I>) -> DispatchResult {
			Self::ensure_admin(origin)?;
			Fee::<T, I>::set(amount);
			Ok(())
		}

		/// Sets number of required votes to finish eth -> substrate transfer,
		/// process receipt and withdraw funds to recipient.
		///
		/// Can be called by SuperAdmin.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn set_votes_required(origin: OriginFor<T>, votes_required: u32) -> DispatchResult {
			Self::ensure_super_admin(origin)?;
			VotesRequired::<T, I>::set(votes_required);
			Ok(())
		}

		/// Add account as authorized Relay.
		///
		/// Can be called by SuperAdmin.
		///
		/// Will fail if:
		/// * relay already exists
		/// * there's already `T::MaxRelays` relays
		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn add_relay(origin: OriginFor<T>, relay: T::AccountId) -> DispatchResult {
			Self::ensure_super_admin(origin)?;
			Relays::<T, I>::try_mutate(|relays| -> Result<(), DispatchError> {
				ensure!(!relays.contains(&relay), Error::<T, I>::AlreadyExists);
				relays.try_push(relay).map_err(|_| Error::<T, I>::TooManyRelays)?;
				Ok(())
			})?;
			Ok(())
		}

		/// Remove account from authorized watchers.
		///
		/// Can be called by SuperAdmin.
		///
		/// Will fail if watcher doesn't exists.
		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn remove_watcher(origin: OriginFor<T>, watcher: T::AccountId) -> DispatchResult {
			Self::ensure_super_admin(origin)?;
			Watchers::<T, I>::try_mutate(|watchers| -> Result<(), DispatchError> {
				let pos = watchers
					.iter()
					.position(|x| *x == watcher)
					.ok_or(Error::<T, I>::InvalidWatcher)?;
				watchers.remove(pos);
				Ok(())
			})?;
			Ok(())
		}

		/// Remove account from authorized relays.
		///
		/// Can be called by Admin and SuperAdmin.
		///
		/// Will fail if relay doesn't exists.
		#[pallet::call_index(7)]
		#[pallet::weight(10_000)]
		pub fn remove_relay(origin: OriginFor<T>, relay: T::AccountId) -> DispatchResult {
			Self::ensure_admin(origin)?;
			Relays::<T, I>::try_mutate(|relays| -> Result<(), DispatchError> {
				let pos =
					relays.iter().position(|x| *x == relay).ok_or(Error::<T, I>::InvalidRelay)?;
				relays.remove(pos);
				Ok(())
			})?;
			Ok(())
		}

		/// Add account as authorized Watcher.
		///
		/// Can be called by Admin and SuperAdmin.
		///
		/// Will fail if:
		/// * watcher already exists
		/// * there's already `T::MaxWatchers` relays
		#[pallet::call_index(8)]
		#[pallet::weight(10_000)]
		pub fn add_watcher(origin: OriginFor<T>, watcher: T::AccountId) -> DispatchResult {
			Self::ensure_admin(origin)?;
			Watchers::<T, I>::try_mutate(|watchers| -> Result<(), DispatchError> {
				ensure!(!watchers.contains(&watcher), Error::<T, I>::AlreadyExists);
				watchers.try_push(watcher).map_err(|_| Error::<T, I>::TooManyWatchers)?;
				Ok(())
			})?;
			Ok(())
		}

		/// Stop or resume bridge
		///
		/// Can be called by Admin and SuperAdmin.
		///
		/// Deposits `StateChanged` event.
		#[pallet::call_index(9)]
		#[pallet::weight(10_000)]
		pub fn set_state(origin: OriginFor<T>, state: BridgeState) -> DispatchResult {
			Self::ensure_admin(origin)?;
			Self::do_set_state(state);
			Ok(())
		}

		/// Emergency stop the bridge. This should only be used in case an
		/// invalid vote is detected.
		///
		/// Can be called by Watchers.
		///
		/// Deposits `EmergencyStop` and `StateChanged` events.
		#[pallet::call_index(10)]
		#[pallet::weight(10_000)]
		pub fn emergency_stop(origin: OriginFor<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let watchers = Watchers::<T, I>::get();
			ensure!(watchers.contains(&caller), Error::<T, I>::Unauthorized);

			Self::do_set_state(BridgeState::Stopped);
			Self::deposit_event(Event::<T, I>::EmergencyStop);
			Ok(())
		}

		/// Set admin.
		///
		/// Admin has rights to:
		/// * remove relays
		/// * add watchers
		/// * stop and resume bridge
		/// * set withdrawal fee
		/// * change admin
		///
		/// Can be called by ForceOrigin, SuperAdmin and Admin
		#[pallet::call_index(11)]
		#[pallet::weight(10_000)]
		pub fn set_admin(origin: OriginFor<T>, admin: T::AccountId) -> DispatchResult {
			if let Err(_) = T::ForceOrigin::ensure_origin(origin.clone()) {
				Self::ensure_admin(origin)?;
			}
			Admin::<T, I>::put(admin);
			Ok(())
		}

		/// Set super admin.
		///
		/// SuperAdmin has rights to:
		/// * add relays
		/// * remove watchers
		/// * change admin and superadmin
		/// * change number of required votes
		/// * all rights of Admin
		///
		/// Can be called by ForceOrigin and SuperAdmin
		#[pallet::call_index(12)]
		#[pallet::weight(10_000)]
		pub fn set_super_admin(origin: OriginFor<T>, super_admin: T::AccountId) -> DispatchResult {
			if let Err(_) = T::ForceOrigin::ensure_origin(origin.clone()) {
				Self::ensure_super_admin(origin)?;
			}
			SuperAdmin::<T, I>::put(super_admin);
			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		fn do_set_state(state: BridgeState) {
			State::<T, I>::set(state.clone());
			Self::deposit_event(Event::StateChanged(state));
		}

		fn take_fee(
			voters: BoundedVec<T::AccountId, T::MaxRelays>,
			account: T::AccountId,
		) -> DispatchResult {
			let total_fee = Fee::<T, I>::get();
			// Rounds down, effectively the sum may be a bit less than total_fee
			let per_voter_fee = total_fee / (voters.len() as u32).into();
			for voter in voters {
				T::Currency::transfer(
					&account,
					&voter,
					per_voter_fee,
					ExistenceRequirement::AllowDeath,
				)?;
			}
			Ok(())
		}

		fn ensure_super_admin(origin: OriginFor<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(Some(caller) == SuperAdmin::<T, I>::get(), Error::<T, I>::Unauthorized);
			Ok(())
		}

		fn ensure_admin(origin: OriginFor<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(
				Some(caller.clone()) == Admin::<T, I>::get() ||
					Some(caller) == SuperAdmin::<T, I>::get(),
				Error::<T, I>::Unauthorized
			);
			Ok(())
		}

		fn rate_limit(amount: BalanceOfToken<T, I>) -> DispatchResult {
			let (mut counter, last_block_number) = WithdrawalCounter::<T, I>::get();
			let (counter_limit, decay_rate) = T::WithdrawalRateLimit::get();
			let current_block_number = frame_system::Pallet::<T>::block_number();

			// first update the counter to process decay
			if let Ok(blocks_elapsed) =
				TryInto::<u32>::try_into(current_block_number - last_block_number)
			{
				let decay = decay_rate.saturating_mul(blocks_elapsed.into());
				counter = counter.saturating_sub(decay);
			} else {
				// more than u32::max() blocks passed, that bigger than allowed window
				counter = 0u8.into();
			}

			// then add what current withdrawal
			counter = counter.saturating_add(amount);

			// check rate limit
			ensure!(counter <= counter_limit, Error::<T, I>::RateLimited);

			// update stored counter
			WithdrawalCounter::<T, I>::set((counter, current_block_number));

			Ok(())
		}
	}
}
