#[allow(dead_code, unused_imports, non_camel_case_types)]
#[allow(clippy::all)]
pub mod api {
	#[allow(unused_imports)]
	mod root_mod {
		pub use super::*;
	}
	pub static PALLETS: [&str; 2usize] = ["LLDBridge", "LLMBridge"];
	#[doc = r" The error type returned when there is a runtime issue."]
	pub type DispatchError = runtime_types::sp_runtime::DispatchError;
	#[derive(
		:: subxt :: ext :: codec :: Decode,
		:: subxt :: ext :: codec :: Encode,
		:: subxt :: ext :: scale_decode :: DecodeAsType,
		:: subxt :: ext :: scale_encode :: EncodeAsType,
		Debug,
	)]
	#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
	#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
	pub enum Event {
		#[codec(index = 59)]
		LLDBridge(lld_bridge::Event),
		#[codec(index = 60)]
		LLMBridge(llm_bridge::Event),
	}
	impl ::subxt::events::RootEvent for Event {
		fn root_event(
			pallet_bytes: &[u8],
			pallet_name: &str,
			pallet_ty: u32,
			metadata: &::subxt::Metadata,
		) -> Result<Self, ::subxt::Error> {
			use ::subxt::metadata::DecodeWithMetadata;
			if pallet_name == "LLDBridge" {
				return Ok(Event::LLDBridge(lld_bridge::Event::decode_with_metadata(
					&mut &*pallet_bytes,
					pallet_ty,
					metadata,
				)?))
			}
			if pallet_name == "LLMBridge" {
				return Ok(Event::LLMBridge(llm_bridge::Event::decode_with_metadata(
					&mut &*pallet_bytes,
					pallet_ty,
					metadata,
				)?))
			}
			Err(::subxt::ext::scale_decode::Error::custom(format!(
				"Pallet name '{}' not found in root Event enum",
				pallet_name
			))
			.into())
		}
	}
	pub fn constants() -> ConstantsApi {
		ConstantsApi
	}
	pub fn storage() -> StorageApi {
		StorageApi
	}
	pub fn tx() -> TransactionApi {
		TransactionApi
	}
	pub struct ConstantsApi;
	impl ConstantsApi {
		pub fn lld_bridge(&self) -> lld_bridge::constants::ConstantsApi {
			lld_bridge::constants::ConstantsApi
		}
		pub fn llm_bridge(&self) -> llm_bridge::constants::ConstantsApi {
			llm_bridge::constants::ConstantsApi
		}
	}
	pub struct StorageApi;
	impl StorageApi {
		pub fn lld_bridge(&self) -> lld_bridge::storage::StorageApi {
			lld_bridge::storage::StorageApi
		}
		pub fn llm_bridge(&self) -> llm_bridge::storage::StorageApi {
			llm_bridge::storage::StorageApi
		}
	}
	pub struct TransactionApi;
	impl TransactionApi {
		pub fn lld_bridge(&self) -> lld_bridge::calls::TransactionApi {
			lld_bridge::calls::TransactionApi
		}
		pub fn llm_bridge(&self) -> llm_bridge::calls::TransactionApi {
			llm_bridge::calls::TransactionApi
		}
	}
	#[doc = r" check whether the Client you are using is aligned with the statically generated codegen."]
	pub fn validate_codegen<T: ::subxt::Config, C: ::subxt::client::OfflineClientT<T>>(
		client: &C,
	) -> Result<(), ::subxt::error::MetadataError> {
		let runtime_metadata_hash = client.metadata().metadata_hash(&PALLETS);
		if runtime_metadata_hash !=
			[
				218u8, 83u8, 188u8, 175u8, 60u8, 91u8, 25u8, 149u8, 239u8, 12u8, 58u8, 1u8, 112u8,
				113u8, 123u8, 51u8, 197u8, 223u8, 99u8, 167u8, 142u8, 13u8, 188u8, 195u8, 78u8,
				190u8, 233u8, 103u8, 6u8, 41u8, 115u8, 237u8,
			] {
			Err(::subxt::error::MetadataError::IncompatibleMetadata)
		} else {
			Ok(())
		}
	}
	pub mod lld_bridge {
		use super::{root_mod, runtime_types};
		#[doc = "Contains one variant per dispatchable that can be called by an extrinsic."]
		pub mod calls {
			use super::{root_mod, runtime_types};
			type DispatchError = runtime_types::sp_runtime::DispatchError;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct Deposit {
				pub amount: ::core::primitive::u128,
				pub eth_recipient: [::core::primitive::u8; 20usize],
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct VoteWithdraw {
				pub receipt_id: [::core::primitive::u8; 32usize],
				pub receipt: runtime_types::pallet_federated_bridge::IncomingReceipt<
					::subxt::utils::AccountId32,
					::core::primitive::u128,
				>,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct Withdraw {
				pub receipt_id: [::core::primitive::u8; 32usize],
			}
			#[derive(
				:: subxt :: ext :: codec :: CompactAs,
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetFee {
				pub amount: ::core::primitive::u128,
			}
			#[derive(
				:: subxt :: ext :: codec :: CompactAs,
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetVotesRequired {
				pub votes_required: ::core::primitive::u32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct AddRelay {
				pub relay: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct RemoveWatcher {
				pub watcher: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct RemoveRelay {
				pub relay: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct AddWatcher {
				pub watcher: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetState {
				pub state: runtime_types::pallet_federated_bridge::BridgeState,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct EmergencyStop;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetAdmin {
				pub admin: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetSuperAdmin {
				pub super_admin: ::subxt::utils::AccountId32,
			}
			pub struct TransactionApi;
			impl TransactionApi {
				#[doc = "Entrypoint for Substrate -> ETH transfer."]
				#[doc = "Takes `amount` of tokens (`T::Token`) from caller and issues a receipt for"]
				#[doc = "transferring them to `eth_recipient` on ETH side."]
				#[doc = ""]
				#[doc = "Deposits `OutgoingReceipt` event on success."]
				#[doc = ""]
				#[doc = "No fees other than transaction fees are taken at this point."]
				#[doc = ""]
				#[doc = "Can be called by any Signed origin."]
				#[doc = ""]
				#[doc = "Fails if bridge is stopped or caller has insufficient funds."]
				pub fn deposit(
					&self,
					amount: ::core::primitive::u128,
					eth_recipient: [::core::primitive::u8; 20usize],
				) -> ::subxt::tx::Payload<Deposit> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"deposit",
						Deposit { amount, eth_recipient },
						[
							189u8, 134u8, 11u8, 142u8, 140u8, 170u8, 15u8, 209u8, 16u8, 145u8,
							197u8, 126u8, 223u8, 52u8, 71u8, 110u8, 166u8, 102u8, 13u8, 102u8,
							167u8, 253u8, 33u8, 103u8, 245u8, 125u8, 70u8, 71u8, 205u8, 172u8,
							239u8, 114u8,
						],
					)
				}
				#[doc = "Cast vote for approving funds withdrawal for given Substrate -> ETH"]
				#[doc = "transfer receipt."]
				#[doc = ""]
				#[doc = "Can only be called by relays."]
				#[doc = ""]
				#[doc = "Fails if:"]
				#[doc = "* bridge is stopped"]
				#[doc = "* caller isn't an authorized relay"]
				#[doc = "* receipt was already processed and funds were withdrawn to recipient"]
				#[doc = "* there's already `T::MaxRelays` number of votes casted"]
				#[doc = ""]
				#[doc = "In case the `receipt` doesn't match previous `receipt` for given"]
				#[doc = "`receipt_id`, whole bridge will be immediately stopped and"]
				#[doc = "`StateChanged` event will be deposited."]
				#[doc = ""]
				#[doc = "Noop if caller already voted for given `receipt_id`."]
				#[doc = ""]
				#[doc = "Deposits `Vote` event on successful vote."]
				pub fn vote_withdraw(
					&self,
					receipt_id: [::core::primitive::u8; 32usize],
					receipt: runtime_types::pallet_federated_bridge::IncomingReceipt<
						::subxt::utils::AccountId32,
						::core::primitive::u128,
					>,
				) -> ::subxt::tx::Payload<VoteWithdraw> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"vote_withdraw",
						VoteWithdraw { receipt_id, receipt },
						[
							120u8, 210u8, 200u8, 163u8, 29u8, 18u8, 57u8, 220u8, 74u8, 234u8, 1u8,
							174u8, 118u8, 173u8, 24u8, 85u8, 11u8, 248u8, 111u8, 7u8, 217u8, 3u8,
							231u8, 144u8, 175u8, 125u8, 122u8, 128u8, 79u8, 102u8, 240u8, 81u8,
						],
					)
				}
				#[doc = "Claim tokens (`T::Token`) from approved ETH -> Substrate transfer."]
				#[doc = ""]
				#[doc = "Any Signed origin can call this, tokens will always be transferred"]
				#[doc = "to recipient specified by Ethereum side in Incoming Receipt."]
				#[doc = ""]
				#[doc = "Takes fee (in `T::Currency`) from caller and proportionally distributes to relays that"]
				#[doc = "casted vote on this receipt."]
				#[doc = ""]
				#[doc = "Will fail if:"]
				#[doc = "* bridge is stopped"]
				#[doc = "* receipt was already withdrawn earlier"]
				#[doc = "* receipt wasn't yet approved by relays"]
				#[doc = "* receipt is unknown on substrate yet"]
				#[doc = "* caller has insufficient funds to cover the fee"]
				pub fn withdraw(
					&self,
					receipt_id: [::core::primitive::u8; 32usize],
				) -> ::subxt::tx::Payload<Withdraw> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"withdraw",
						Withdraw { receipt_id },
						[
							29u8, 24u8, 169u8, 31u8, 77u8, 24u8, 147u8, 242u8, 206u8, 9u8, 249u8,
							220u8, 72u8, 102u8, 81u8, 52u8, 208u8, 169u8, 213u8, 253u8, 17u8, 30u8,
							146u8, 220u8, 197u8, 35u8, 1u8, 40u8, 1u8, 125u8, 23u8, 22u8,
						],
					)
				}
				#[doc = "Sets the withdrawal fee."]
				#[doc = ""]
				#[doc = "Can be called by Admin and SuperAdmin."]
				#[doc = ""]
				#[doc = "Should be set high enough to cover running costs for all relays."]
				pub fn set_fee(
					&self,
					amount: ::core::primitive::u128,
				) -> ::subxt::tx::Payload<SetFee> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"set_fee",
						SetFee { amount },
						[
							22u8, 212u8, 130u8, 17u8, 2u8, 51u8, 44u8, 152u8, 236u8, 135u8, 250u8,
							121u8, 138u8, 132u8, 104u8, 86u8, 100u8, 127u8, 69u8, 166u8, 140u8,
							229u8, 248u8, 244u8, 120u8, 192u8, 211u8, 53u8, 228u8, 186u8, 87u8,
							214u8,
						],
					)
				}
				#[doc = "Sets number of required votes to finish eth -> substrate transfer,"]
				#[doc = "process receipt and withdraw funds to recipient."]
				#[doc = ""]
				#[doc = "Can be called by SuperAdmin."]
				pub fn set_votes_required(
					&self,
					votes_required: ::core::primitive::u32,
				) -> ::subxt::tx::Payload<SetVotesRequired> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"set_votes_required",
						SetVotesRequired { votes_required },
						[
							244u8, 219u8, 18u8, 11u8, 31u8, 18u8, 76u8, 154u8, 99u8, 162u8, 140u8,
							223u8, 181u8, 209u8, 221u8, 89u8, 35u8, 48u8, 240u8, 171u8, 37u8,
							106u8, 48u8, 71u8, 18u8, 87u8, 108u8, 14u8, 179u8, 204u8, 28u8, 130u8,
						],
					)
				}
				#[doc = "Add account as authorized Relay."]
				#[doc = ""]
				#[doc = "Can be called by SuperAdmin."]
				#[doc = ""]
				#[doc = "Will fail if:"]
				#[doc = "* relay already exists"]
				#[doc = "* there's already `T::MaxRelays` relays"]
				pub fn add_relay(
					&self,
					relay: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<AddRelay> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"add_relay",
						AddRelay { relay },
						[
							168u8, 71u8, 157u8, 7u8, 173u8, 242u8, 133u8, 9u8, 92u8, 75u8, 124u8,
							186u8, 119u8, 60u8, 130u8, 168u8, 218u8, 96u8, 232u8, 203u8, 250u8,
							75u8, 209u8, 222u8, 219u8, 167u8, 201u8, 241u8, 187u8, 86u8, 205u8,
							26u8,
						],
					)
				}
				#[doc = "Remove account from authorized watchers."]
				#[doc = ""]
				#[doc = "Can be called by SuperAdmin."]
				#[doc = ""]
				#[doc = "Will fail if watcher doesn't exists."]
				pub fn remove_watcher(
					&self,
					watcher: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<RemoveWatcher> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"remove_watcher",
						RemoveWatcher { watcher },
						[
							124u8, 226u8, 46u8, 241u8, 144u8, 44u8, 174u8, 140u8, 221u8, 119u8,
							243u8, 161u8, 193u8, 39u8, 217u8, 89u8, 223u8, 94u8, 168u8, 219u8,
							57u8, 206u8, 195u8, 64u8, 173u8, 190u8, 123u8, 197u8, 183u8, 173u8,
							97u8, 77u8,
						],
					)
				}
				#[doc = "Remove account from authorized relays."]
				#[doc = ""]
				#[doc = "Can be called by Admin and SuperAdmin."]
				#[doc = ""]
				#[doc = "Will fail if relay doesn't exists."]
				pub fn remove_relay(
					&self,
					relay: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<RemoveRelay> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"remove_relay",
						RemoveRelay { relay },
						[
							31u8, 171u8, 94u8, 185u8, 16u8, 233u8, 162u8, 132u8, 254u8, 227u8,
							111u8, 36u8, 1u8, 108u8, 158u8, 79u8, 3u8, 230u8, 168u8, 224u8, 110u8,
							121u8, 101u8, 71u8, 137u8, 88u8, 18u8, 81u8, 200u8, 142u8, 112u8,
							131u8,
						],
					)
				}
				#[doc = "Add account as authorized Watcher."]
				#[doc = ""]
				#[doc = "Can be called by Admin and SuperAdmin."]
				#[doc = ""]
				#[doc = "Will fail if:"]
				#[doc = "* watcher already exists"]
				#[doc = "* there's already `T::MaxWatchers` relays"]
				pub fn add_watcher(
					&self,
					watcher: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<AddWatcher> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"add_watcher",
						AddWatcher { watcher },
						[
							78u8, 154u8, 36u8, 156u8, 46u8, 66u8, 9u8, 194u8, 224u8, 86u8, 203u8,
							94u8, 25u8, 101u8, 174u8, 211u8, 83u8, 159u8, 136u8, 253u8, 88u8, 65u8,
							75u8, 1u8, 243u8, 3u8, 35u8, 137u8, 129u8, 78u8, 75u8, 31u8,
						],
					)
				}
				#[doc = "Stop or resume bridge"]
				#[doc = ""]
				#[doc = "Can be called by Admin and SuperAdmin."]
				#[doc = ""]
				#[doc = "Deposits `StateChanged` event."]
				pub fn set_state(
					&self,
					state: runtime_types::pallet_federated_bridge::BridgeState,
				) -> ::subxt::tx::Payload<SetState> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"set_state",
						SetState { state },
						[
							15u8, 25u8, 137u8, 235u8, 158u8, 59u8, 11u8, 218u8, 251u8, 190u8, 43u8,
							110u8, 208u8, 252u8, 116u8, 163u8, 252u8, 125u8, 131u8, 12u8, 252u8,
							252u8, 129u8, 213u8, 233u8, 68u8, 91u8, 160u8, 187u8, 236u8, 109u8,
							106u8,
						],
					)
				}
				#[doc = "Emergency stop the bridge. This should only be used in case an"]
				#[doc = "invalid vote is detected."]
				#[doc = ""]
				#[doc = "Can be called by Watchers."]
				#[doc = ""]
				#[doc = "Deposits `EmergencyStop` and `StateChanged` events."]
				pub fn emergency_stop(&self) -> ::subxt::tx::Payload<EmergencyStop> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"emergency_stop",
						EmergencyStop {},
						[
							116u8, 45u8, 38u8, 56u8, 85u8, 26u8, 181u8, 82u8, 52u8, 6u8, 115u8,
							250u8, 121u8, 0u8, 99u8, 201u8, 117u8, 77u8, 62u8, 181u8, 24u8, 226u8,
							110u8, 177u8, 86u8, 221u8, 35u8, 149u8, 184u8, 210u8, 184u8, 45u8,
						],
					)
				}
				#[doc = "Set admin."]
				#[doc = ""]
				#[doc = "Admin has rights to:"]
				#[doc = "* remove relays"]
				#[doc = "* add watchers"]
				#[doc = "* stop and resume bridge"]
				#[doc = "* set withdrawal fee"]
				#[doc = "* change admin"]
				#[doc = ""]
				#[doc = "Can be called by ForceOrigin, SuperAdmin and Admin"]
				pub fn set_admin(
					&self,
					admin: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<SetAdmin> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"set_admin",
						SetAdmin { admin },
						[
							150u8, 233u8, 216u8, 121u8, 168u8, 205u8, 166u8, 76u8, 175u8, 32u8,
							191u8, 35u8, 167u8, 189u8, 99u8, 182u8, 228u8, 158u8, 49u8, 131u8,
							89u8, 210u8, 62u8, 188u8, 204u8, 100u8, 166u8, 75u8, 49u8, 45u8, 213u8,
							160u8,
						],
					)
				}
				#[doc = "Set super admin."]
				#[doc = ""]
				#[doc = "SuperAdmin has rights to:"]
				#[doc = "* add relays"]
				#[doc = "* remove watchers"]
				#[doc = "* change admin and superadmin"]
				#[doc = "* change number of required votes"]
				#[doc = "* all rights of Admin"]
				#[doc = ""]
				#[doc = "Can be called by ForceOrigin and SuperAdmin"]
				pub fn set_super_admin(
					&self,
					super_admin: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<SetSuperAdmin> {
					::subxt::tx::Payload::new_static(
						"LLDBridge",
						"set_super_admin",
						SetSuperAdmin { super_admin },
						[
							5u8, 156u8, 47u8, 227u8, 25u8, 175u8, 129u8, 60u8, 14u8, 24u8, 166u8,
							82u8, 147u8, 158u8, 197u8, 90u8, 240u8, 144u8, 224u8, 147u8, 38u8,
							217u8, 98u8, 149u8, 192u8, 143u8, 49u8, 85u8, 93u8, 85u8, 242u8, 84u8,
						],
					)
				}
			}
		}
		#[doc = "\n\t\t\tThe [event](https://docs.substrate.io/main-docs/build/events-errors/) emitted\n\t\t\tby this pallet.\n\t\t\t"]
		pub type Event = runtime_types::pallet_federated_bridge::pallet::Event;
		pub mod events {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Receipt for substrate -> eth transfer"]
			pub struct OutgoingReceipt {
				pub amount: ::core::primitive::u128,
				pub eth_recipient: [::core::primitive::u8; 20usize],
			}
			impl ::subxt::events::StaticEvent for OutgoingReceipt {
				const PALLET: &'static str = "LLDBridge";
				const EVENT: &'static str = "OutgoingReceipt";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Relay voted on approving given eth -> substrate receipt for processing"]
			pub struct Vote {
				pub relay: ::subxt::utils::AccountId32,
				pub receipt_id: [::core::primitive::u8; 32usize],
				pub block_number: ::core::primitive::u32,
			}
			impl ::subxt::events::StaticEvent for Vote {
				const PALLET: &'static str = "LLDBridge";
				const EVENT: &'static str = "Vote";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Incoming Receipt got approved for withdrawal"]
			pub struct Approved(pub [::core::primitive::u8; 32usize]);
			impl ::subxt::events::StaticEvent for Approved {
				const PALLET: &'static str = "LLDBridge";
				const EVENT: &'static str = "Approved";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Incoming Receipt was processed, eth -> substrate transfer complete"]
			pub struct Processed(pub [::core::primitive::u8; 32usize]);
			impl ::subxt::events::StaticEvent for Processed {
				const PALLET: &'static str = "LLDBridge";
				const EVENT: &'static str = "Processed";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Bridge state was changed by watcher, admin or superadmin"]
			pub struct StateChanged(pub runtime_types::pallet_federated_bridge::BridgeState);
			impl ::subxt::events::StaticEvent for StateChanged {
				const PALLET: &'static str = "LLDBridge";
				const EVENT: &'static str = "StateChanged";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Bridge was stopped by watcher after detecting invalid vote by relay"]
			pub struct EmergencyStop;
			impl ::subxt::events::StaticEvent for EmergencyStop {
				const PALLET: &'static str = "LLDBridge";
				const EVENT: &'static str = "EmergencyStop";
			}
		}
		pub mod storage {
			use super::runtime_types;
			pub struct StorageApi;
			impl StorageApi {
				#[doc = " List of accounts that act as Relays"]
				pub fn relays(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::sp_core::bounded::bounded_vec::BoundedVec<
						::subxt::utils::AccountId32,
					>,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"Relays",
						vec![],
						[
							232u8, 168u8, 174u8, 160u8, 12u8, 86u8, 162u8, 81u8, 55u8, 101u8, 2u8,
							60u8, 100u8, 190u8, 126u8, 84u8, 146u8, 0u8, 48u8, 148u8, 78u8, 198u8,
							203u8, 38u8, 150u8, 249u8, 159u8, 163u8, 245u8, 18u8, 156u8, 91u8,
						],
					)
				}
				#[doc = " List of accounts that act as Watchers"]
				pub fn watchers(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::sp_core::bounded::bounded_vec::BoundedVec<
						::subxt::utils::AccountId32,
					>,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"Watchers",
						vec![],
						[
							136u8, 214u8, 69u8, 190u8, 244u8, 89u8, 241u8, 23u8, 30u8, 4u8, 173u8,
							185u8, 227u8, 68u8, 82u8, 218u8, 80u8, 145u8, 119u8, 57u8, 167u8,
							154u8, 48u8, 70u8, 235u8, 126u8, 65u8, 153u8, 158u8, 246u8, 19u8, 70u8,
						],
					)
				}
				#[doc = " Number of Relay votes required to approve withdrawal"]
				pub fn votes_required(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					::core::primitive::u32,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"VotesRequired",
						vec![],
						[
							58u8, 185u8, 167u8, 116u8, 150u8, 219u8, 32u8, 87u8, 247u8, 247u8,
							96u8, 13u8, 153u8, 121u8, 197u8, 169u8, 185u8, 180u8, 75u8, 70u8,
							125u8, 196u8, 66u8, 56u8, 88u8, 193u8, 119u8, 20u8, 251u8, 105u8, 27u8,
							219u8,
						],
					)
				}
				#[doc = " Fee taken on withdrawal from caller and distributed to Voters"]
				pub fn fee(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					::core::primitive::u128,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"Fee",
						vec![],
						[
							70u8, 52u8, 4u8, 185u8, 249u8, 200u8, 188u8, 16u8, 12u8, 167u8, 62u8,
							36u8, 58u8, 29u8, 92u8, 80u8, 161u8, 33u8, 15u8, 241u8, 253u8, 248u8,
							53u8, 232u8, 9u8, 101u8, 192u8, 189u8, 181u8, 210u8, 209u8, 204u8,
						],
					)
				}
				#[doc = " Bridge state"]
				pub fn state(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::BridgeState,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"State",
						vec![],
						[
							205u8, 98u8, 17u8, 5u8, 222u8, 186u8, 181u8, 11u8, 65u8, 188u8, 37u8,
							33u8, 107u8, 130u8, 198u8, 247u8, 9u8, 63u8, 167u8, 244u8, 114u8, 95u8,
							184u8, 71u8, 190u8, 27u8, 188u8, 2u8, 191u8, 240u8, 61u8, 85u8,
						],
					)
				}
				#[doc = " Incoming Receipts - details on eth -> substrate transfers"]
				pub fn incoming_receipts(
					&self,
					_0: impl ::std::borrow::Borrow<[::core::primitive::u8; 32usize]>,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::IncomingReceipt<
						::subxt::utils::AccountId32,
						::core::primitive::u128,
					>,
					::subxt::storage::address::Yes,
					(),
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"IncomingReceipts",
						vec![::subxt::storage::address::make_static_storage_map_key(_0.borrow())],
						[
							192u8, 110u8, 16u8, 94u8, 104u8, 237u8, 37u8, 136u8, 24u8, 156u8, 90u8,
							129u8, 220u8, 20u8, 117u8, 3u8, 69u8, 69u8, 154u8, 219u8, 30u8, 24u8,
							5u8, 33u8, 175u8, 116u8, 68u8, 228u8, 184u8, 246u8, 76u8, 73u8,
						],
					)
				}
				#[doc = " Incoming Receipts - details on eth -> substrate transfers"]
				pub fn incoming_receipts_root(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::IncomingReceipt<
						::subxt::utils::AccountId32,
						::core::primitive::u128,
					>,
					(),
					(),
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"IncomingReceipts",
						Vec::new(),
						[
							192u8, 110u8, 16u8, 94u8, 104u8, 237u8, 37u8, 136u8, 24u8, 156u8, 90u8,
							129u8, 220u8, 20u8, 117u8, 3u8, 69u8, 69u8, 154u8, 219u8, 30u8, 24u8,
							5u8, 33u8, 175u8, 116u8, 68u8, 228u8, 184u8, 246u8, 76u8, 73u8,
						],
					)
				}
				#[doc = " Status of incoming receipts - eth -> substrate transfers"]
				pub fn status_of(
					&self,
					_0: impl ::std::borrow::Borrow<[::core::primitive::u8; 32usize]>,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::IncomingReceiptStatus<
						::core::primitive::u32,
					>,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"StatusOf",
						vec![::subxt::storage::address::make_static_storage_map_key(_0.borrow())],
						[
							231u8, 100u8, 219u8, 146u8, 114u8, 92u8, 188u8, 192u8, 88u8, 173u8,
							120u8, 30u8, 130u8, 34u8, 172u8, 154u8, 245u8, 71u8, 44u8, 18u8, 113u8,
							217u8, 141u8, 36u8, 42u8, 25u8, 158u8, 191u8, 220u8, 11u8, 163u8,
							157u8,
						],
					)
				}
				#[doc = " Status of incoming receipts - eth -> substrate transfers"]
				pub fn status_of_root(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::IncomingReceiptStatus<
						::core::primitive::u32,
					>,
					(),
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"StatusOf",
						Vec::new(),
						[
							231u8, 100u8, 219u8, 146u8, 114u8, 92u8, 188u8, 192u8, 88u8, 173u8,
							120u8, 30u8, 130u8, 34u8, 172u8, 154u8, 245u8, 71u8, 44u8, 18u8, 113u8,
							217u8, 141u8, 36u8, 42u8, 25u8, 158u8, 191u8, 220u8, 11u8, 163u8,
							157u8,
						],
					)
				}
				#[doc = " List of relays that voted for approval of given receipt"]
				pub fn voting(
					&self,
					_0: impl ::std::borrow::Borrow<[::core::primitive::u8; 32usize]>,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::sp_core::bounded::bounded_vec::BoundedVec<
						::subxt::utils::AccountId32,
					>,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"Voting",
						vec![::subxt::storage::address::make_static_storage_map_key(_0.borrow())],
						[
							127u8, 180u8, 185u8, 192u8, 159u8, 163u8, 221u8, 232u8, 227u8, 3u8,
							151u8, 107u8, 20u8, 50u8, 183u8, 113u8, 173u8, 51u8, 97u8, 108u8,
							179u8, 122u8, 147u8, 9u8, 106u8, 254u8, 126u8, 239u8, 70u8, 103u8,
							70u8, 125u8,
						],
					)
				}
				#[doc = " List of relays that voted for approval of given receipt"]
				pub fn voting_root(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::sp_core::bounded::bounded_vec::BoundedVec<
						::subxt::utils::AccountId32,
					>,
					(),
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"Voting",
						Vec::new(),
						[
							127u8, 180u8, 185u8, 192u8, 159u8, 163u8, 221u8, 232u8, 227u8, 3u8,
							151u8, 107u8, 20u8, 50u8, 183u8, 113u8, 173u8, 51u8, 97u8, 108u8,
							179u8, 122u8, 147u8, 9u8, 106u8, 254u8, 126u8, 239u8, 70u8, 103u8,
							70u8, 125u8,
						],
					)
				}
				#[doc = " Account that can use calls that potentially lower bridge security"]
				pub fn super_admin(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					::subxt::utils::AccountId32,
					::subxt::storage::address::Yes,
					(),
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"SuperAdmin",
						vec![],
						[
							88u8, 56u8, 216u8, 145u8, 133u8, 221u8, 68u8, 1u8, 194u8, 159u8, 92u8,
							137u8, 220u8, 179u8, 148u8, 189u8, 7u8, 115u8, 216u8, 166u8, 7u8, 0u8,
							97u8, 232u8, 106u8, 150u8, 242u8, 158u8, 83u8, 47u8, 163u8, 122u8,
						],
					)
				}
				#[doc = " Account that can use calls that changes config and restarts bridge"]
				pub fn admin(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					::subxt::utils::AccountId32,
					::subxt::storage::address::Yes,
					(),
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"Admin",
						vec![],
						[
							10u8, 87u8, 141u8, 118u8, 70u8, 57u8, 247u8, 80u8, 16u8, 12u8, 236u8,
							78u8, 189u8, 5u8, 37u8, 108u8, 169u8, 56u8, 2u8, 99u8, 185u8, 187u8,
							169u8, 78u8, 89u8, 92u8, 25u8, 245u8, 143u8, 203u8, 101u8, 168u8,
						],
					)
				}
				#[doc = " Counter used to track rate limiting"]
				#[doc = " Decays linearly each block"]
				pub fn withdrawal_counter(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					(::core::primitive::u128, ::core::primitive::u32),
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLDBridge",
						"WithdrawalCounter",
						vec![],
						[
							202u8, 130u8, 208u8, 131u8, 247u8, 113u8, 52u8, 243u8, 187u8, 176u8,
							169u8, 40u8, 59u8, 33u8, 64u8, 132u8, 33u8, 10u8, 41u8, 134u8, 170u8,
							53u8, 118u8, 84u8, 81u8, 6u8, 241u8, 248u8, 24u8, 115u8, 82u8, 32u8,
						],
					)
				}
			}
		}
		pub mod constants {
			use super::runtime_types;
			pub struct ConstantsApi;
			impl ConstantsApi {
				#[doc = " PalletId used to derive bridge's AccountId (wallet)"]
				pub fn pallet_id(
					&self,
				) -> ::subxt::constants::Address<runtime_types::frame_support::PalletId> {
					::subxt::constants::Address::new_static(
						"LLDBridge",
						"PalletId",
						[
							139u8, 109u8, 228u8, 151u8, 252u8, 32u8, 130u8, 69u8, 112u8, 154u8,
							174u8, 45u8, 83u8, 245u8, 51u8, 132u8, 173u8, 5u8, 186u8, 24u8, 243u8,
							9u8, 12u8, 214u8, 80u8, 74u8, 69u8, 189u8, 30u8, 94u8, 22u8, 39u8,
						],
					)
				}
				#[doc = " Maximum number of relays"]
				pub fn max_relays(&self) -> ::subxt::constants::Address<::core::primitive::u32> {
					::subxt::constants::Address::new_static(
						"LLDBridge",
						"MaxRelays",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
				#[doc = " Maximum number of watchers"]
				pub fn max_watchers(&self) -> ::subxt::constants::Address<::core::primitive::u32> {
					::subxt::constants::Address::new_static(
						"LLDBridge",
						"MaxWatchers",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
				#[doc = " Maximum number of tokens that can be locked in pallet (a.k.a."]
				#[doc = " bridged to ETH)"]
				pub fn max_total_locked(
					&self,
				) -> ::subxt::constants::Address<::core::primitive::u128> {
					::subxt::constants::Address::new_static(
						"LLDBridge",
						"MaxTotalLocked",
						[
							84u8, 157u8, 140u8, 4u8, 93u8, 57u8, 29u8, 133u8, 105u8, 200u8, 214u8,
							27u8, 144u8, 208u8, 218u8, 160u8, 130u8, 109u8, 101u8, 54u8, 210u8,
							136u8, 71u8, 63u8, 49u8, 237u8, 234u8, 15u8, 178u8, 98u8, 148u8, 156u8,
						],
					)
				}
				#[doc = " Delay between getting approval from relays and actually unlocking"]
				#[doc = " withdrawal for eth -> substrate transfer."]
				#[doc = " Gives watchers time to stop bridge."]
				pub fn withdrawal_delay(
					&self,
				) -> ::subxt::constants::Address<::core::primitive::u32> {
					::subxt::constants::Address::new_static(
						"LLDBridge",
						"WithdrawalDelay",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
				#[doc = " Rate limit parameters This is implemented as The Leaky Bucket as a"]
				#[doc = " Meter algorithm - https://en.wikipedia.org/wiki/Leaky_bucket.:w"]
				#[doc = " First parameter is the max counter (a.k.a. max burst, max single"]
				#[doc = " withdrawal)"]
				#[doc = " Second parameter is the decay rate (a.k.a. after reaching max, how"]
				#[doc = " much can be withdrawn per block)"]
				pub fn withdrawal_rate_limit(
					&self,
				) -> ::subxt::constants::Address<(::core::primitive::u128, ::core::primitive::u128)>
				{
					::subxt::constants::Address::new_static(
						"LLDBridge",
						"WithdrawalRateLimit",
						[
							238u8, 107u8, 194u8, 177u8, 164u8, 108u8, 188u8, 7u8, 212u8, 177u8,
							156u8, 4u8, 111u8, 127u8, 113u8, 78u8, 239u8, 237u8, 85u8, 223u8,
							173u8, 146u8, 48u8, 35u8, 10u8, 30u8, 216u8, 37u8, 105u8, 197u8, 163u8,
							35u8,
						],
					)
				}
			}
		}
	}
	pub mod llm_bridge {
		use super::{root_mod, runtime_types};
		#[doc = "Contains one variant per dispatchable that can be called by an extrinsic."]
		pub mod calls {
			use super::{root_mod, runtime_types};
			type DispatchError = runtime_types::sp_runtime::DispatchError;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct Deposit {
				pub amount: ::core::primitive::u128,
				pub eth_recipient: [::core::primitive::u8; 20usize],
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct VoteWithdraw {
				pub receipt_id: [::core::primitive::u8; 32usize],
				pub receipt: runtime_types::pallet_federated_bridge::IncomingReceipt<
					::subxt::utils::AccountId32,
					::core::primitive::u128,
				>,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct Withdraw {
				pub receipt_id: [::core::primitive::u8; 32usize],
			}
			#[derive(
				:: subxt :: ext :: codec :: CompactAs,
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetFee {
				pub amount: ::core::primitive::u128,
			}
			#[derive(
				:: subxt :: ext :: codec :: CompactAs,
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetVotesRequired {
				pub votes_required: ::core::primitive::u32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct AddRelay {
				pub relay: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct RemoveWatcher {
				pub watcher: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct RemoveRelay {
				pub relay: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct AddWatcher {
				pub watcher: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetState {
				pub state: runtime_types::pallet_federated_bridge::BridgeState,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct EmergencyStop;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetAdmin {
				pub admin: ::subxt::utils::AccountId32,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct SetSuperAdmin {
				pub super_admin: ::subxt::utils::AccountId32,
			}
			pub struct TransactionApi;
			impl TransactionApi {
				#[doc = "Entrypoint for Substrate -> ETH transfer."]
				#[doc = "Takes `amount` of tokens (`T::Token`) from caller and issues a receipt for"]
				#[doc = "transferring them to `eth_recipient` on ETH side."]
				#[doc = ""]
				#[doc = "Deposits `OutgoingReceipt` event on success."]
				#[doc = ""]
				#[doc = "No fees other than transaction fees are taken at this point."]
				#[doc = ""]
				#[doc = "Can be called by any Signed origin."]
				#[doc = ""]
				#[doc = "Fails if bridge is stopped or caller has insufficient funds."]
				pub fn deposit(
					&self,
					amount: ::core::primitive::u128,
					eth_recipient: [::core::primitive::u8; 20usize],
				) -> ::subxt::tx::Payload<Deposit> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"deposit",
						Deposit { amount, eth_recipient },
						[
							189u8, 134u8, 11u8, 142u8, 140u8, 170u8, 15u8, 209u8, 16u8, 145u8,
							197u8, 126u8, 223u8, 52u8, 71u8, 110u8, 166u8, 102u8, 13u8, 102u8,
							167u8, 253u8, 33u8, 103u8, 245u8, 125u8, 70u8, 71u8, 205u8, 172u8,
							239u8, 114u8,
						],
					)
				}
				#[doc = "Cast vote for approving funds withdrawal for given Substrate -> ETH"]
				#[doc = "transfer receipt."]
				#[doc = ""]
				#[doc = "Can only be called by relays."]
				#[doc = ""]
				#[doc = "Fails if:"]
				#[doc = "* bridge is stopped"]
				#[doc = "* caller isn't an authorized relay"]
				#[doc = "* receipt was already processed and funds were withdrawn to recipient"]
				#[doc = "* there's already `T::MaxRelays` number of votes casted"]
				#[doc = ""]
				#[doc = "In case the `receipt` doesn't match previous `receipt` for given"]
				#[doc = "`receipt_id`, whole bridge will be immediately stopped and"]
				#[doc = "`StateChanged` event will be deposited."]
				#[doc = ""]
				#[doc = "Noop if caller already voted for given `receipt_id`."]
				#[doc = ""]
				#[doc = "Deposits `Vote` event on successful vote."]
				pub fn vote_withdraw(
					&self,
					receipt_id: [::core::primitive::u8; 32usize],
					receipt: runtime_types::pallet_federated_bridge::IncomingReceipt<
						::subxt::utils::AccountId32,
						::core::primitive::u128,
					>,
				) -> ::subxt::tx::Payload<VoteWithdraw> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"vote_withdraw",
						VoteWithdraw { receipt_id, receipt },
						[
							120u8, 210u8, 200u8, 163u8, 29u8, 18u8, 57u8, 220u8, 74u8, 234u8, 1u8,
							174u8, 118u8, 173u8, 24u8, 85u8, 11u8, 248u8, 111u8, 7u8, 217u8, 3u8,
							231u8, 144u8, 175u8, 125u8, 122u8, 128u8, 79u8, 102u8, 240u8, 81u8,
						],
					)
				}
				#[doc = "Claim tokens (`T::Token`) from approved ETH -> Substrate transfer."]
				#[doc = ""]
				#[doc = "Any Signed origin can call this, tokens will always be transferred"]
				#[doc = "to recipient specified by Ethereum side in Incoming Receipt."]
				#[doc = ""]
				#[doc = "Takes fee (in `T::Currency`) from caller and proportionally distributes to relays that"]
				#[doc = "casted vote on this receipt."]
				#[doc = ""]
				#[doc = "Will fail if:"]
				#[doc = "* bridge is stopped"]
				#[doc = "* receipt was already withdrawn earlier"]
				#[doc = "* receipt wasn't yet approved by relays"]
				#[doc = "* receipt is unknown on substrate yet"]
				#[doc = "* caller has insufficient funds to cover the fee"]
				pub fn withdraw(
					&self,
					receipt_id: [::core::primitive::u8; 32usize],
				) -> ::subxt::tx::Payload<Withdraw> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"withdraw",
						Withdraw { receipt_id },
						[
							29u8, 24u8, 169u8, 31u8, 77u8, 24u8, 147u8, 242u8, 206u8, 9u8, 249u8,
							220u8, 72u8, 102u8, 81u8, 52u8, 208u8, 169u8, 213u8, 253u8, 17u8, 30u8,
							146u8, 220u8, 197u8, 35u8, 1u8, 40u8, 1u8, 125u8, 23u8, 22u8,
						],
					)
				}
				#[doc = "Sets the withdrawal fee."]
				#[doc = ""]
				#[doc = "Can be called by Admin and SuperAdmin."]
				#[doc = ""]
				#[doc = "Should be set high enough to cover running costs for all relays."]
				pub fn set_fee(
					&self,
					amount: ::core::primitive::u128,
				) -> ::subxt::tx::Payload<SetFee> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"set_fee",
						SetFee { amount },
						[
							22u8, 212u8, 130u8, 17u8, 2u8, 51u8, 44u8, 152u8, 236u8, 135u8, 250u8,
							121u8, 138u8, 132u8, 104u8, 86u8, 100u8, 127u8, 69u8, 166u8, 140u8,
							229u8, 248u8, 244u8, 120u8, 192u8, 211u8, 53u8, 228u8, 186u8, 87u8,
							214u8,
						],
					)
				}
				#[doc = "Sets number of required votes to finish eth -> substrate transfer,"]
				#[doc = "process receipt and withdraw funds to recipient."]
				#[doc = ""]
				#[doc = "Can be called by SuperAdmin."]
				pub fn set_votes_required(
					&self,
					votes_required: ::core::primitive::u32,
				) -> ::subxt::tx::Payload<SetVotesRequired> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"set_votes_required",
						SetVotesRequired { votes_required },
						[
							244u8, 219u8, 18u8, 11u8, 31u8, 18u8, 76u8, 154u8, 99u8, 162u8, 140u8,
							223u8, 181u8, 209u8, 221u8, 89u8, 35u8, 48u8, 240u8, 171u8, 37u8,
							106u8, 48u8, 71u8, 18u8, 87u8, 108u8, 14u8, 179u8, 204u8, 28u8, 130u8,
						],
					)
				}
				#[doc = "Add account as authorized Relay."]
				#[doc = ""]
				#[doc = "Can be called by SuperAdmin."]
				#[doc = ""]
				#[doc = "Will fail if:"]
				#[doc = "* relay already exists"]
				#[doc = "* there's already `T::MaxRelays` relays"]
				pub fn add_relay(
					&self,
					relay: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<AddRelay> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"add_relay",
						AddRelay { relay },
						[
							168u8, 71u8, 157u8, 7u8, 173u8, 242u8, 133u8, 9u8, 92u8, 75u8, 124u8,
							186u8, 119u8, 60u8, 130u8, 168u8, 218u8, 96u8, 232u8, 203u8, 250u8,
							75u8, 209u8, 222u8, 219u8, 167u8, 201u8, 241u8, 187u8, 86u8, 205u8,
							26u8,
						],
					)
				}
				#[doc = "Remove account from authorized watchers."]
				#[doc = ""]
				#[doc = "Can be called by SuperAdmin."]
				#[doc = ""]
				#[doc = "Will fail if watcher doesn't exists."]
				pub fn remove_watcher(
					&self,
					watcher: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<RemoveWatcher> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"remove_watcher",
						RemoveWatcher { watcher },
						[
							124u8, 226u8, 46u8, 241u8, 144u8, 44u8, 174u8, 140u8, 221u8, 119u8,
							243u8, 161u8, 193u8, 39u8, 217u8, 89u8, 223u8, 94u8, 168u8, 219u8,
							57u8, 206u8, 195u8, 64u8, 173u8, 190u8, 123u8, 197u8, 183u8, 173u8,
							97u8, 77u8,
						],
					)
				}
				#[doc = "Remove account from authorized relays."]
				#[doc = ""]
				#[doc = "Can be called by Admin and SuperAdmin."]
				#[doc = ""]
				#[doc = "Will fail if relay doesn't exists."]
				pub fn remove_relay(
					&self,
					relay: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<RemoveRelay> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"remove_relay",
						RemoveRelay { relay },
						[
							31u8, 171u8, 94u8, 185u8, 16u8, 233u8, 162u8, 132u8, 254u8, 227u8,
							111u8, 36u8, 1u8, 108u8, 158u8, 79u8, 3u8, 230u8, 168u8, 224u8, 110u8,
							121u8, 101u8, 71u8, 137u8, 88u8, 18u8, 81u8, 200u8, 142u8, 112u8,
							131u8,
						],
					)
				}
				#[doc = "Add account as authorized Watcher."]
				#[doc = ""]
				#[doc = "Can be called by Admin and SuperAdmin."]
				#[doc = ""]
				#[doc = "Will fail if:"]
				#[doc = "* watcher already exists"]
				#[doc = "* there's already `T::MaxWatchers` relays"]
				pub fn add_watcher(
					&self,
					watcher: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<AddWatcher> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"add_watcher",
						AddWatcher { watcher },
						[
							78u8, 154u8, 36u8, 156u8, 46u8, 66u8, 9u8, 194u8, 224u8, 86u8, 203u8,
							94u8, 25u8, 101u8, 174u8, 211u8, 83u8, 159u8, 136u8, 253u8, 88u8, 65u8,
							75u8, 1u8, 243u8, 3u8, 35u8, 137u8, 129u8, 78u8, 75u8, 31u8,
						],
					)
				}
				#[doc = "Stop or resume bridge"]
				#[doc = ""]
				#[doc = "Can be called by Admin and SuperAdmin."]
				#[doc = ""]
				#[doc = "Deposits `StateChanged` event."]
				pub fn set_state(
					&self,
					state: runtime_types::pallet_federated_bridge::BridgeState,
				) -> ::subxt::tx::Payload<SetState> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"set_state",
						SetState { state },
						[
							15u8, 25u8, 137u8, 235u8, 158u8, 59u8, 11u8, 218u8, 251u8, 190u8, 43u8,
							110u8, 208u8, 252u8, 116u8, 163u8, 252u8, 125u8, 131u8, 12u8, 252u8,
							252u8, 129u8, 213u8, 233u8, 68u8, 91u8, 160u8, 187u8, 236u8, 109u8,
							106u8,
						],
					)
				}
				#[doc = "Emergency stop the bridge. This should only be used in case an"]
				#[doc = "invalid vote is detected."]
				#[doc = ""]
				#[doc = "Can be called by Watchers."]
				#[doc = ""]
				#[doc = "Deposits `EmergencyStop` and `StateChanged` events."]
				pub fn emergency_stop(&self) -> ::subxt::tx::Payload<EmergencyStop> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"emergency_stop",
						EmergencyStop {},
						[
							116u8, 45u8, 38u8, 56u8, 85u8, 26u8, 181u8, 82u8, 52u8, 6u8, 115u8,
							250u8, 121u8, 0u8, 99u8, 201u8, 117u8, 77u8, 62u8, 181u8, 24u8, 226u8,
							110u8, 177u8, 86u8, 221u8, 35u8, 149u8, 184u8, 210u8, 184u8, 45u8,
						],
					)
				}
				#[doc = "Set admin."]
				#[doc = ""]
				#[doc = "Admin has rights to:"]
				#[doc = "* remove relays"]
				#[doc = "* add watchers"]
				#[doc = "* stop and resume bridge"]
				#[doc = "* set withdrawal fee"]
				#[doc = "* change admin"]
				#[doc = ""]
				#[doc = "Can be called by ForceOrigin, SuperAdmin and Admin"]
				pub fn set_admin(
					&self,
					admin: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<SetAdmin> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"set_admin",
						SetAdmin { admin },
						[
							150u8, 233u8, 216u8, 121u8, 168u8, 205u8, 166u8, 76u8, 175u8, 32u8,
							191u8, 35u8, 167u8, 189u8, 99u8, 182u8, 228u8, 158u8, 49u8, 131u8,
							89u8, 210u8, 62u8, 188u8, 204u8, 100u8, 166u8, 75u8, 49u8, 45u8, 213u8,
							160u8,
						],
					)
				}
				#[doc = "Set super admin."]
				#[doc = ""]
				#[doc = "SuperAdmin has rights to:"]
				#[doc = "* add relays"]
				#[doc = "* remove watchers"]
				#[doc = "* change admin and superadmin"]
				#[doc = "* change number of required votes"]
				#[doc = "* all rights of Admin"]
				#[doc = ""]
				#[doc = "Can be called by ForceOrigin and SuperAdmin"]
				pub fn set_super_admin(
					&self,
					super_admin: ::subxt::utils::AccountId32,
				) -> ::subxt::tx::Payload<SetSuperAdmin> {
					::subxt::tx::Payload::new_static(
						"LLMBridge",
						"set_super_admin",
						SetSuperAdmin { super_admin },
						[
							5u8, 156u8, 47u8, 227u8, 25u8, 175u8, 129u8, 60u8, 14u8, 24u8, 166u8,
							82u8, 147u8, 158u8, 197u8, 90u8, 240u8, 144u8, 224u8, 147u8, 38u8,
							217u8, 98u8, 149u8, 192u8, 143u8, 49u8, 85u8, 93u8, 85u8, 242u8, 84u8,
						],
					)
				}
			}
		}
		#[doc = "\n\t\t\tThe [event](https://docs.substrate.io/main-docs/build/events-errors/) emitted\n\t\t\tby this pallet.\n\t\t\t"]
		pub type Event = runtime_types::pallet_federated_bridge::pallet::Event;
		pub mod events {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Receipt for substrate -> eth transfer"]
			pub struct OutgoingReceipt {
				pub amount: ::core::primitive::u128,
				pub eth_recipient: [::core::primitive::u8; 20usize],
			}
			impl ::subxt::events::StaticEvent for OutgoingReceipt {
				const PALLET: &'static str = "LLMBridge";
				const EVENT: &'static str = "OutgoingReceipt";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Relay voted on approving given eth -> substrate receipt for processing"]
			pub struct Vote {
				pub relay: ::subxt::utils::AccountId32,
				pub receipt_id: [::core::primitive::u8; 32usize],
				pub block_number: ::core::primitive::u32,
			}
			impl ::subxt::events::StaticEvent for Vote {
				const PALLET: &'static str = "LLMBridge";
				const EVENT: &'static str = "Vote";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Incoming Receipt got approved for withdrawal"]
			pub struct Approved(pub [::core::primitive::u8; 32usize]);
			impl ::subxt::events::StaticEvent for Approved {
				const PALLET: &'static str = "LLMBridge";
				const EVENT: &'static str = "Approved";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Incoming Receipt was processed, eth -> substrate transfer complete"]
			pub struct Processed(pub [::core::primitive::u8; 32usize]);
			impl ::subxt::events::StaticEvent for Processed {
				const PALLET: &'static str = "LLMBridge";
				const EVENT: &'static str = "Processed";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Bridge state was changed by watcher, admin or superadmin"]
			pub struct StateChanged(pub runtime_types::pallet_federated_bridge::BridgeState);
			impl ::subxt::events::StaticEvent for StateChanged {
				const PALLET: &'static str = "LLMBridge";
				const EVENT: &'static str = "StateChanged";
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			#[doc = "Bridge was stopped by watcher after detecting invalid vote by relay"]
			pub struct EmergencyStop;
			impl ::subxt::events::StaticEvent for EmergencyStop {
				const PALLET: &'static str = "LLMBridge";
				const EVENT: &'static str = "EmergencyStop";
			}
		}
		pub mod storage {
			use super::runtime_types;
			pub struct StorageApi;
			impl StorageApi {
				#[doc = " List of accounts that act as Relays"]
				pub fn relays(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::sp_core::bounded::bounded_vec::BoundedVec<
						::subxt::utils::AccountId32,
					>,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"Relays",
						vec![],
						[
							232u8, 168u8, 174u8, 160u8, 12u8, 86u8, 162u8, 81u8, 55u8, 101u8, 2u8,
							60u8, 100u8, 190u8, 126u8, 84u8, 146u8, 0u8, 48u8, 148u8, 78u8, 198u8,
							203u8, 38u8, 150u8, 249u8, 159u8, 163u8, 245u8, 18u8, 156u8, 91u8,
						],
					)
				}
				#[doc = " List of accounts that act as Watchers"]
				pub fn watchers(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::sp_core::bounded::bounded_vec::BoundedVec<
						::subxt::utils::AccountId32,
					>,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"Watchers",
						vec![],
						[
							136u8, 214u8, 69u8, 190u8, 244u8, 89u8, 241u8, 23u8, 30u8, 4u8, 173u8,
							185u8, 227u8, 68u8, 82u8, 218u8, 80u8, 145u8, 119u8, 57u8, 167u8,
							154u8, 48u8, 70u8, 235u8, 126u8, 65u8, 153u8, 158u8, 246u8, 19u8, 70u8,
						],
					)
				}
				#[doc = " Number of Relay votes required to approve withdrawal"]
				pub fn votes_required(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					::core::primitive::u32,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"VotesRequired",
						vec![],
						[
							58u8, 185u8, 167u8, 116u8, 150u8, 219u8, 32u8, 87u8, 247u8, 247u8,
							96u8, 13u8, 153u8, 121u8, 197u8, 169u8, 185u8, 180u8, 75u8, 70u8,
							125u8, 196u8, 66u8, 56u8, 88u8, 193u8, 119u8, 20u8, 251u8, 105u8, 27u8,
							219u8,
						],
					)
				}
				#[doc = " Fee taken on withdrawal from caller and distributed to Voters"]
				pub fn fee(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					::core::primitive::u128,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"Fee",
						vec![],
						[
							70u8, 52u8, 4u8, 185u8, 249u8, 200u8, 188u8, 16u8, 12u8, 167u8, 62u8,
							36u8, 58u8, 29u8, 92u8, 80u8, 161u8, 33u8, 15u8, 241u8, 253u8, 248u8,
							53u8, 232u8, 9u8, 101u8, 192u8, 189u8, 181u8, 210u8, 209u8, 204u8,
						],
					)
				}
				#[doc = " Bridge state"]
				pub fn state(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::BridgeState,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"State",
						vec![],
						[
							205u8, 98u8, 17u8, 5u8, 222u8, 186u8, 181u8, 11u8, 65u8, 188u8, 37u8,
							33u8, 107u8, 130u8, 198u8, 247u8, 9u8, 63u8, 167u8, 244u8, 114u8, 95u8,
							184u8, 71u8, 190u8, 27u8, 188u8, 2u8, 191u8, 240u8, 61u8, 85u8,
						],
					)
				}
				#[doc = " Incoming Receipts - details on eth -> substrate transfers"]
				pub fn incoming_receipts(
					&self,
					_0: impl ::std::borrow::Borrow<[::core::primitive::u8; 32usize]>,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::IncomingReceipt<
						::subxt::utils::AccountId32,
						::core::primitive::u128,
					>,
					::subxt::storage::address::Yes,
					(),
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"IncomingReceipts",
						vec![::subxt::storage::address::make_static_storage_map_key(_0.borrow())],
						[
							192u8, 110u8, 16u8, 94u8, 104u8, 237u8, 37u8, 136u8, 24u8, 156u8, 90u8,
							129u8, 220u8, 20u8, 117u8, 3u8, 69u8, 69u8, 154u8, 219u8, 30u8, 24u8,
							5u8, 33u8, 175u8, 116u8, 68u8, 228u8, 184u8, 246u8, 76u8, 73u8,
						],
					)
				}
				#[doc = " Incoming Receipts - details on eth -> substrate transfers"]
				pub fn incoming_receipts_root(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::IncomingReceipt<
						::subxt::utils::AccountId32,
						::core::primitive::u128,
					>,
					(),
					(),
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"IncomingReceipts",
						Vec::new(),
						[
							192u8, 110u8, 16u8, 94u8, 104u8, 237u8, 37u8, 136u8, 24u8, 156u8, 90u8,
							129u8, 220u8, 20u8, 117u8, 3u8, 69u8, 69u8, 154u8, 219u8, 30u8, 24u8,
							5u8, 33u8, 175u8, 116u8, 68u8, 228u8, 184u8, 246u8, 76u8, 73u8,
						],
					)
				}
				#[doc = " Status of incoming receipts - eth -> substrate transfers"]
				pub fn status_of(
					&self,
					_0: impl ::std::borrow::Borrow<[::core::primitive::u8; 32usize]>,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::IncomingReceiptStatus<
						::core::primitive::u32,
					>,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"StatusOf",
						vec![::subxt::storage::address::make_static_storage_map_key(_0.borrow())],
						[
							231u8, 100u8, 219u8, 146u8, 114u8, 92u8, 188u8, 192u8, 88u8, 173u8,
							120u8, 30u8, 130u8, 34u8, 172u8, 154u8, 245u8, 71u8, 44u8, 18u8, 113u8,
							217u8, 141u8, 36u8, 42u8, 25u8, 158u8, 191u8, 220u8, 11u8, 163u8,
							157u8,
						],
					)
				}
				#[doc = " Status of incoming receipts - eth -> substrate transfers"]
				pub fn status_of_root(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::pallet_federated_bridge::IncomingReceiptStatus<
						::core::primitive::u32,
					>,
					(),
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"StatusOf",
						Vec::new(),
						[
							231u8, 100u8, 219u8, 146u8, 114u8, 92u8, 188u8, 192u8, 88u8, 173u8,
							120u8, 30u8, 130u8, 34u8, 172u8, 154u8, 245u8, 71u8, 44u8, 18u8, 113u8,
							217u8, 141u8, 36u8, 42u8, 25u8, 158u8, 191u8, 220u8, 11u8, 163u8,
							157u8,
						],
					)
				}
				#[doc = " List of relays that voted for approval of given receipt"]
				pub fn voting(
					&self,
					_0: impl ::std::borrow::Borrow<[::core::primitive::u8; 32usize]>,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::sp_core::bounded::bounded_vec::BoundedVec<
						::subxt::utils::AccountId32,
					>,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"Voting",
						vec![::subxt::storage::address::make_static_storage_map_key(_0.borrow())],
						[
							127u8, 180u8, 185u8, 192u8, 159u8, 163u8, 221u8, 232u8, 227u8, 3u8,
							151u8, 107u8, 20u8, 50u8, 183u8, 113u8, 173u8, 51u8, 97u8, 108u8,
							179u8, 122u8, 147u8, 9u8, 106u8, 254u8, 126u8, 239u8, 70u8, 103u8,
							70u8, 125u8,
						],
					)
				}
				#[doc = " List of relays that voted for approval of given receipt"]
				pub fn voting_root(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					runtime_types::sp_core::bounded::bounded_vec::BoundedVec<
						::subxt::utils::AccountId32,
					>,
					(),
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"Voting",
						Vec::new(),
						[
							127u8, 180u8, 185u8, 192u8, 159u8, 163u8, 221u8, 232u8, 227u8, 3u8,
							151u8, 107u8, 20u8, 50u8, 183u8, 113u8, 173u8, 51u8, 97u8, 108u8,
							179u8, 122u8, 147u8, 9u8, 106u8, 254u8, 126u8, 239u8, 70u8, 103u8,
							70u8, 125u8,
						],
					)
				}
				#[doc = " Account that can use calls that potentially lower bridge security"]
				pub fn super_admin(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					::subxt::utils::AccountId32,
					::subxt::storage::address::Yes,
					(),
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"SuperAdmin",
						vec![],
						[
							88u8, 56u8, 216u8, 145u8, 133u8, 221u8, 68u8, 1u8, 194u8, 159u8, 92u8,
							137u8, 220u8, 179u8, 148u8, 189u8, 7u8, 115u8, 216u8, 166u8, 7u8, 0u8,
							97u8, 232u8, 106u8, 150u8, 242u8, 158u8, 83u8, 47u8, 163u8, 122u8,
						],
					)
				}
				#[doc = " Account that can use calls that changes config and restarts bridge"]
				pub fn admin(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					::subxt::utils::AccountId32,
					::subxt::storage::address::Yes,
					(),
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"Admin",
						vec![],
						[
							10u8, 87u8, 141u8, 118u8, 70u8, 57u8, 247u8, 80u8, 16u8, 12u8, 236u8,
							78u8, 189u8, 5u8, 37u8, 108u8, 169u8, 56u8, 2u8, 99u8, 185u8, 187u8,
							169u8, 78u8, 89u8, 92u8, 25u8, 245u8, 143u8, 203u8, 101u8, 168u8,
						],
					)
				}
				#[doc = " Counter used to track rate limiting"]
				#[doc = " Decays linearly each block"]
				pub fn withdrawal_counter(
					&self,
				) -> ::subxt::storage::address::Address<
					::subxt::storage::address::StaticStorageMapKey,
					(::core::primitive::u128, ::core::primitive::u32),
					::subxt::storage::address::Yes,
					::subxt::storage::address::Yes,
					(),
				> {
					::subxt::storage::address::Address::new_static(
						"LLMBridge",
						"WithdrawalCounter",
						vec![],
						[
							202u8, 130u8, 208u8, 131u8, 247u8, 113u8, 52u8, 243u8, 187u8, 176u8,
							169u8, 40u8, 59u8, 33u8, 64u8, 132u8, 33u8, 10u8, 41u8, 134u8, 170u8,
							53u8, 118u8, 84u8, 81u8, 6u8, 241u8, 248u8, 24u8, 115u8, 82u8, 32u8,
						],
					)
				}
			}
		}
		pub mod constants {
			use super::runtime_types;
			pub struct ConstantsApi;
			impl ConstantsApi {
				#[doc = " PalletId used to derive bridge's AccountId (wallet)"]
				pub fn pallet_id(
					&self,
				) -> ::subxt::constants::Address<runtime_types::frame_support::PalletId> {
					::subxt::constants::Address::new_static(
						"LLMBridge",
						"PalletId",
						[
							139u8, 109u8, 228u8, 151u8, 252u8, 32u8, 130u8, 69u8, 112u8, 154u8,
							174u8, 45u8, 83u8, 245u8, 51u8, 132u8, 173u8, 5u8, 186u8, 24u8, 243u8,
							9u8, 12u8, 214u8, 80u8, 74u8, 69u8, 189u8, 30u8, 94u8, 22u8, 39u8,
						],
					)
				}
				#[doc = " Maximum number of relays"]
				pub fn max_relays(&self) -> ::subxt::constants::Address<::core::primitive::u32> {
					::subxt::constants::Address::new_static(
						"LLMBridge",
						"MaxRelays",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
				#[doc = " Maximum number of watchers"]
				pub fn max_watchers(&self) -> ::subxt::constants::Address<::core::primitive::u32> {
					::subxt::constants::Address::new_static(
						"LLMBridge",
						"MaxWatchers",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
				#[doc = " Maximum number of tokens that can be locked in pallet (a.k.a."]
				#[doc = " bridged to ETH)"]
				pub fn max_total_locked(
					&self,
				) -> ::subxt::constants::Address<::core::primitive::u128> {
					::subxt::constants::Address::new_static(
						"LLMBridge",
						"MaxTotalLocked",
						[
							84u8, 157u8, 140u8, 4u8, 93u8, 57u8, 29u8, 133u8, 105u8, 200u8, 214u8,
							27u8, 144u8, 208u8, 218u8, 160u8, 130u8, 109u8, 101u8, 54u8, 210u8,
							136u8, 71u8, 63u8, 49u8, 237u8, 234u8, 15u8, 178u8, 98u8, 148u8, 156u8,
						],
					)
				}
				#[doc = " Delay between getting approval from relays and actually unlocking"]
				#[doc = " withdrawal for eth -> substrate transfer."]
				#[doc = " Gives watchers time to stop bridge."]
				pub fn withdrawal_delay(
					&self,
				) -> ::subxt::constants::Address<::core::primitive::u32> {
					::subxt::constants::Address::new_static(
						"LLMBridge",
						"WithdrawalDelay",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
				#[doc = " Rate limit parameters This is implemented as The Leaky Bucket as a"]
				#[doc = " Meter algorithm - https://en.wikipedia.org/wiki/Leaky_bucket.:w"]
				#[doc = " First parameter is the max counter (a.k.a. max burst, max single"]
				#[doc = " withdrawal)"]
				#[doc = " Second parameter is the decay rate (a.k.a. after reaching max, how"]
				#[doc = " much can be withdrawn per block)"]
				pub fn withdrawal_rate_limit(
					&self,
				) -> ::subxt::constants::Address<(::core::primitive::u128, ::core::primitive::u128)>
				{
					::subxt::constants::Address::new_static(
						"LLMBridge",
						"WithdrawalRateLimit",
						[
							238u8, 107u8, 194u8, 177u8, 164u8, 108u8, 188u8, 7u8, 212u8, 177u8,
							156u8, 4u8, 111u8, 127u8, 113u8, 78u8, 239u8, 237u8, 85u8, 223u8,
							173u8, 146u8, 48u8, 35u8, 10u8, 30u8, 216u8, 37u8, 105u8, 197u8, 163u8,
							35u8,
						],
					)
				}
			}
		}
	}
	pub mod runtime_types {
		use super::runtime_types;
		pub mod frame_support {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct PalletId(pub [::core::primitive::u8; 8usize]);
		}
		pub mod frame_system {
			use super::runtime_types;
			pub mod extensions {
				use super::runtime_types;
				pub mod check_genesis {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct CheckGenesis;
				}
				pub mod check_mortality {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct CheckMortality(pub runtime_types::sp_runtime::generic::era::Era);
				}
				pub mod check_non_zero_sender {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct CheckNonZeroSender;
				}
				pub mod check_nonce {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct CheckNonce(#[codec(compact)] pub ::core::primitive::u32);
				}
				pub mod check_spec_version {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct CheckSpecVersion;
				}
				pub mod check_tx_version {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct CheckTxVersion;
				}
				pub mod check_weight {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct CheckWeight;
				}
			}
		}
		pub mod kitchensink_runtime {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct Runtime;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub enum RuntimeCall {
				#[codec(index = 59)]
				LLDBridge(runtime_types::pallet_federated_bridge::pallet::Call),
				#[codec(index = 60)]
				LLMBridge(runtime_types::pallet_federated_bridge::pallet::Call),
			}
		}
		pub mod pallet_asset_tx_payment {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct ChargeAssetTxPayment {
				#[codec(compact)]
				pub tip: ::core::primitive::u128,
				pub asset_id: ::core::option::Option<::core::primitive::u32>,
			}
		}
		pub mod pallet_federated_bridge {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: codec :: Decode,
					:: subxt :: ext :: codec :: Encode,
					:: subxt :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
				#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
				#[doc = "Contains one variant per dispatchable that can be called by an extrinsic."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Entrypoint for Substrate -> ETH transfer."]
					#[doc = "Takes `amount` of tokens (`T::Token`) from caller and issues a receipt for"]
					#[doc = "transferring them to `eth_recipient` on ETH side."]
					#[doc = ""]
					#[doc = "Deposits `OutgoingReceipt` event on success."]
					#[doc = ""]
					#[doc = "No fees other than transaction fees are taken at this point."]
					#[doc = ""]
					#[doc = "Can be called by any Signed origin."]
					#[doc = ""]
					#[doc = "Fails if bridge is stopped or caller has insufficient funds."]
					deposit {
						amount: ::core::primitive::u128,
						eth_recipient: [::core::primitive::u8; 20usize],
					},
					#[codec(index = 1)]
					#[doc = "Cast vote for approving funds withdrawal for given Substrate -> ETH"]
					#[doc = "transfer receipt."]
					#[doc = ""]
					#[doc = "Can only be called by relays."]
					#[doc = ""]
					#[doc = "Fails if:"]
					#[doc = "* bridge is stopped"]
					#[doc = "* caller isn't an authorized relay"]
					#[doc = "* receipt was already processed and funds were withdrawn to recipient"]
					#[doc = "* there's already `T::MaxRelays` number of votes casted"]
					#[doc = ""]
					#[doc = "In case the `receipt` doesn't match previous `receipt` for given"]
					#[doc = "`receipt_id`, whole bridge will be immediately stopped and"]
					#[doc = "`StateChanged` event will be deposited."]
					#[doc = ""]
					#[doc = "Noop if caller already voted for given `receipt_id`."]
					#[doc = ""]
					#[doc = "Deposits `Vote` event on successful vote."]
					vote_withdraw {
						receipt_id: [::core::primitive::u8; 32usize],
						receipt: runtime_types::pallet_federated_bridge::IncomingReceipt<
							::subxt::utils::AccountId32,
							::core::primitive::u128,
						>,
					},
					#[codec(index = 2)]
					#[doc = "Claim tokens (`T::Token`) from approved ETH -> Substrate transfer."]
					#[doc = ""]
					#[doc = "Any Signed origin can call this, tokens will always be transferred"]
					#[doc = "to recipient specified by Ethereum side in Incoming Receipt."]
					#[doc = ""]
					#[doc = "Takes fee (in `T::Currency`) from caller and proportionally distributes to relays that"]
					#[doc = "casted vote on this receipt."]
					#[doc = ""]
					#[doc = "Will fail if:"]
					#[doc = "* bridge is stopped"]
					#[doc = "* receipt was already withdrawn earlier"]
					#[doc = "* receipt wasn't yet approved by relays"]
					#[doc = "* receipt is unknown on substrate yet"]
					#[doc = "* caller has insufficient funds to cover the fee"]
					withdraw { receipt_id: [::core::primitive::u8; 32usize] },
					#[codec(index = 3)]
					#[doc = "Sets the withdrawal fee."]
					#[doc = ""]
					#[doc = "Can be called by Admin and SuperAdmin."]
					#[doc = ""]
					#[doc = "Should be set high enough to cover running costs for all relays."]
					set_fee { amount: ::core::primitive::u128 },
					#[codec(index = 4)]
					#[doc = "Sets number of required votes to finish eth -> substrate transfer,"]
					#[doc = "process receipt and withdraw funds to recipient."]
					#[doc = ""]
					#[doc = "Can be called by SuperAdmin."]
					set_votes_required { votes_required: ::core::primitive::u32 },
					#[codec(index = 5)]
					#[doc = "Add account as authorized Relay."]
					#[doc = ""]
					#[doc = "Can be called by SuperAdmin."]
					#[doc = ""]
					#[doc = "Will fail if:"]
					#[doc = "* relay already exists"]
					#[doc = "* there's already `T::MaxRelays` relays"]
					add_relay { relay: ::subxt::utils::AccountId32 },
					#[codec(index = 6)]
					#[doc = "Remove account from authorized watchers."]
					#[doc = ""]
					#[doc = "Can be called by SuperAdmin."]
					#[doc = ""]
					#[doc = "Will fail if watcher doesn't exists."]
					remove_watcher { watcher: ::subxt::utils::AccountId32 },
					#[codec(index = 7)]
					#[doc = "Remove account from authorized relays."]
					#[doc = ""]
					#[doc = "Can be called by Admin and SuperAdmin."]
					#[doc = ""]
					#[doc = "Will fail if relay doesn't exists."]
					remove_relay { relay: ::subxt::utils::AccountId32 },
					#[codec(index = 8)]
					#[doc = "Add account as authorized Watcher."]
					#[doc = ""]
					#[doc = "Can be called by Admin and SuperAdmin."]
					#[doc = ""]
					#[doc = "Will fail if:"]
					#[doc = "* watcher already exists"]
					#[doc = "* there's already `T::MaxWatchers` relays"]
					add_watcher { watcher: ::subxt::utils::AccountId32 },
					#[codec(index = 9)]
					#[doc = "Stop or resume bridge"]
					#[doc = ""]
					#[doc = "Can be called by Admin and SuperAdmin."]
					#[doc = ""]
					#[doc = "Deposits `StateChanged` event."]
					set_state { state: runtime_types::pallet_federated_bridge::BridgeState },
					#[codec(index = 10)]
					#[doc = "Emergency stop the bridge. This should only be used in case an"]
					#[doc = "invalid vote is detected."]
					#[doc = ""]
					#[doc = "Can be called by Watchers."]
					#[doc = ""]
					#[doc = "Deposits `EmergencyStop` and `StateChanged` events."]
					emergency_stop,
					#[codec(index = 11)]
					#[doc = "Set admin."]
					#[doc = ""]
					#[doc = "Admin has rights to:"]
					#[doc = "* remove relays"]
					#[doc = "* add watchers"]
					#[doc = "* stop and resume bridge"]
					#[doc = "* set withdrawal fee"]
					#[doc = "* change admin"]
					#[doc = ""]
					#[doc = "Can be called by ForceOrigin, SuperAdmin and Admin"]
					set_admin { admin: ::subxt::utils::AccountId32 },
					#[codec(index = 12)]
					#[doc = "Set super admin."]
					#[doc = ""]
					#[doc = "SuperAdmin has rights to:"]
					#[doc = "* add relays"]
					#[doc = "* remove watchers"]
					#[doc = "* change admin and superadmin"]
					#[doc = "* change number of required votes"]
					#[doc = "* all rights of Admin"]
					#[doc = ""]
					#[doc = "Can be called by ForceOrigin and SuperAdmin"]
					set_super_admin { super_admin: ::subxt::utils::AccountId32 },
				}
				#[derive(
					:: subxt :: ext :: codec :: Decode,
					:: subxt :: ext :: codec :: Encode,
					:: subxt :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
				#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
				#[doc = "\n\t\t\tCustom [dispatch errors](https://docs.substrate.io/main-docs/build/events-errors/)\n\t\t\tof this pallet.\n\t\t\t"]
				pub enum Error {
					#[codec(index = 0)]
					#[doc = "Relay/Watcher already exists"]
					AlreadyExists,
					#[codec(index = 1)]
					#[doc = "Incoming Receipt already processed and funds withdrawn"]
					AlreadyProcessed,
					#[codec(index = 2)]
					#[doc = "Bridge is stopped"]
					BridgeStopped,
					#[codec(index = 3)]
					#[doc = "This incoming receipt id is unknown"]
					UnknownReceiptId,
					#[codec(index = 4)]
					#[doc = "Invalid relay"]
					InvalidRelay,
					#[codec(index = 5)]
					#[doc = "Invalid watcher"]
					InvalidWatcher,
					#[codec(index = 6)]
					#[doc = "Incoming receipt not approved for processing yet"]
					NotApproved,
					#[codec(index = 7)]
					#[doc = "Too many votes"]
					TooManyVotes,
					#[codec(index = 8)]
					#[doc = "Too many watchers"]
					TooManyWatchers,
					#[codec(index = 9)]
					#[doc = "Too many relays"]
					TooManyRelays,
					#[codec(index = 10)]
					#[doc = "Caller is unauthorized for this action"]
					Unauthorized,
					#[codec(index = 11)]
					#[doc = "Not enough time passed since incoming receipt approval"]
					TooSoon,
					#[codec(index = 12)]
					#[doc = "Too many tokens withdrawn in short time from bridge, try again later"]
					RateLimited,
					#[codec(index = 13)]
					#[doc = "Too much locked in pallet already"]
					TooMuchLocked,
				}
				#[derive(
					:: subxt :: ext :: codec :: Decode,
					:: subxt :: ext :: codec :: Encode,
					:: subxt :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
				#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
				#[doc = "\n\t\t\tThe [event](https://docs.substrate.io/main-docs/build/events-errors/) emitted\n\t\t\tby this pallet.\n\t\t\t"]
				pub enum Event {
					#[codec(index = 0)]
					#[doc = "Receipt for substrate -> eth transfer"]
					OutgoingReceipt {
						amount: ::core::primitive::u128,
						eth_recipient: [::core::primitive::u8; 20usize],
					},
					#[codec(index = 1)]
					#[doc = "Relay voted on approving given eth -> substrate receipt for processing"]
					Vote {
						relay: ::subxt::utils::AccountId32,
						receipt_id: [::core::primitive::u8; 32usize],
						block_number: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					#[doc = "Incoming Receipt got approved for withdrawal"]
					Approved([::core::primitive::u8; 32usize]),
					#[codec(index = 3)]
					#[doc = "Incoming Receipt was processed, eth -> substrate transfer complete"]
					Processed([::core::primitive::u8; 32usize]),
					#[codec(index = 4)]
					#[doc = "Bridge state was changed by watcher, admin or superadmin"]
					StateChanged(runtime_types::pallet_federated_bridge::BridgeState),
					#[codec(index = 5)]
					#[doc = "Bridge was stopped by watcher after detecting invalid vote by relay"]
					EmergencyStop,
				}
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub enum BridgeState {
				#[codec(index = 0)]
				Stopped,
				#[codec(index = 1)]
				Active,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct IncomingReceipt<_0, _1> {
				pub eth_block_number: ::core::primitive::u64,
				pub substrate_recipient: _0,
				pub amount: _1,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub enum IncomingReceiptStatus<_0> {
				#[codec(index = 0)]
				Voting,
				#[codec(index = 1)]
				Approved(_0),
				#[codec(index = 2)]
				Processed(_0),
			}
		}
		pub mod sp_arithmetic {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub enum ArithmeticError {
				#[codec(index = 0)]
				Underflow,
				#[codec(index = 1)]
				Overflow,
				#[codec(index = 2)]
				DivisionByZero,
			}
		}
		pub mod sp_core {
			use super::runtime_types;
			pub mod bounded {
				use super::runtime_types;
				pub mod bounded_vec {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct BoundedVec<_0>(pub ::std::vec::Vec<_0>);
				}
			}
			pub mod ecdsa {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: codec :: Decode,
					:: subxt :: ext :: codec :: Encode,
					:: subxt :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
				#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
				pub struct Signature(pub [::core::primitive::u8; 65usize]);
			}
			pub mod ed25519 {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: codec :: Decode,
					:: subxt :: ext :: codec :: Encode,
					:: subxt :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
				#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
				pub struct Signature(pub [::core::primitive::u8; 64usize]);
			}
			pub mod sr25519 {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: codec :: Decode,
					:: subxt :: ext :: codec :: Encode,
					:: subxt :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
				#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
				pub struct Signature(pub [::core::primitive::u8; 64usize]);
			}
		}
		pub mod sp_runtime {
			use super::runtime_types;
			pub mod generic {
				use super::runtime_types;
				pub mod era {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub enum Era {
						#[codec(index = 0)]
						Immortal,
						#[codec(index = 1)]
						Mortal1(::core::primitive::u8),
						#[codec(index = 2)]
						Mortal2(::core::primitive::u8),
						#[codec(index = 3)]
						Mortal3(::core::primitive::u8),
						#[codec(index = 4)]
						Mortal4(::core::primitive::u8),
						#[codec(index = 5)]
						Mortal5(::core::primitive::u8),
						#[codec(index = 6)]
						Mortal6(::core::primitive::u8),
						#[codec(index = 7)]
						Mortal7(::core::primitive::u8),
						#[codec(index = 8)]
						Mortal8(::core::primitive::u8),
						#[codec(index = 9)]
						Mortal9(::core::primitive::u8),
						#[codec(index = 10)]
						Mortal10(::core::primitive::u8),
						#[codec(index = 11)]
						Mortal11(::core::primitive::u8),
						#[codec(index = 12)]
						Mortal12(::core::primitive::u8),
						#[codec(index = 13)]
						Mortal13(::core::primitive::u8),
						#[codec(index = 14)]
						Mortal14(::core::primitive::u8),
						#[codec(index = 15)]
						Mortal15(::core::primitive::u8),
						#[codec(index = 16)]
						Mortal16(::core::primitive::u8),
						#[codec(index = 17)]
						Mortal17(::core::primitive::u8),
						#[codec(index = 18)]
						Mortal18(::core::primitive::u8),
						#[codec(index = 19)]
						Mortal19(::core::primitive::u8),
						#[codec(index = 20)]
						Mortal20(::core::primitive::u8),
						#[codec(index = 21)]
						Mortal21(::core::primitive::u8),
						#[codec(index = 22)]
						Mortal22(::core::primitive::u8),
						#[codec(index = 23)]
						Mortal23(::core::primitive::u8),
						#[codec(index = 24)]
						Mortal24(::core::primitive::u8),
						#[codec(index = 25)]
						Mortal25(::core::primitive::u8),
						#[codec(index = 26)]
						Mortal26(::core::primitive::u8),
						#[codec(index = 27)]
						Mortal27(::core::primitive::u8),
						#[codec(index = 28)]
						Mortal28(::core::primitive::u8),
						#[codec(index = 29)]
						Mortal29(::core::primitive::u8),
						#[codec(index = 30)]
						Mortal30(::core::primitive::u8),
						#[codec(index = 31)]
						Mortal31(::core::primitive::u8),
						#[codec(index = 32)]
						Mortal32(::core::primitive::u8),
						#[codec(index = 33)]
						Mortal33(::core::primitive::u8),
						#[codec(index = 34)]
						Mortal34(::core::primitive::u8),
						#[codec(index = 35)]
						Mortal35(::core::primitive::u8),
						#[codec(index = 36)]
						Mortal36(::core::primitive::u8),
						#[codec(index = 37)]
						Mortal37(::core::primitive::u8),
						#[codec(index = 38)]
						Mortal38(::core::primitive::u8),
						#[codec(index = 39)]
						Mortal39(::core::primitive::u8),
						#[codec(index = 40)]
						Mortal40(::core::primitive::u8),
						#[codec(index = 41)]
						Mortal41(::core::primitive::u8),
						#[codec(index = 42)]
						Mortal42(::core::primitive::u8),
						#[codec(index = 43)]
						Mortal43(::core::primitive::u8),
						#[codec(index = 44)]
						Mortal44(::core::primitive::u8),
						#[codec(index = 45)]
						Mortal45(::core::primitive::u8),
						#[codec(index = 46)]
						Mortal46(::core::primitive::u8),
						#[codec(index = 47)]
						Mortal47(::core::primitive::u8),
						#[codec(index = 48)]
						Mortal48(::core::primitive::u8),
						#[codec(index = 49)]
						Mortal49(::core::primitive::u8),
						#[codec(index = 50)]
						Mortal50(::core::primitive::u8),
						#[codec(index = 51)]
						Mortal51(::core::primitive::u8),
						#[codec(index = 52)]
						Mortal52(::core::primitive::u8),
						#[codec(index = 53)]
						Mortal53(::core::primitive::u8),
						#[codec(index = 54)]
						Mortal54(::core::primitive::u8),
						#[codec(index = 55)]
						Mortal55(::core::primitive::u8),
						#[codec(index = 56)]
						Mortal56(::core::primitive::u8),
						#[codec(index = 57)]
						Mortal57(::core::primitive::u8),
						#[codec(index = 58)]
						Mortal58(::core::primitive::u8),
						#[codec(index = 59)]
						Mortal59(::core::primitive::u8),
						#[codec(index = 60)]
						Mortal60(::core::primitive::u8),
						#[codec(index = 61)]
						Mortal61(::core::primitive::u8),
						#[codec(index = 62)]
						Mortal62(::core::primitive::u8),
						#[codec(index = 63)]
						Mortal63(::core::primitive::u8),
						#[codec(index = 64)]
						Mortal64(::core::primitive::u8),
						#[codec(index = 65)]
						Mortal65(::core::primitive::u8),
						#[codec(index = 66)]
						Mortal66(::core::primitive::u8),
						#[codec(index = 67)]
						Mortal67(::core::primitive::u8),
						#[codec(index = 68)]
						Mortal68(::core::primitive::u8),
						#[codec(index = 69)]
						Mortal69(::core::primitive::u8),
						#[codec(index = 70)]
						Mortal70(::core::primitive::u8),
						#[codec(index = 71)]
						Mortal71(::core::primitive::u8),
						#[codec(index = 72)]
						Mortal72(::core::primitive::u8),
						#[codec(index = 73)]
						Mortal73(::core::primitive::u8),
						#[codec(index = 74)]
						Mortal74(::core::primitive::u8),
						#[codec(index = 75)]
						Mortal75(::core::primitive::u8),
						#[codec(index = 76)]
						Mortal76(::core::primitive::u8),
						#[codec(index = 77)]
						Mortal77(::core::primitive::u8),
						#[codec(index = 78)]
						Mortal78(::core::primitive::u8),
						#[codec(index = 79)]
						Mortal79(::core::primitive::u8),
						#[codec(index = 80)]
						Mortal80(::core::primitive::u8),
						#[codec(index = 81)]
						Mortal81(::core::primitive::u8),
						#[codec(index = 82)]
						Mortal82(::core::primitive::u8),
						#[codec(index = 83)]
						Mortal83(::core::primitive::u8),
						#[codec(index = 84)]
						Mortal84(::core::primitive::u8),
						#[codec(index = 85)]
						Mortal85(::core::primitive::u8),
						#[codec(index = 86)]
						Mortal86(::core::primitive::u8),
						#[codec(index = 87)]
						Mortal87(::core::primitive::u8),
						#[codec(index = 88)]
						Mortal88(::core::primitive::u8),
						#[codec(index = 89)]
						Mortal89(::core::primitive::u8),
						#[codec(index = 90)]
						Mortal90(::core::primitive::u8),
						#[codec(index = 91)]
						Mortal91(::core::primitive::u8),
						#[codec(index = 92)]
						Mortal92(::core::primitive::u8),
						#[codec(index = 93)]
						Mortal93(::core::primitive::u8),
						#[codec(index = 94)]
						Mortal94(::core::primitive::u8),
						#[codec(index = 95)]
						Mortal95(::core::primitive::u8),
						#[codec(index = 96)]
						Mortal96(::core::primitive::u8),
						#[codec(index = 97)]
						Mortal97(::core::primitive::u8),
						#[codec(index = 98)]
						Mortal98(::core::primitive::u8),
						#[codec(index = 99)]
						Mortal99(::core::primitive::u8),
						#[codec(index = 100)]
						Mortal100(::core::primitive::u8),
						#[codec(index = 101)]
						Mortal101(::core::primitive::u8),
						#[codec(index = 102)]
						Mortal102(::core::primitive::u8),
						#[codec(index = 103)]
						Mortal103(::core::primitive::u8),
						#[codec(index = 104)]
						Mortal104(::core::primitive::u8),
						#[codec(index = 105)]
						Mortal105(::core::primitive::u8),
						#[codec(index = 106)]
						Mortal106(::core::primitive::u8),
						#[codec(index = 107)]
						Mortal107(::core::primitive::u8),
						#[codec(index = 108)]
						Mortal108(::core::primitive::u8),
						#[codec(index = 109)]
						Mortal109(::core::primitive::u8),
						#[codec(index = 110)]
						Mortal110(::core::primitive::u8),
						#[codec(index = 111)]
						Mortal111(::core::primitive::u8),
						#[codec(index = 112)]
						Mortal112(::core::primitive::u8),
						#[codec(index = 113)]
						Mortal113(::core::primitive::u8),
						#[codec(index = 114)]
						Mortal114(::core::primitive::u8),
						#[codec(index = 115)]
						Mortal115(::core::primitive::u8),
						#[codec(index = 116)]
						Mortal116(::core::primitive::u8),
						#[codec(index = 117)]
						Mortal117(::core::primitive::u8),
						#[codec(index = 118)]
						Mortal118(::core::primitive::u8),
						#[codec(index = 119)]
						Mortal119(::core::primitive::u8),
						#[codec(index = 120)]
						Mortal120(::core::primitive::u8),
						#[codec(index = 121)]
						Mortal121(::core::primitive::u8),
						#[codec(index = 122)]
						Mortal122(::core::primitive::u8),
						#[codec(index = 123)]
						Mortal123(::core::primitive::u8),
						#[codec(index = 124)]
						Mortal124(::core::primitive::u8),
						#[codec(index = 125)]
						Mortal125(::core::primitive::u8),
						#[codec(index = 126)]
						Mortal126(::core::primitive::u8),
						#[codec(index = 127)]
						Mortal127(::core::primitive::u8),
						#[codec(index = 128)]
						Mortal128(::core::primitive::u8),
						#[codec(index = 129)]
						Mortal129(::core::primitive::u8),
						#[codec(index = 130)]
						Mortal130(::core::primitive::u8),
						#[codec(index = 131)]
						Mortal131(::core::primitive::u8),
						#[codec(index = 132)]
						Mortal132(::core::primitive::u8),
						#[codec(index = 133)]
						Mortal133(::core::primitive::u8),
						#[codec(index = 134)]
						Mortal134(::core::primitive::u8),
						#[codec(index = 135)]
						Mortal135(::core::primitive::u8),
						#[codec(index = 136)]
						Mortal136(::core::primitive::u8),
						#[codec(index = 137)]
						Mortal137(::core::primitive::u8),
						#[codec(index = 138)]
						Mortal138(::core::primitive::u8),
						#[codec(index = 139)]
						Mortal139(::core::primitive::u8),
						#[codec(index = 140)]
						Mortal140(::core::primitive::u8),
						#[codec(index = 141)]
						Mortal141(::core::primitive::u8),
						#[codec(index = 142)]
						Mortal142(::core::primitive::u8),
						#[codec(index = 143)]
						Mortal143(::core::primitive::u8),
						#[codec(index = 144)]
						Mortal144(::core::primitive::u8),
						#[codec(index = 145)]
						Mortal145(::core::primitive::u8),
						#[codec(index = 146)]
						Mortal146(::core::primitive::u8),
						#[codec(index = 147)]
						Mortal147(::core::primitive::u8),
						#[codec(index = 148)]
						Mortal148(::core::primitive::u8),
						#[codec(index = 149)]
						Mortal149(::core::primitive::u8),
						#[codec(index = 150)]
						Mortal150(::core::primitive::u8),
						#[codec(index = 151)]
						Mortal151(::core::primitive::u8),
						#[codec(index = 152)]
						Mortal152(::core::primitive::u8),
						#[codec(index = 153)]
						Mortal153(::core::primitive::u8),
						#[codec(index = 154)]
						Mortal154(::core::primitive::u8),
						#[codec(index = 155)]
						Mortal155(::core::primitive::u8),
						#[codec(index = 156)]
						Mortal156(::core::primitive::u8),
						#[codec(index = 157)]
						Mortal157(::core::primitive::u8),
						#[codec(index = 158)]
						Mortal158(::core::primitive::u8),
						#[codec(index = 159)]
						Mortal159(::core::primitive::u8),
						#[codec(index = 160)]
						Mortal160(::core::primitive::u8),
						#[codec(index = 161)]
						Mortal161(::core::primitive::u8),
						#[codec(index = 162)]
						Mortal162(::core::primitive::u8),
						#[codec(index = 163)]
						Mortal163(::core::primitive::u8),
						#[codec(index = 164)]
						Mortal164(::core::primitive::u8),
						#[codec(index = 165)]
						Mortal165(::core::primitive::u8),
						#[codec(index = 166)]
						Mortal166(::core::primitive::u8),
						#[codec(index = 167)]
						Mortal167(::core::primitive::u8),
						#[codec(index = 168)]
						Mortal168(::core::primitive::u8),
						#[codec(index = 169)]
						Mortal169(::core::primitive::u8),
						#[codec(index = 170)]
						Mortal170(::core::primitive::u8),
						#[codec(index = 171)]
						Mortal171(::core::primitive::u8),
						#[codec(index = 172)]
						Mortal172(::core::primitive::u8),
						#[codec(index = 173)]
						Mortal173(::core::primitive::u8),
						#[codec(index = 174)]
						Mortal174(::core::primitive::u8),
						#[codec(index = 175)]
						Mortal175(::core::primitive::u8),
						#[codec(index = 176)]
						Mortal176(::core::primitive::u8),
						#[codec(index = 177)]
						Mortal177(::core::primitive::u8),
						#[codec(index = 178)]
						Mortal178(::core::primitive::u8),
						#[codec(index = 179)]
						Mortal179(::core::primitive::u8),
						#[codec(index = 180)]
						Mortal180(::core::primitive::u8),
						#[codec(index = 181)]
						Mortal181(::core::primitive::u8),
						#[codec(index = 182)]
						Mortal182(::core::primitive::u8),
						#[codec(index = 183)]
						Mortal183(::core::primitive::u8),
						#[codec(index = 184)]
						Mortal184(::core::primitive::u8),
						#[codec(index = 185)]
						Mortal185(::core::primitive::u8),
						#[codec(index = 186)]
						Mortal186(::core::primitive::u8),
						#[codec(index = 187)]
						Mortal187(::core::primitive::u8),
						#[codec(index = 188)]
						Mortal188(::core::primitive::u8),
						#[codec(index = 189)]
						Mortal189(::core::primitive::u8),
						#[codec(index = 190)]
						Mortal190(::core::primitive::u8),
						#[codec(index = 191)]
						Mortal191(::core::primitive::u8),
						#[codec(index = 192)]
						Mortal192(::core::primitive::u8),
						#[codec(index = 193)]
						Mortal193(::core::primitive::u8),
						#[codec(index = 194)]
						Mortal194(::core::primitive::u8),
						#[codec(index = 195)]
						Mortal195(::core::primitive::u8),
						#[codec(index = 196)]
						Mortal196(::core::primitive::u8),
						#[codec(index = 197)]
						Mortal197(::core::primitive::u8),
						#[codec(index = 198)]
						Mortal198(::core::primitive::u8),
						#[codec(index = 199)]
						Mortal199(::core::primitive::u8),
						#[codec(index = 200)]
						Mortal200(::core::primitive::u8),
						#[codec(index = 201)]
						Mortal201(::core::primitive::u8),
						#[codec(index = 202)]
						Mortal202(::core::primitive::u8),
						#[codec(index = 203)]
						Mortal203(::core::primitive::u8),
						#[codec(index = 204)]
						Mortal204(::core::primitive::u8),
						#[codec(index = 205)]
						Mortal205(::core::primitive::u8),
						#[codec(index = 206)]
						Mortal206(::core::primitive::u8),
						#[codec(index = 207)]
						Mortal207(::core::primitive::u8),
						#[codec(index = 208)]
						Mortal208(::core::primitive::u8),
						#[codec(index = 209)]
						Mortal209(::core::primitive::u8),
						#[codec(index = 210)]
						Mortal210(::core::primitive::u8),
						#[codec(index = 211)]
						Mortal211(::core::primitive::u8),
						#[codec(index = 212)]
						Mortal212(::core::primitive::u8),
						#[codec(index = 213)]
						Mortal213(::core::primitive::u8),
						#[codec(index = 214)]
						Mortal214(::core::primitive::u8),
						#[codec(index = 215)]
						Mortal215(::core::primitive::u8),
						#[codec(index = 216)]
						Mortal216(::core::primitive::u8),
						#[codec(index = 217)]
						Mortal217(::core::primitive::u8),
						#[codec(index = 218)]
						Mortal218(::core::primitive::u8),
						#[codec(index = 219)]
						Mortal219(::core::primitive::u8),
						#[codec(index = 220)]
						Mortal220(::core::primitive::u8),
						#[codec(index = 221)]
						Mortal221(::core::primitive::u8),
						#[codec(index = 222)]
						Mortal222(::core::primitive::u8),
						#[codec(index = 223)]
						Mortal223(::core::primitive::u8),
						#[codec(index = 224)]
						Mortal224(::core::primitive::u8),
						#[codec(index = 225)]
						Mortal225(::core::primitive::u8),
						#[codec(index = 226)]
						Mortal226(::core::primitive::u8),
						#[codec(index = 227)]
						Mortal227(::core::primitive::u8),
						#[codec(index = 228)]
						Mortal228(::core::primitive::u8),
						#[codec(index = 229)]
						Mortal229(::core::primitive::u8),
						#[codec(index = 230)]
						Mortal230(::core::primitive::u8),
						#[codec(index = 231)]
						Mortal231(::core::primitive::u8),
						#[codec(index = 232)]
						Mortal232(::core::primitive::u8),
						#[codec(index = 233)]
						Mortal233(::core::primitive::u8),
						#[codec(index = 234)]
						Mortal234(::core::primitive::u8),
						#[codec(index = 235)]
						Mortal235(::core::primitive::u8),
						#[codec(index = 236)]
						Mortal236(::core::primitive::u8),
						#[codec(index = 237)]
						Mortal237(::core::primitive::u8),
						#[codec(index = 238)]
						Mortal238(::core::primitive::u8),
						#[codec(index = 239)]
						Mortal239(::core::primitive::u8),
						#[codec(index = 240)]
						Mortal240(::core::primitive::u8),
						#[codec(index = 241)]
						Mortal241(::core::primitive::u8),
						#[codec(index = 242)]
						Mortal242(::core::primitive::u8),
						#[codec(index = 243)]
						Mortal243(::core::primitive::u8),
						#[codec(index = 244)]
						Mortal244(::core::primitive::u8),
						#[codec(index = 245)]
						Mortal245(::core::primitive::u8),
						#[codec(index = 246)]
						Mortal246(::core::primitive::u8),
						#[codec(index = 247)]
						Mortal247(::core::primitive::u8),
						#[codec(index = 248)]
						Mortal248(::core::primitive::u8),
						#[codec(index = 249)]
						Mortal249(::core::primitive::u8),
						#[codec(index = 250)]
						Mortal250(::core::primitive::u8),
						#[codec(index = 251)]
						Mortal251(::core::primitive::u8),
						#[codec(index = 252)]
						Mortal252(::core::primitive::u8),
						#[codec(index = 253)]
						Mortal253(::core::primitive::u8),
						#[codec(index = 254)]
						Mortal254(::core::primitive::u8),
						#[codec(index = 255)]
						Mortal255(::core::primitive::u8),
					}
				}
				pub mod unchecked_extrinsic {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: codec :: Decode,
						:: subxt :: ext :: codec :: Encode,
						:: subxt :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
					#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
					pub struct UncheckedExtrinsic<_0, _1, _2, _3>(
						pub ::std::vec::Vec<::core::primitive::u8>,
						#[codec(skip)] pub ::core::marker::PhantomData<(_0, _1, _2, _3)>,
					);
				}
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub enum DispatchError {
				#[codec(index = 0)]
				Other,
				#[codec(index = 1)]
				CannotLookup,
				#[codec(index = 2)]
				BadOrigin,
				#[codec(index = 3)]
				Module(runtime_types::sp_runtime::ModuleError),
				#[codec(index = 4)]
				ConsumerRemaining,
				#[codec(index = 5)]
				NoProviders,
				#[codec(index = 6)]
				TooManyConsumers,
				#[codec(index = 7)]
				Token(runtime_types::sp_runtime::TokenError),
				#[codec(index = 8)]
				Arithmetic(runtime_types::sp_arithmetic::ArithmeticError),
				#[codec(index = 9)]
				Transactional(runtime_types::sp_runtime::TransactionalError),
				#[codec(index = 10)]
				Exhausted,
				#[codec(index = 11)]
				Corruption,
				#[codec(index = 12)]
				Unavailable,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub struct ModuleError {
				pub index: ::core::primitive::u8,
				pub error: [::core::primitive::u8; 4usize],
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub enum MultiSignature {
				#[codec(index = 0)]
				Ed25519(runtime_types::sp_core::ed25519::Signature),
				#[codec(index = 1)]
				Sr25519(runtime_types::sp_core::sr25519::Signature),
				#[codec(index = 2)]
				Ecdsa(runtime_types::sp_core::ecdsa::Signature),
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub enum TokenError {
				#[codec(index = 0)]
				NoFunds,
				#[codec(index = 1)]
				WouldDie,
				#[codec(index = 2)]
				BelowMinimum,
				#[codec(index = 3)]
				CannotCreate,
				#[codec(index = 4)]
				UnknownAsset,
				#[codec(index = 5)]
				Frozen,
				#[codec(index = 6)]
				Unsupported,
			}
			#[derive(
				:: subxt :: ext :: codec :: Decode,
				:: subxt :: ext :: codec :: Encode,
				:: subxt :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: scale_encode")]
			pub enum TransactionalError {
				#[codec(index = 0)]
				LimitReached,
				#[codec(index = 1)]
				NoLayer,
			}
		}
	}
}
