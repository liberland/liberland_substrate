use frame_support::ensure;
use sp_runtime::traits::Get;
use sp_std::marker::PhantomData;

use super::*;

/// Mock implementation of CitizenshipChecker - mostly to be used in tests
pub struct MockCitizenshipChecker<T, C1, C2>(PhantomData<T>, PhantomData<C1>, PhantomData<C2>);
impl<T, C1, C2> CitizenshipChecker<T> for MockCitizenshipChecker<T, C1, C2>
where
	T: PartialEq,
	C1: Get<T>,
	C2: Get<T>,
{
	fn ensure_politics_allowed(account: &T) -> Result<(), DispatchError> {
		ensure!(Self::is_citizen(account), DispatchError::Other("NotCitizen"));
		Ok(())
	}

	fn ensure_land_nfts_allowed(account: &T) -> Result<(), DispatchError> {
		ensure!(Self::is_citizen(account), DispatchError::Other("NotCitizen"));
		Ok(())
	}

	fn is_citizen(account: &T) -> bool {
		*account == C1::get() || *account == C2::get()
	}

	fn citizens_count() -> u64 {
		2
	}

	fn identity_changed(_was_citizen_before_change: bool, _account: &T) {}
}

/// Noop implementation of CitizenshipChecker - mostly to be used in tests
impl<T> OnLLMPoliticsUnlock<T> for () {
	fn on_llm_politics_unlock(_account: &T) -> Result<(), DispatchError> {
		Ok(())
	}
}

/// Noop implementation of CitizenshipChecker - mostly to be used in tests to
/// prevent circular dependencies
impl<T> CitizenshipChecker<T> for () {
	fn ensure_politics_allowed(_account: &T) -> Result<(), DispatchError> {
		Ok(())
	}

	fn ensure_land_nfts_allowed(_account: &T) -> Result<(), DispatchError> {
		Ok(())
	}

	fn is_citizen(_account: &T) -> bool {
		false
	}

	fn citizens_count() -> u64 {
		0
	}

	fn identity_changed(_was_citizen_before_change: bool, _account: &T) {}
}
