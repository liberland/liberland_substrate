use sp_runtime::DispatchError;

/// trait for LLM methods that only interact with LLM pallet
pub trait LLM<AccountId, Balance> {
	/// check if sender has any LLM politipooled
	fn check_pooled_llm(account: &AccountId) -> bool;
	/// check if sender has election rights unlocked
	fn is_election_unlocked(account: &AccountId) -> bool;
	/// get amount of politipooled LLM
	fn get_politi_pooled_amount() -> u64;
	/// get amount of free LLM for politics for account
	fn get_llm_politics(account: &AccountId) -> Balance;
}

/// trait for more abstract methods that take data from multiple sources
pub trait CitizenshipChecker<AccountId> {
	/// Checks if account has democracy allowed. For democracy to be allowed, account needs to:
	/// * have LLM politipooled
	/// * not have LLM Electionlock
	/// * have a KnownGood judgement
	fn ensure_democracy_allowed(account: &AccountId) -> Result<(), DispatchError>;

	/// Checks if account has elections allowed. For elections to be allowed, account needs to:
	/// * not have LLM Electionlock
	/// * have a KnownGood judgement
	fn ensure_elections_allowed(account: &AccountId) -> Result<(), DispatchError>;
}
