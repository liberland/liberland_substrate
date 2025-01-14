use psp22::PSP22Error;

#[ink::trait_definition]
pub trait WrappedLLD {
	/// Deposits the transferred amount of LLD and mints that much wLLD to the callers account.
	///
	/// # Events
	///
	/// On success a `Transfer` event is emitted for newly minted wLLD (with `from` being `None`).
	///
	/// No-op if the transferred value is zero, returns success and no events are emitted.
	///
	/// # Errors
	///
	/// Reverts with `Custom` error variant if minting new tokens would cause the total token supply
	/// to exceed maximal `u128` value.
	#[ink(message, payable)]
	fn deposit(&mut self) -> Result<(), PSP22Error>;

	/// Burns `value` wLLD tokens from the callers account and transfers that much LLD to them.
	///
	/// # Events
	///
	/// On success a `Transfer` event is emitted for burned wLLD (with `to` being `None`).
	///
	/// No-op if the `value` is zero, returns success and no events are emitted.
	///
	/// # Errors
	///
	/// Reverts with `InsufficientBalance` if the `value` exceeds the caller's wLLD balance.
	#[ink(message)]
	fn withdraw(&mut self, value: u128) -> Result<(), PSP22Error>;
}

/// Mainnet deployment address
pub const MAINNET: &str = "5FBQRNJfzsttYvw1XnSwxwSUmb7A3EYm4q8aiscADREqvzYz";
/// Bastiat deployment address
pub const BASTIAT: &str = "5D6qmZpwAaAc1L32fUYP67MQi2qj2LurCoxT2KZK1ikUsRqr";
