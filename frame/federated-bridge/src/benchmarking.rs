/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{
	Pallet as Bridge,
	StatusOf,
	IncomingReceiptStatus,
	Fee,
	VotesRequired,
	BridgeState,
	Admin, SuperAdmin,
};
use sp_runtime::traits::Bounded;
use frame_benchmarking::{account, benchmarks_instance_pallet, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_std::prelude::*;
use frame_support::traits::{Currency, fungible::{Mutate, Inspect}};

const SEED: u32 = 0;

type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type BalanceOfToken<T, I> =
	<<T as Config<I>>::Token as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

fn activate<T: Config<I>, I: 'static>() {
	let admin: T::AccountId = account("admin", 0, SEED);
	Bridge::<T, I>::set_admin(RawOrigin::Root.into(), admin.clone()).unwrap();
	Bridge::<T, I>::set_state(RawOrigin::Signed(admin).into(), BridgeState::Active).unwrap();
}

fn receipt<T: Config<I>, I: 'static>(i: u8) -> (ReceiptId, IncomingReceipt<T::AccountId, BalanceOfToken<T, I>>) {
	(
		[i; 32],
		IncomingReceipt {
			eth_block_number: Default::default(),
			substrate_recipient: account("recipient", 0, SEED),
			amount: T::Token::minimum_balance(),
		},
	)
}

fn add_watchers<T: Config<I>, I: 'static>(r: u32) -> Result<(), &'static str> {
	let admin: T::AccountId = account("admin", 0, SEED);
	Bridge::<T, I>::set_super_admin(RawOrigin::Root.into(), admin.clone()).unwrap();
	let watchers: BoundedVec<T::AccountId, T::MaxWatchers> = vec![].try_into().unwrap();
	Watchers::<T, I>::put(watchers);

	for i in 1..=r {
		let watcher: T::AccountId = account("watcher", i, SEED);
		Bridge::<T, I>::add_watcher(RawOrigin::Signed(admin.clone()).into(), watcher).unwrap();
	}

	assert_eq!(Watchers::<T, I>::get().len(), r as usize);
	Ok(())
}

fn add_relays<T: Config<I>, I: 'static>(r: u32) -> Result<(), &'static str> {
	let admin: T::AccountId = account("admin", 0, SEED);
	Bridge::<T, I>::set_super_admin(RawOrigin::Root.into(), admin.clone()).unwrap();
	let relays: BoundedVec<T::AccountId, T::MaxRelays> = vec![].try_into().unwrap();
	Relays::<T, I>::put(relays);

	for i in 1..=r {
		let relay: T::AccountId = account("relay", i, SEED);
		Bridge::<T, I>::add_relay(RawOrigin::Signed(admin.clone()).into(), relay).unwrap();
	}

	assert_eq!(Relays::<T, I>::get().len(), r as usize);
	Ok(())
}

fn add_votes<T: Config<I>, I: 'static>(r: u32) -> Result<(), &'static str> {
	activate::<T, I>();
	let (receipt_id, receipt_data) = receipt::<T, I>(0);
	for i in 1..=r {
		let relay: T::AccountId = account("relay", i, SEED);
		Bridge::<T, I>::vote_withdraw(RawOrigin::Signed(relay).into(),
			receipt_id.clone(),
			receipt_data.clone(),
		).unwrap();
	}

	assert_eq!(Voting::<T, I>::get(receipt_id).len(), r as usize);
	Ok(())
}


benchmarks_instance_pallet! {
	deposit {
		activate::<T, I>();
		let acc: T::AccountId = account("a", 1, SEED);
		let amount = T::Token::minimum_balance();
		let eth_recipient: EthAddress = Default::default();
		T::Token::set_balance(&acc, BalanceOfToken::<T, I>::max_value()/2u8.into());
		T::Currency::make_free_balance_be(&acc, BalanceOf::<T, I>::max_value()/2u8.into());

		let origin = RawOrigin::Signed(acc.clone());
	}: _(origin, amount.clone(), eth_recipient.clone())
	verify {
		let ev: <T as Config<I>>::RuntimeEvent =
			Event::<T, I>::OutgoingReceipt { amount, eth_recipient }.into();
		frame_system::Pallet::<T>::assert_last_event(ev.into());
	}

	vote_withdraw {
		let r in 1 .. T::MaxRelays::get() => add_relays::<T, I>(r)?;
		activate::<T, I>();

		let admin: T::AccountId = account("admin", 0, SEED);
		let origin = RawOrigin::Signed(admin.clone());
		Bridge::<T, I>::set_votes_required(origin.into(), r).unwrap();

		add_votes::<T, I>(r-1).unwrap();

		let relay: T::AccountId = account("relay", r, SEED);
		let origin = RawOrigin::Signed(relay.clone());
		let (receipt_id, receipt_data) = receipt::<T, I>(0);
	}: _(origin, receipt_id.clone(), receipt_data)
	verify {
		assert!(
			matches!(
				StatusOf::<T, I>::get(receipt_id),
				IncomingReceiptStatus::Approved(_),
			),
		);
	}

	withdraw {
		let r in 1 .. T::MaxRelays::get() => add_relays::<T, I>(r)?;

		let admin: T::AccountId = account("admin", 0, SEED);
		let origin = RawOrigin::Signed(admin.clone());
		Bridge::<T, I>::set_votes_required(origin.into(), r).unwrap();
		add_votes::<T, I>(r).unwrap();

		let user: T::AccountId = account("user", r, SEED);
		let origin = RawOrigin::Signed(user.clone());
		let (receipt_id, _) = receipt::<T, I>(0);

		frame_system::Pallet::<T>::set_block_number(
			T::WithdrawalDelay::get() + 10u8.into()
		);

		T::Token::set_balance(&Bridge::<T, I>::account_id(), BalanceOfToken::<T, I>::max_value()/2u8.into());
		T::Currency::make_free_balance_be(&user, BalanceOf::<T, I>::max_value()/2u8.into());

	}: _(origin, receipt_id.clone())
	verify {
		assert!(
			matches!(
				StatusOf::<T, I>::get(receipt_id),
				IncomingReceiptStatus::Processed(_),
			),
		);
	}

	set_fee {
		let admin: T::AccountId = account("admin", 0, SEED);
		Bridge::<T, I>::set_admin(RawOrigin::Root.into(), admin.clone()).unwrap();
		let origin = RawOrigin::Signed(admin.clone());
	}: _(origin, 10u32.into())
	verify {
		assert_eq!(Fee::<T, I>::get(), 10u32.into());
	}

	set_votes_required {
		let admin: T::AccountId = account("admin", 0, SEED);
		Bridge::<T, I>::set_super_admin(RawOrigin::Root.into(), admin.clone()).unwrap();
		let origin = RawOrigin::Signed(admin.clone());
	}: _(origin, 10u32)
	verify {
		assert_eq!(VotesRequired::<T, I>::get(), 10u32);
	}

	add_relay {
		let r in 1 .. T::MaxRelays::get() - 1 => add_relays::<T, I>(r)?;
		let admin: T::AccountId = account("admin", 0, SEED);
		let relay: T::AccountId = account("relay", r+1, SEED);
        let origin = RawOrigin::Signed(admin.clone());
	}: _(origin, relay.clone())
	verify {
		assert!(Relays::<T, I>::get().contains(&relay));
	}

	remove_watcher {
		let r in 1 .. T::MaxWatchers::get() => add_watchers::<T, I>(r)?;
		let admin: T::AccountId = account("admin", 0, SEED);
		let watcher: T::AccountId = account("watcher", 1, SEED);
        let origin = RawOrigin::Signed(admin.clone());
		assert!(Watchers::<T, I>::get().contains(&watcher));
	}: _(origin, watcher.clone())
	verify {
		assert!(!Watchers::<T, I>::get().contains(&watcher));
	}

	remove_relay {
		let r in 1 .. T::MaxRelays::get() => add_relays::<T, I>(r)?;
		let admin: T::AccountId = account("admin", 0, SEED);
		let relay: T::AccountId = account("relay", 1, SEED);
        let origin = RawOrigin::Signed(admin.clone());
		assert!(Relays::<T, I>::get().contains(&relay));
	}: _(origin, relay.clone())
	verify {
		assert!(!Relays::<T, I>::get().contains(&relay));
	}

	add_watcher {
		let r in 1 .. T::MaxWatchers::get() - 1 => add_watchers::<T, I>(r)?;
		let admin: T::AccountId = account("admin", 0, SEED);
		let watcher: T::AccountId = account("watcher", r+1, SEED);
        let origin = RawOrigin::Signed(admin.clone());
	}: _(origin, watcher.clone())
	verify {
		assert!(Watchers::<T, I>::get().contains(&watcher));
	}

	set_state {
		let admin: T::AccountId = account("admin", 0, SEED);
		Bridge::<T, I>::set_admin(RawOrigin::Root.into(), admin.clone()).unwrap();
		let origin = RawOrigin::Signed(admin.clone());
	}: _(origin, BridgeState::Stopped)
	verify {
		assert_eq!(State::<T, I>::get(), BridgeState::Stopped);
	}

	emergency_stop {
		let r in 1 .. T::MaxWatchers::get() => add_watchers::<T, I>(r)?;
		let watcher: T::AccountId = account("watcher", 1, SEED);
        let origin = RawOrigin::Signed(watcher.clone());
	}: _(origin)
	verify {
		assert_eq!(State::<T, I>::get(), BridgeState::Stopped);
	}

	set_admin {
		let admin: T::AccountId = account("admin", 0, SEED);
	}: _(RawOrigin::Root, admin.clone())
	verify {
		assert_eq!(Admin::<T, I>::get(), Some(admin));
	}

	set_super_admin {
		let admin: T::AccountId = account("admin", 0, SEED);
	}: _(RawOrigin::Root, admin.clone())
	verify {
		assert_eq!(SuperAdmin::<T, I>::get(), Some(admin));
	}
}

impl_benchmark_test_suite!(Bridge, crate::mock::new_test_ext(), crate::mock::Test,);
