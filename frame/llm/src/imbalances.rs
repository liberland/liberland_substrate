// FIXME this is derived from
// https://paritytech.github.io/substrate/master/src/pallet_balances/lib.rs.html#1251,
// with minor changes only.  add proper copyright notices & licensing info

use frame_support::traits::{Imbalance, SameOrOther, TryDrop};
use sp_runtime::{traits::Zero, RuntimeDebug};
use sp_std::{mem, result};

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been created without any equal and opposite accounting.
#[must_use]
#[derive(RuntimeDebug, PartialEq, Eq)]
pub struct PositiveImbalance(u128);

impl PositiveImbalance {
	/// Create a new positive imbalance from a balance.
	pub fn new(amount: u128) -> Self {
		PositiveImbalance(amount)
	}
}

/// Opaque, move-only struct with private fields that serves as a token denoting that
/// funds have been destroyed without any equal and opposite accounting.
#[must_use]
#[derive(RuntimeDebug, PartialEq, Eq)]
pub struct NegativeImbalance(u128);

impl NegativeImbalance {
	/// Create a new negative imbalance from a balance.
	pub fn new(amount: u128) -> Self {
		NegativeImbalance(amount)
	}
}

impl TryDrop for PositiveImbalance {
	fn try_drop(self) -> result::Result<(), Self> {
		self.drop_zero()
	}
}

impl Default for PositiveImbalance {
	fn default() -> Self {
		Self::zero()
	}
}

impl Imbalance<u128> for PositiveImbalance {
	type Opposite = NegativeImbalance;

	fn zero() -> Self {
		Self(Zero::zero())
	}
	fn drop_zero(self) -> result::Result<(), Self> {
		if self.0.is_zero() {
			Ok(())
		} else {
			Err(self)
		}
	}
	fn split(self, amount: u128) -> (Self, Self) {
		let first = self.0.min(amount);
		let second = self.0 - first;

		mem::forget(self);
		(Self(first), Self(second))
	}
	fn merge(mut self, other: Self) -> Self {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);

		self
	}
	fn subsume(&mut self, other: Self) {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);
	}
	fn offset(self, other: Self::Opposite) -> SameOrOther<Self, Self::Opposite> {
		let (a, b) = (self.0, other.0);
		mem::forget((self, other));

		if a > b {
			SameOrOther::Same(Self(a - b))
		} else if b > a {
			SameOrOther::Other(NegativeImbalance::new(b - a))
		} else {
			SameOrOther::None
		}
	}
	fn peek(&self) -> u128 {
		self.0
	}
}

impl TryDrop for NegativeImbalance {
	fn try_drop(self) -> result::Result<(), Self> {
		self.drop_zero()
	}
}

impl Default for NegativeImbalance {
	fn default() -> Self {
		Self::zero()
	}
}

impl Imbalance<u128> for NegativeImbalance {
	type Opposite = PositiveImbalance;

	fn zero() -> Self {
		Self(Zero::zero())
	}
	fn drop_zero(self) -> result::Result<(), Self> {
		if self.0.is_zero() {
			Ok(())
		} else {
			Err(self)
		}
	}
	fn split(self, amount: u128) -> (Self, Self) {
		let first = self.0.min(amount);
		let second = self.0 - first;

		mem::forget(self);
		(Self(first), Self(second))
	}
	fn merge(mut self, other: Self) -> Self {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);

		self
	}
	fn subsume(&mut self, other: Self) {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);
	}
	fn offset(self, other: Self::Opposite) -> SameOrOther<Self, Self::Opposite> {
		let (a, b) = (self.0, other.0);
		mem::forget((self, other));

		if a > b {
			SameOrOther::Same(Self(a - b))
		} else if b > a {
			SameOrOther::Other(PositiveImbalance::new(b - a))
		} else {
			SameOrOther::None
		}
	}
	fn peek(&self) -> u128 {
		self.0
	}
}

impl Drop for PositiveImbalance {
	/// Basic drop handler will just square up the total issuance.
	fn drop(&mut self) {
		unimplemented!("NOT SUPPORTED!");
	}
}

impl Drop for NegativeImbalance {
	/// Basic drop handler will just square up the total issuance.
	fn drop(&mut self) {
		unimplemented!("NOT SUPPORTED!");
	}
}
