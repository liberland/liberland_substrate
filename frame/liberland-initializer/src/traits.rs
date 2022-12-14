pub trait LLInitializer<AccountId, Balance> {
	#[cfg(feature = "runtime-benchmarks")]
	fn make_citizen(account: &AccountId, amount: Balance);
}