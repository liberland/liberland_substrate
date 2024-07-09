/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(test)]

use crate::{mock::*, Event, NegativeImbalanceOf};
use frame_support::traits::{Imbalance, OnUnbalanced, SortedMembers};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use sp_runtime::traits::{AccountIdConversion, Hash};

#[test]
fn execute_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(CustomAccount::execute(
			RuntimeOrigin::root(),
			Box::new(frame_system::Call::remark_with_event { remark: vec![] }.into())
		));
		System::assert_last_event(Event::<Test>::CallExecuted { result: Ok(()) }.into());
	});
}

#[test]
fn execute_verifies_origin() {
	new_test_ext().execute_with(|| {
		let call: Box<RuntimeCall> =
			Box::new(frame_system::Call::remark_with_event { remark: vec![] }.into());
		assert_noop!(CustomAccount::execute(RuntimeOrigin::signed(1), call.clone()), BadOrigin,);
		assert_ok!(CustomAccount::execute(RuntimeOrigin::root(), call.clone()));
		System::assert_last_event(Event::<Test>::CallExecuted { result: Ok(()) }.into());
	});
}

#[test]
fn execute_filters_calls() {
	new_test_ext().execute_with(|| {
		assert_ok!(CustomAccount::execute(
			RuntimeOrigin::root(),
			Box::new(frame_system::Call::remark_with_event { remark: vec![] }.into())
		));
		System::assert_last_event(Event::<Test>::CallExecuted { result: Ok(()) }.into());
		assert_noop!(
			CustomAccount::execute(
				RuntimeOrigin::root(),
				Box::new(frame_system::Call::remark { remark: vec![] }.into())
			),
			frame_system::Error::<Test>::CallFiltered
		);
	});
}

#[test]
fn execute_works() {
	new_test_ext().execute_with(|| {
		let remark: Vec<u8> = vec![1, 2, 3];
		let sender = CustomAccountPalletId::get().into_account_truncating();
		let hash = <Test as frame_system::Config>::Hashing::hash(&remark[..]);
		let call = Box::new(frame_system::Call::remark_with_event { remark }.into());

		assert_ok!(CustomAccount::execute(RuntimeOrigin::root(), call));
		System::assert_has_event(frame_system::Event::<Test>::Remarked { sender, hash }.into());
	});
}

#[test]
fn execute_unbalanced() {
	new_test_ext().execute_with(|| {
		let imbalance = NegativeImbalanceOf::<Test, ()>::new(100u64);
		let amount = imbalance.peek();
		let call_account_id: u64 = CustomAccountPalletId::get().into_account_truncating();
		let balance_before = Balances::free_balance(call_account_id);
		CustomAccount::on_unbalanced(imbalance);
		let balance_after = Balances::free_balance(call_account_id);
		assert_eq!(balance_before, balance_after - 100);
		System::assert_last_event(Event::<Test>::Deposit { value: amount }.into());
	});
}

#[test]
fn sorted_members_are_correct() {
	new_test_ext().execute_with(|| {
		let call_account_id: u64 = CustomAccountPalletId::get().into_account_truncating();
		let members = CustomAccount::sorted_members();
		assert_eq!(members.len(), 1);
		assert_eq!(members[0], call_account_id)
	});
}
