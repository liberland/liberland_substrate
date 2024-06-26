use psp22::PSP22Error;

#[ink::trait_definition]
pub trait WrappedAsset {
	/// Deposits the transferred amount of asset and mints that much wAsset to the callers account.
	///
	/// # Events
	///
	/// On success a `Transfer` event is emitted for newly minted wAsset (with `from` being `None`).
	///
	/// No-op if the transferred value is zero, returns success and no events are emitted.
	///
	/// # Errors
	///
	/// Reverts with `Custom` error variant if minting new tokens would cause the total token supply
	/// to exceed maximal `u128` value.
	#[ink(message, payable)]
	fn deposit(&mut self, value: u128) -> Result<(), PSP22Error>;

	/// Burns `value` wAsset tokens from the callers account and transfers that much Asset to them.
	///
	/// # Events
	///
	/// On success a `Transfer` event is emitted for burned wAsset (with `to` being `None`).
	///
	/// No-op if the `value` is zero, returns success and no events are emitted.
	///
	/// # Errors
	///
	/// Reverts with `InsufficientBalance` if the `value` exceeds the caller's wAsset balance.
	#[ink(message)]
	fn withdraw(&mut self, value: u128) -> Result<(), PSP22Error>;
}

/// Mainnet deployment address
pub const MAINNET: &str = "";
/// Bastiat deployment address
pub const BASTIAT: &str = "";
