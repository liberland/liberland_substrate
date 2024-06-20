#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use log::{error, trace};
use pallet_contracts::chain_extension::{ChainExtension, Environment, Ext, InitState, RetVal};
use sp_runtime::DispatchError;

#[derive(Decode, Encode, MaxEncodedLen)]
pub struct LLMForceTransferArguments<T: pallet_llm::Config> {
	from: pallet_llm::LLMAccount<T::AccountId>,
	to: pallet_llm::LLMAccount<T::AccountId>,
	amount: T::Balance,
}

/// Contract extension for the Liberland Chain
#[derive(Default)]
pub struct LiberlandExtension;

impl LiberlandExtension {
	fn llm_force_transfer<E: Ext>(
		&mut self,
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_llm::Config + pallet_contracts::Config,
		<E::T as pallet_contracts::Config>::RuntimeCall: From<pallet_llm::Call<E::T>>,
	{
		trace!(
			target: "runtime",
			"[ChainExtension]|call|llm_force_transfer"
		);
		let mut env = env.buf_in_buf_out();
		let args: LLMForceTransferArguments<E::T> = env.read_as()?;
		let ext = env.ext();
		let call: <E::T as pallet_contracts::Config>::RuntimeCall =
			pallet_llm::Call::<E::T>::force_transfer {
				from: args.from,
				to: args.to,
				amount: args.amount,
			}
			.into();
		ext.call_runtime(call).map_err(|e| e.error)?;
		Ok(RetVal::Converging(0))
	}
}

impl<T> ChainExtension<T> for LiberlandExtension
where
	T: pallet_llm::Config + pallet_contracts::Config,
	<T as pallet_contracts::Config>::RuntimeCall: From<pallet_llm::Call<T>>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_llm::Config + pallet_contracts::Config,
		<E::T as pallet_contracts::Config>::RuntimeCall: From<pallet_llm::Call<E::T>>,
	{
		let func_id = env.func_id();
		match func_id {
			1 => self.llm_force_transfer::<E>(env),
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			},
		}
	}

	fn enabled() -> bool {
		true
	}
}
