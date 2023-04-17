/*

Copyright © 2023 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

use crate::*;
use frame_support::ensure;
use liberland_traits::CitizenshipChecker;
use sp_runtime::DispatchResult;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub fn do_set_citizenship_required(
		collection: T::CollectionId,
		citizenship_required: bool,
		maybe_check_owner: Option<T::AccountId>,
	) -> DispatchResult {
		let collection_details =
			Collection::<T, I>::get(collection).ok_or(Error::<T, I>::UnknownCollection)?;
		if let Some(check_owner) = maybe_check_owner {
			ensure!(collection_details.owner == check_owner, Error::<T, I>::NoPermission);
		}

		CitizenshipRequired::<T, I>::insert(collection, citizenship_required);
		Ok(())
	}

	pub fn maybe_ensure_citizenship(
		collection: T::CollectionId,
		who: &T::AccountId,
	) -> DispatchResult {
		if CitizenshipRequired::<T, I>::get(collection) {
			T::Citizenship::ensure_land_nfts_allowed(who)?
		}
		Ok(())
	}
}
