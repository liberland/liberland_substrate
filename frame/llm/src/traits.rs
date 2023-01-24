use sp_runtime::DispatchError;

/// trait for LLM methods that only interact with LLM pallet
pub trait LLM<AccountId, Balance> {
	/// check if sender has any LLM politipooled
	fn check_pooled_llm(account: &AccountId) -> bool;
	/// check if sender has election rights unlocked
	fn is_election_unlocked(account: &AccountId) -> bool;
	/// get amount of politipooled LLM
	fn get_politi_pooled_amount() -> Balance;
	/// get amount of free LLM for politics for account
	fn get_llm_politics(account: &AccountId) -> Balance;
}

/// trait for more abstract methods that take data from multiple sources
pub trait CitizenshipChecker<AccountId> {
	/// Checks if account has politics allowed. For politics to be allowed, account needs to:
	/// * have LLM politipooled
	/// * not have LLM Electionlock
	/// * have a KnownGood judgement
	fn ensure_politics_allowed(account: &AccountId) -> Result<(), DispatchError>;

	/// Check if given account is a citizen (KnownGood judgement)
	fn is_citizen(account: &AccountId) -> bool;

	/// Calculate number of valid citizens (KnownGood judgements). This is expensive.
	fn citizens_count() -> usize;
}
