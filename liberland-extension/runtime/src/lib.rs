#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use log::{error, trace};
use pallet_contracts::chain_extension::{ChainExtension, Environment, Ext, InitState, RetVal};
use sp_runtime::{
	traits::{Get, StaticLookup},
	DispatchError,
};

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
	fn llm_force_transfer<E: Ext>(env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_llm::Config,
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

	fn asset_balance_of<E: Ext>(env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_assets::Config,
	{
		trace!(
			target: "runtime",
			"[ChainExtension]|call|asset_balance_of"
		);

		let mut env = env.buf_in_buf_out();

		let weight = <E::T as frame_system::Config>::DbWeight::get().reads_writes(1, 0);
		env.charge_weight(weight)?;

		let (asset_id, account): (
			<E::T as pallet_assets::Config>::AssetId,
			<E::T as frame_system::Config>::AccountId,
		) = env.read_as()?;

		let balance = pallet_assets::Pallet::<E::T>::balance(asset_id, account);
		let balance_encoded = balance.encode();
		env.write(&balance_encoded, false, None)?;
		Ok(RetVal::Converging(0))
	}

	fn asset_total_supply_of<E: Ext>(
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_assets::Config,
	{
		trace!(
			target: "runtime",
			"[ChainExtension]|call|asset_total_supply_of"
		);

		let mut env = env.buf_in_buf_out();

		let weight = <E::T as frame_system::Config>::DbWeight::get().reads_writes(1, 0);
		env.charge_weight(weight)?;

		let asset_id: <E::T as pallet_assets::Config>::AssetId = env.read_as()?;

		let supply = pallet_assets::Pallet::<E::T>::total_supply(asset_id);
		let supply_encoded = supply.encode();
		env.write(&supply_encoded, false, None)?;
		Ok(RetVal::Converging(0))
	}

	fn asset_approve_transfer<E: Ext>(
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_assets::Config,
		<E::T as pallet_contracts::Config>::RuntimeCall: From<pallet_assets::Call<E::T>>,
	{
		trace!(
			target: "runtime",
			"[ChainExtension]|call|asset_approve_transfer"
		);

		let mut env = env.buf_in_buf_out();
		let (asset_id, delegate, amount): (
			<E::T as pallet_assets::Config>::AssetId,
			<E::T as frame_system::Config>::AccountId,
			<E::T as pallet_assets::Config>::Balance,
		) = env.read_as()?;

		let ext = env.ext();
		let call: <E::T as pallet_contracts::Config>::RuntimeCall =
			pallet_assets::Call::<E::T>::approve_transfer {
				id: asset_id.into(),
				delegate: <E::T as frame_system::Config>::Lookup::unlookup(delegate),
				amount,
			}
			.into();
		ext.call_runtime(call).map_err(|e| e.error)?;

		Ok(RetVal::Converging(0))
	}

	fn asset_cancel_approval<E: Ext>(
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_assets::Config,
		<E::T as pallet_contracts::Config>::RuntimeCall: From<pallet_assets::Call<E::T>>,
	{
		trace!(
			target: "runtime",
			"[ChainExtension]|call|asset_cancel_approval"
		);

		let mut env = env.buf_in_buf_out();
		let (asset_id, delegate): (
			<E::T as pallet_assets::Config>::AssetId,
			<E::T as frame_system::Config>::AccountId,
		) = env.read_as()?;

		let ext = env.ext();
		let call: <E::T as pallet_contracts::Config>::RuntimeCall =
			pallet_assets::Call::<E::T>::cancel_approval {
				id: asset_id.into(),
				delegate: <E::T as frame_system::Config>::Lookup::unlookup(delegate),
			}
			.into();
		ext.call_runtime(call).map_err(|e| e.error)?;

		Ok(RetVal::Converging(0))
	}

	fn asset_transfer<E: Ext>(env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_assets::Config,
		<E::T as pallet_contracts::Config>::RuntimeCall: From<pallet_assets::Call<E::T>>,
	{
		trace!(
			target: "runtime",
			"[ChainExtension]|call|asset_transfer"
		);

		let mut env = env.buf_in_buf_out();
		let (asset_id, target, amount): (
			<E::T as pallet_assets::Config>::AssetId,
			<E::T as frame_system::Config>::AccountId,
			<E::T as pallet_assets::Config>::Balance,
		) = env.read_as()?;

		let ext = env.ext();
		let call: <E::T as pallet_contracts::Config>::RuntimeCall =
			pallet_assets::Call::<E::T>::transfer {
				id: asset_id.into(),
				target: <E::T as frame_system::Config>::Lookup::unlookup(target),
				amount,
			}
			.into();
		ext.call_runtime(call).map_err(|e| e.error)?;

		Ok(RetVal::Converging(0))
	}

	fn asset_transfer_approved<E: Ext>(
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_assets::Config,
		<E::T as pallet_contracts::Config>::RuntimeCall: From<pallet_assets::Call<E::T>>,
	{
		trace!(
			target: "runtime",
			"[ChainExtension]|call|asset_transfer_approved"
		);

		let mut env = env.buf_in_buf_out();
		let (asset_id, owner, destination, amount): (
			<E::T as pallet_assets::Config>::AssetId,
			<E::T as frame_system::Config>::AccountId,
			<E::T as frame_system::Config>::AccountId,
			<E::T as pallet_assets::Config>::Balance,
		) = env.read_as()?;

		let ext = env.ext();
		let call: <E::T as pallet_contracts::Config>::RuntimeCall =
			pallet_assets::Call::<E::T>::transfer_approved {
				id: asset_id.into(),
				owner: <E::T as frame_system::Config>::Lookup::unlookup(owner),
				destination: <E::T as frame_system::Config>::Lookup::unlookup(destination),
				amount,
			}
			.into();
		ext.call_runtime(call).map_err(|e| e.error)?;

		Ok(RetVal::Converging(0))
	}

	fn asset_transfer_keep_alive<E: Ext>(
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError>
	where
		E::T: pallet_assets::Config,
		<E::T as pallet_contracts::Config>::RuntimeCall: From<pallet_assets::Call<E::T>>,
	{
		trace!(
			target: "runtime",
			"[ChainExtension]|call|asset_transfer_keep_alive"
		);

		let mut env = env.buf_in_buf_out();
		let (asset_id, target, amount): (
			<E::T as pallet_assets::Config>::AssetId,
			<E::T as frame_system::Config>::AccountId,
			<E::T as pallet_assets::Config>::Balance,
		) = env.read_as()?;

		let ext = env.ext();
		let call: <E::T as pallet_contracts::Config>::RuntimeCall =
			pallet_assets::Call::<E::T>::transfer_keep_alive {
				id: asset_id.into(),
				target: <E::T as frame_system::Config>::Lookup::unlookup(target),
				amount,
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
	<T as pallet_contracts::Config>::RuntimeCall: From<pallet_assets::Call<T>>,
{
	fn call<E>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
	{
		let func_id = env.func_id();
		match func_id {
			1 => Self::llm_force_transfer::<E>(env),
			2 => Self::asset_balance_of::<E>(env),
			3 => Self::asset_total_supply_of::<E>(env),
			4 => Self::asset_approve_transfer::<E>(env),
			5 => Self::asset_cancel_approval::<E>(env),
			6 => Self::asset_transfer::<E>(env),
			7 => Self::asset_transfer_approved::<E>(env),
			8 => Self::asset_transfer_keep_alive::<E>(env),
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
