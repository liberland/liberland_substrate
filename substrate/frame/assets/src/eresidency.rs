/*

Copyright © 2024 Liberland

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

*/

use crate::*;
use sp_runtime::DispatchResult;
use liberland_traits::CitizenshipChecker;
use frame_support::ensure;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub fn do_set_parameters(
		asset: T::AssetId,
        parameters: AssetParameters,
		maybe_check_owner: Option<T::AccountId>,
	) -> DispatchResult {
        if let Some(check_owner) = maybe_check_owner {
            let d = Asset::<T, I>::get(&asset).ok_or(Error::<T, I>::Unknown)?;
            ensure!(d.owner == check_owner, Error::<T, I>::NoPermission);
        }

        Parameters::<T, I>::insert(&asset, parameters);
		Self::deposit_event(Event::ParametersSet { asset_id: asset, parameters });
        Ok(())
	}

    pub fn maybe_ensure_eresidency(asset: T::AssetId, who: &T::AccountId) -> DispatchResult {
        let AssetParameters { eresidency_required } = Parameters::<T, I>::get(asset);
        if eresidency_required {
            T::Citizenship::ensure_stocks_allowed(who)?
        }
        Ok(())
    }
}
