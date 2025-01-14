/*
Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

#![cfg(test)]

use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::{AccountIdConversion, BadOrigin, Hash};

#[test]
fn set_admin_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(Office::set_admin(RuntimeOrigin::signed(0), 1));
		System::assert_last_event(Event::<Test>::AdminChanged { new_admin: 1 }.into());
	});
}

#[test]
fn set_admin_verifies_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(Office::set_admin(RuntimeOrigin::signed(1), 1), Error::<Test>::NoPermission);
		assert_ok!(Office::set_admin(RuntimeOrigin::root(), 1));
		assert_ok!(Office::set_admin(RuntimeOrigin::signed(1), 1));
	});
}

#[test]
fn set_admin_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Office::set_admin(RuntimeOrigin::signed(0), 1));
		assert_ok!(Office::set_admin(RuntimeOrigin::signed(1), 2));
		assert_noop!(Office::set_admin(RuntimeOrigin::signed(0), 3), Error::<Test>::NoPermission);
		assert_noop!(Office::set_admin(RuntimeOrigin::signed(1), 3), Error::<Test>::NoPermission);
	});
}

#[test]
fn set_clerk_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Any));
		System::assert_last_event(
			Event::<Test>::ClerkSet { account: 1, call_filter: OfficeCallFilter::Any }.into(),
		);
	});
}

#[test]
fn set_clerk_verifies_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Office::set_clerk(RuntimeOrigin::signed(1), 1, OfficeCallFilter::Any),
			Error::<Test>::NoPermission
		);
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Any));
		assert_ok!(Office::set_clerk(RuntimeOrigin::root(), 1, OfficeCallFilter::Any));
	});
}

#[test]
fn set_clerk_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Any));
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 2, OfficeCallFilter::Remark));
		assert_eq!(Office::clerks(1), Some(OfficeCallFilter::Any));
		assert_eq!(Office::clerks(2), Some(OfficeCallFilter::Remark));
		assert_eq!(Office::clerks(3), None);
	});
}

#[test]
fn remove_clerk_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Any));
		assert_ok!(Office::remove_clerk(RuntimeOrigin::signed(0), 1));
		System::assert_last_event(Event::<Test>::ClerkRemoved { account: 1 }.into());
	});
}

#[test]
fn remove_clerk_verifies_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Office::remove_clerk(RuntimeOrigin::signed(1), 1),
			Error::<Test>::NoPermission
		);
		assert_ok!(Office::remove_clerk(RuntimeOrigin::signed(0), 1));
		assert_ok!(Office::remove_clerk(RuntimeOrigin::root(), 1));
	});
}

#[test]
fn remove_clerk_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Any));
		assert_eq!(Office::clerks(1), Some(OfficeCallFilter::Any));
		assert_ok!(Office::remove_clerk(RuntimeOrigin::signed(0), 1));
		assert_eq!(Office::clerks(1), None);
	});
}

#[test]
fn execute_deposits_event() {
	new_test_ext().execute_with(|| {
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Any));
		assert_ok!(Office::execute(
			RuntimeOrigin::signed(1),
			Box::new(frame_system::Call::remark { remark: vec![] }.into())
		));
		System::assert_last_event(Event::<Test>::CallExecuted { result: Ok(()) }.into());
	});
}

#[test]
fn execute_verifies_origin() {
	new_test_ext().execute_with(|| {
		let call: Box<RuntimeCall> = Box::new(frame_system::Call::remark { remark: vec![] }.into());
		assert_noop!(
			Office::execute(RuntimeOrigin::signed(1), call.clone()),
			Error::<Test>::NoPermission
		);
		assert_noop!(Office::execute(RuntimeOrigin::root(), call.clone()), BadOrigin);
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Any));
		assert_ok!(Office::execute(RuntimeOrigin::signed(1), call.clone()));
		System::assert_last_event(Event::<Test>::CallExecuted { result: Ok(()) }.into());
	});
}

#[test]
fn execute_filters_calls() {
	new_test_ext().execute_with(|| {
		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Remark));
		assert_ok!(Office::execute(
			RuntimeOrigin::signed(1),
			Box::new(frame_system::Call::remark { remark: vec![] }.into())
		));
		System::assert_last_event(Event::<Test>::CallExecuted { result: Ok(()) }.into());
		assert_noop!(
			Office::execute(
				RuntimeOrigin::signed(1),
				Box::new(frame_system::Call::remark_with_event { remark: vec![] }.into())
			),
			frame_system::Error::<Test>::CallFiltered,
		);
	});
}

#[test]
fn execute_works() {
	new_test_ext().execute_with(|| {
		let remark: Vec<u8> = vec![1, 2, 3];
		let sender = OfficePalletId::get().into_account_truncating();
		let hash = <Test as frame_system::Config>::Hashing::hash(&remark[..]);
		let call = Box::new(frame_system::Call::remark_with_event { remark }.into());

		assert_ok!(Office::set_clerk(RuntimeOrigin::signed(0), 1, OfficeCallFilter::Any));
		assert_ok!(Office::execute(RuntimeOrigin::signed(1), call));
		System::assert_has_event(frame_system::Event::<Test>::Remarked { sender, hash }.into());
	});
}

#[test]
fn genesis_config_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Office::admin(), Some(0));
		assert_eq!(Office::clerks(100), None);
		assert_eq!(GenesisOffice::admin(), Some(99));
		assert_eq!(GenesisOffice::clerks(100), Some(OfficeCallFilter::Any));
		assert_eq!(GenesisOffice::clerks(101), Some(OfficeCallFilter::Remark));
	});
}
