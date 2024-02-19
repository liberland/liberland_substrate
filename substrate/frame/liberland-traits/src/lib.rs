#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::DispatchError;

mod impls;
pub use impls::*;

pub trait LLInitializer<AccountId> {
	#[cfg(any(test, feature = "runtime-benchmarks"))]
	fn make_test_citizen(account: &AccountId);
}

pub trait OnLLMPoliticsUnlock<AccountId> {
	fn on_llm_politics_unlock(account: &AccountId) -> Result<(), DispatchError>;
}

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
	/// * have valid citizen identity
	/// * have a KnownGood judgement
	fn ensure_politics_allowed(account: &AccountId) -> Result<(), DispatchError>;

	/// Checks if account has NFTs allowed. For NFTs to be allowed, account needs to:
	/// * have LLM politipooled
	/// * have valid citizen identity
	/// * have a KnownGood judgement
	fn ensure_land_nfts_allowed(account: &AccountId) -> Result<(), DispatchError>;

	/// Check if given account is a citizen (valid identity + KnownGood judgement)
	fn is_citizen(account: &AccountId) -> bool;

	/// Calculate number of valid citizens (valid identities + KnownGood
	/// judgements).
	fn citizens_count() -> u64;

	/// To be used by identity pallet to update stored number of citizens
	fn identity_changed(was_citizen_before_change: bool, account: &AccountId);
}
