use super::*;

/// Noop implementation of CitizenshipChecker - mostly to be used in tests to
/// prevent circular dependencies
impl<T> CitizenshipChecker<T> for () {
    fn ensure_politics_allowed(_account: &T) -> Result<(), DispatchError> {
        Ok(())
    }

    fn is_citizen(_account: &T) -> bool {
        false
    }

    fn citizens_count() -> u64 {
        0
    }

    fn identity_changed(_was_citizen_before_change: bool, _account: &T) {
    }
}