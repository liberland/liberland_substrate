#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[cfg(test)]
mod mock;

#[ink::contract(env = liberland_extension::LiberlandEnvironment)]
mod msig_court {
	use ink::codegen::Env;
	use ink::prelude::vec::Vec;
	use ink::storage::Mapping;
	use liberland_extension::LLMForceTransferArguments;

	#[derive(Debug, Clone, PartialEq, Eq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	pub enum Proposal {
		LLMForceTransfer(LLMForceTransferArguments),
		SetGovernance { threshold: u32, judges: Vec<AccountId> },
	}

	#[derive(Debug, PartialEq, Eq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum ProposalState {
		/// Waiting for more judge approvals.
		PendingApprovals,
		/// Enough approvals collected, waiting for veto period to pass.
		PendingVetoPeriod,
		/// Proposal was vetoed by veto authority.
		Vetoed,
		/// Proposal was executed on-chain.
		Executed(Result<()>),
	}

	#[derive(Debug, PartialEq, Eq, Clone)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Error {
		/// Unauthorized
		Unauthorized,
		/// Proposal already exists
		AlreadyExists,
		/// Proposal not found
		NotFound,
		/// Caller already approved for this proposal
		AlreadyApproved,
		/// Call failed
		CallFailed,
		/// Invalid parameters
		InvalidParameters,
		/// Proposal is still in veto period
		StillInVetoPeriod,
		/// Proposal was already vetoed
		AlreadyVetoed,
		/// Caller is not the veto authority
		NotVetoAuthority,
	}

	impl From<liberland_extension::Error> for Error {
		fn from(_: liberland_extension::Error) -> Self {
			Self::CallFailed
		}
	}

	pub type Result<T> = core::result::Result<T, Error>;
	pub type PropKey = <ink::env::hash::Blake2x256 as ink::env::hash::HashOutput>::Type;

	/// Default veto period used when the contract is instantiated (in blocks).
	/// Assuming ~6 second blocks, 14 days ≈ 201_600 blocks.
	const DEFAULT_VETO_PERIOD: BlockNumber = 201_600;

	#[ink(storage)]
	pub struct MsigCourt {
		threshold: u32,
		judges: Vec<AccountId>,
		veto_authority: AccountId,
		proposals: Mapping<PropKey, Proposal>,
		approvals: Mapping<PropKey, Vec<AccountId>>,
		pending_executions: Mapping<PropKey, (Proposal, BlockNumber)>,
		vetoed: Mapping<PropKey, bool>,
		veto_period: BlockNumber,
	}

	#[ink(event)]
	pub struct Proposed {
		#[ink(topic)]
		proposer: AccountId,
		key: PropKey,
		proposal: Proposal,
	}

	#[ink(event)]
	pub struct Approved {
		#[ink(topic)]
		approver: AccountId,
		key: PropKey,
	}

	#[ink(event)]
	pub struct Executed {
		#[ink(topic)]
		approver: AccountId,
		key: PropKey,
		result: Result<()>,
	}

	/// A proposal reached the required threshold and entered the veto period.
	#[ink(event)]
	pub struct PendingExecution {
		#[ink(topic)]
		approver: AccountId,
		#[ink(topic)]
		key: PropKey,
		execute_after: BlockNumber,
	}

	/// A proposal was vetoed by the veto authority.
	#[ink(event)]
	pub struct Vetoed {
		#[ink(topic)]
		vetoer: AccountId,
		#[ink(topic)]
		key: PropKey,
	}

	impl MsigCourt {
		fn execute(&mut self, proposal: Proposal) -> Result<()> {
			use Proposal::*;
			match proposal {
				LLMForceTransfer(args) => {
					self.env().extension().llm_force_transfer(args).map_err(|e| e.into())
				},
				SetGovernance { threshold, judges } => self.set_governance(threshold, judges),
			}
		}

		fn do_approve(&mut self, approver: AccountId, key: PropKey) -> Result<ProposalState> {
			let approvals = self.approvals.take(key).ok_or(Error::NotFound)?;
			if approvals.contains(&approver) {
				return Err(Error::AlreadyApproved);
			}

			if approvals.len().saturating_add(1) >= self.threshold as usize {
				let proposal =
					self.proposals.take(key).expect("Approvals exist, so proposal must exist too");

				let now = self.env().block_number();
				let execute_after = now.saturating_add(self.veto_period);
				self.pending_executions.insert(key, &(proposal, execute_after));

				self.env().emit_event(PendingExecution { approver, key, execute_after });
				Ok(ProposalState::PendingVetoPeriod)
			} else {
				let mut approvals = approvals;
				approvals.push(approver);
				self.approvals.insert(key, &approvals);
				self.env().emit_event(Approved { approver, key });
				Ok(ProposalState::PendingApprovals)
			}
		}

		fn set_governance(&mut self, threshold: u32, judges: Vec<AccountId>) -> Result<()> {
			if threshold as usize > judges.len() {
				return Err(Error::InvalidParameters);
			}

			self.threshold = threshold;
			self.judges = judges;
			Ok(())
		}
	}

	impl Default for MsigCourt {
		fn default() -> Self {
			Self {
				threshold: 0,
				judges: Vec::new(),
				veto_authority: AccountId::from([0u8; 32]),
				proposals: Mapping::new(),
				approvals: Mapping::new(),
				pending_executions: Mapping::new(),
				vetoed: Mapping::new(),
				veto_period: DEFAULT_VETO_PERIOD,
			}
		}
	}

	impl MsigCourt {
		#[ink(constructor)]
		pub fn new(threshold: u32, judges: Vec<AccountId>, veto_authority: AccountId) -> Self {
			assert!(threshold as usize <= judges.len());
			Self {
				threshold,
				judges,
				veto_authority,
				veto_period: DEFAULT_VETO_PERIOD,
				..Default::default()
			}
		}

		#[ink(message)]
		pub fn propose(&mut self, proposal: Proposal) -> Result<(PropKey, ProposalState)> {
			let caller = self.env().caller();
			if !self.judges.contains(&caller) {
				return Err(Error::Unauthorized);
			}

			let mut key =
				<ink::env::hash::Blake2x256 as ink::env::hash::HashOutput>::Type::default();
			ink::env::hash_encoded::<ink::env::hash::Blake2x256, _>(&proposal, &mut key);

			if self.proposals.contains(key) {
				return Err(Error::AlreadyExists);
			}

			self.proposals.insert(key, &proposal);
			self.approvals.insert(key, &Vec::<AccountId>::new());
			self.env().emit_event(Proposed { proposer: caller, key, proposal });
			let state = self.do_approve(caller, key)?;
			Ok((key, state))
		}

		#[ink(message)]
		pub fn approve(&mut self, key: PropKey) -> Result<ProposalState> {
			let caller = self.env().caller();
			if !self.judges.contains(&caller) {
				return Err(Error::Unauthorized);
			}
			self.do_approve(caller, key)
		}

		#[ink(message)]
		pub fn get_threshold(&self) -> u32 {
			self.threshold
		}

		#[ink(message)]
		pub fn get_judges(&self) -> Vec<AccountId> {
			self.judges.clone()
		}

		#[ink(message)]
		pub fn get_proposal(&self, key: PropKey) -> Option<(Proposal, Vec<AccountId>)> {
			Some((self.proposals.get(key)?, self.approvals.get(key)?))
		}

		#[ink(message)]
		pub fn get_veto_authority(&self) -> AccountId {
			self.veto_authority
		}

		/// Veto a pending proposal. Can only be called by the veto authority account.
		#[ink(message)]
		pub fn veto(&mut self, key: PropKey) -> Result<()> {
			let caller = self.env().caller();
			if caller != self.veto_authority {
				return Err(Error::NotVetoAuthority)
			}

			if self.pending_executions.get(key).is_none() {
				return Err(Error::NotFound)
			}

			self.pending_executions.remove(key);
			self.vetoed.insert(key, &true);
			self.env().emit_event(Vetoed { vetoer: caller, key });
			Ok(())
		}

		/// Execute a proposal after the veto period has passed.
		///
		/// Anyone can trigger this; the authority is encoded in the proposal itself.
		#[ink(message)]
		pub fn execute_pending(&mut self, key: PropKey) -> Result<ProposalState> {
			if self.vetoed.get(key).unwrap_or(false) {
				return Err(Error::AlreadyVetoed)
			}

			let (proposal, execute_after) =
				self.pending_executions.get(key).ok_or(Error::NotFound)?;

			let now = self.env().block_number();
			if now < execute_after {
				return Err(Error::StillInVetoPeriod)
			}

			self.pending_executions.remove(key);
			let caller = self.env().caller();
			let result = self.execute(proposal);
			self.env().emit_event(Executed { approver: caller, key, result: result.clone() });
			Ok(ProposalState::Executed(result))
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;
		use crate::mock::*;
		use liberland_extension::LLMAccount;

		fn alice() -> AccountId {
			ink::env::test::default_accounts::<Environment>().alice
		}

		fn bob() -> AccountId {
			ink::env::test::default_accounts::<Environment>().bob
		}

		fn charlie() -> AccountId {
			ink::env::test::default_accounts::<Environment>().charlie
		}

		fn django() -> AccountId {
			ink::env::test::default_accounts::<Environment>().django
		}

		fn set_next_caller(caller: AccountId) {
			ink::env::test::set_caller::<Environment>(caller);
		}

		fn advance_block(blocks: BlockNumber) {
			for _ in 0..blocks {
				ink::env::test::advance_block::<Environment>();
			}
		}

		fn assert_proposed_event(
			event: &ink::env::test::EmittedEvent,
			expected_proposer: AccountId,
			expected_key: PropKey,
			expected_proposal: Proposal,
		) {
			let decoded_event = <Proposed as ink::scale::Decode>::decode(&mut &event.data[..])
				.expect("encountered invalid contract event data buffer");
			let Proposed { proposer, key, proposal } = decoded_event;
			assert_eq!(proposer, expected_proposer);
			assert_eq!(key, expected_key);
			assert_eq!(proposal, expected_proposal);
		}

		fn assert_approved_event(
			event: &ink::env::test::EmittedEvent,
			expected_approver: AccountId,
			expected_key: PropKey,
		) {
			let decoded_event = <Approved as ink::scale::Decode>::decode(&mut &event.data[..])
				.expect("encountered invalid contract event data buffer");
			let Approved { approver, key } = decoded_event;
			assert_eq!(approver, expected_approver);
			assert_eq!(key, expected_key);
		}

		fn assert_executed_event(
			event: &ink::env::test::EmittedEvent,
			expected_approver: AccountId,
			expected_key: PropKey,
			expected_result: Result<()>,
		) {
			let decoded_event = <Executed as ink::scale::Decode>::decode(&mut &event.data[..])
				.expect("encountered invalid contract event data buffer");
			let Executed { approver, key, result } = decoded_event;
			assert_eq!(approver, expected_approver);
			assert_eq!(key, expected_key);
			assert_eq!(result, expected_result);
		}

		fn assert_pending_execution_event(
			event: &ink::env::test::EmittedEvent,
			expected_approver: AccountId,
			expected_key: PropKey,
		) {
			let decoded_event = <PendingExecution as ink::scale::Decode>::decode(&mut &event.data[..])
				.expect("encountered invalid contract event data buffer");
			let PendingExecution { approver, key, execute_after: _ } = decoded_event;
			assert_eq!(approver, expected_approver);
			assert_eq!(key, expected_key);
		}

		#[ink::test]
		fn new_works() {
			let msig_court = MsigCourt::new(1, vec![alice()], django());
			assert_eq!(msig_court.threshold, 1);
			assert_eq!(msig_court.judges[0], alice());
			assert_eq!(msig_court.judges.len(), 1);

			let msig_court = MsigCourt::new(2, vec![alice(), bob(), charlie()], django());
			assert_eq!(msig_court.threshold, 2);
			assert_eq!(msig_court.judges[0], alice());
			assert_eq!(msig_court.judges[1], bob());
			assert_eq!(msig_court.judges[2], charlie());
			assert_eq!(msig_court.judges.len(), 3);
		}

		#[ink::test]
		#[should_panic]
		fn new_prevents_bricking() {
			MsigCourt::new(2, vec![alice()], django());
		}

		#[ink::test]
		fn propose_executes_immediately_with_threshold_1() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, state) = msig_court
				.propose(Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] })
				.expect("propose shouldnt fail");

			assert_eq!(state, ProposalState::PendingVetoPeriod);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			let result = msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			assert_eq!(result, ProposalState::Executed(Ok(())));

			assert_eq!(msig_court.threshold, 2);
			assert_eq!(msig_court.judges.len(), 2);
		}

		#[ink::test]
		fn must_be_a_judge_to_propose() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(bob());
			let res = msig_court
				.propose(Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] });
			assert_eq!(res, Err(Error::Unauthorized));
		}

		#[ink::test]
		fn propose_doesnt_execute_with_threshold_2() {
			let mut msig_court = MsigCourt::new(2, vec![alice(), bob()], django());
			set_next_caller(alice());
			let proposal = Proposal::SetGovernance { threshold: 1, judges: vec![alice()] };
			let (key, state) = msig_court.propose(proposal.clone()).expect("propose shouldnt fail");
			assert_eq!(state, ProposalState::PendingApprovals);
			assert_eq!(msig_court.proposals.get(&key), Some(proposal));
			assert_eq!(msig_court.approvals.get(&key), Some(vec![alice()]));
		}

		#[ink::test]
		fn cant_duplicate_proposals() {
			let mut msig_court = MsigCourt::new(2, vec![alice(), bob()], django());
			set_next_caller(alice());
			let proposal = Proposal::SetGovernance { threshold: 1, judges: vec![alice()] };
			let (_, state) = msig_court.propose(proposal.clone()).expect("propose shouldnt fail");
			assert_eq!(state, ProposalState::PendingApprovals);

			let res = msig_court.propose(proposal.clone());
			assert_eq!(res, Err(Error::AlreadyExists));
		}

		#[ink::test]
		fn approve_works() {
			let mut msig_court = MsigCourt::new(3, vec![alice(), bob(), charlie()], django());
			set_next_caller(alice());
			let (key, _) = msig_court
				.propose(Proposal::SetGovernance { threshold: 1, judges: vec![alice()] })
				.expect("propose shouldnt fail");

			set_next_caller(bob());
			let res = msig_court.approve(key);
			assert_eq!(res, Ok(ProposalState::PendingApprovals));
			assert_eq!(msig_court.approvals.get(&key), Some(vec![alice(), bob()]))
		}

		#[ink::test]
		fn cant_double_approve() {
			let mut msig_court = MsigCourt::new(3, vec![alice(), bob(), charlie()], django());
			set_next_caller(alice());
			let (key, _) = msig_court
				.propose(Proposal::SetGovernance { threshold: 1, judges: vec![alice()] })
				.expect("propose shouldnt fail");

			let res = msig_court.approve(key);
			assert_eq!(res, Err(Error::AlreadyApproved));
		}

		#[ink::test]
		fn must_be_a_judge_to_approve() {
			let mut msig_court = MsigCourt::new(2, vec![alice(), bob()], django());
			set_next_caller(alice());
			let (key, _) = msig_court
				.propose(Proposal::SetGovernance { threshold: 1, judges: vec![alice()] })
				.expect("propose shouldnt fail");

			set_next_caller(charlie());
			let res = msig_court.approve(key);
			assert_eq!(res, Err(Error::Unauthorized));
		}

		#[ink::test]
		fn set_governance_works() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, state) = msig_court
				.propose(Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] })
				.expect("propose shouldnt fail");
			assert_eq!(state, ProposalState::PendingVetoPeriod);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			let result = msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			assert_eq!(result, ProposalState::Executed(Ok(())));

			assert_eq!(msig_court.threshold, 2);
			assert_eq!(msig_court.judges[0], alice());
			assert_eq!(msig_court.judges[1], bob());
			assert_eq!(msig_court.judges.len(), 2);
		}

		#[ink::test]
		fn set_governance_prevents_bricking() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, state) = msig_court
				.propose(Proposal::SetGovernance { threshold: 3, judges: vec![alice(), bob()] })
				.expect("propose shouldnt fail");
			assert_eq!(state, ProposalState::PendingVetoPeriod);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			let result = msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			assert_eq!(result, ProposalState::Executed(Err(Error::InvalidParameters)));
			assert_eq!(msig_court.threshold, 1);
			assert_eq!(msig_court.judges[0], alice());
			assert_eq!(msig_court.judges.len(), 1);
		}

		#[ink::test]
		fn llm_force_transfer_works() {
			ink::env::test::register_chain_extension(MockedLiberlandExtensionSuccess);

			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, state) = msig_court
				.propose(Proposal::LLMForceTransfer(LLMForceTransferArguments {
					from: LLMAccount::Locked(alice()),
					to: LLMAccount::Locked(bob()),
					amount: 1u8.into(),
				}))
				.expect("propose shouldnt fail");
			assert_eq!(state, ProposalState::PendingVetoPeriod);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			let result = msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			assert_eq!(result, ProposalState::Executed(Ok(())));
		}

		#[ink::test]
		fn llm_force_transfer_propagates_errors() {
			ink::env::test::register_chain_extension(MockedLiberlandExtensionFail);

			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, state) = msig_court
				.propose(Proposal::LLMForceTransfer(LLMForceTransferArguments {
					from: LLMAccount::Locked(alice()),
					to: LLMAccount::Locked(bob()),
					amount: 1u8.into(),
				}))
				.expect("propose shouldnt fail");
			assert_eq!(state, ProposalState::PendingVetoPeriod);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			let result = msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			assert_eq!(result, ProposalState::Executed(Err(Error::CallFailed)));
		}

		#[ink::test]
		fn correct_events_for_threshold_1() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			let proposal = Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] };
			set_next_caller(alice());
			let (key, _) = msig_court.propose(proposal.clone()).expect("propose shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 2);
			assert_proposed_event(&emitted_events[0], alice(), key, proposal);
			assert_pending_execution_event(&emitted_events[1], alice(), key);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 3);
			assert_executed_event(&emitted_events[2], alice(), key, Ok(()));
		}

		#[ink::test]
		fn correct_events_for_threshold_2() {
			let mut msig_court = MsigCourt::new(2, vec![alice(), bob()], django());
			let proposal =
				Proposal::SetGovernance { threshold: 3, judges: vec![alice(), bob(), charlie()] };

			set_next_caller(alice());
			let (key, _) = msig_court.propose(proposal.clone()).expect("propose shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 2);
			assert_proposed_event(&emitted_events[0], alice(), key, proposal);
			assert_approved_event(&emitted_events[1], alice(), key);

			set_next_caller(bob());
			msig_court.approve(key).expect("approve shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 3);
			assert_pending_execution_event(&emitted_events[2], bob(), key);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 4);
			assert_executed_event(&emitted_events[3], bob(), key, Ok(()));
		}
		#[ink::test]
		fn correct_events_for_threshold_3() {
			let mut msig_court = MsigCourt::new(3, vec![alice(), bob(), charlie()], django());
			let proposal = Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] };

			set_next_caller(alice());
			let (key, _) = msig_court.propose(proposal.clone()).expect("propose shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 2);
			assert_proposed_event(&emitted_events[0], alice(), key, proposal);
			assert_approved_event(&emitted_events[1], alice(), key);

			set_next_caller(bob());
			msig_court.approve(key).expect("approve shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 3);
			assert_approved_event(&emitted_events[2], bob(), key);

			set_next_caller(charlie());
			msig_court.approve(key).expect("approve shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 4);
			assert_pending_execution_event(&emitted_events[3], charlie(), key);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 5);
			assert_executed_event(&emitted_events[4], charlie(), key, Ok(()));
		}

		#[ink::test]
		fn correct_events_for_failed_call() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			let proposal = Proposal::SetGovernance { threshold: 3, judges: vec![alice(), bob()] };
			set_next_caller(alice());
			let (key, _) = msig_court.propose(proposal.clone()).expect("propose shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 2);
			assert_proposed_event(&emitted_events[0], alice(), key, proposal);
			assert_pending_execution_event(&emitted_events[1], alice(), key);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
			assert_eq!(emitted_events.len(), 3);
			assert_executed_event(&emitted_events[2], alice(), key, Err(Error::InvalidParameters));
		}

		#[ink::test]
		fn get_threshold_works() {
			let msig_court = MsigCourt::new(1, vec![alice()], django());
			assert_eq!(msig_court.get_threshold(), 1);
		}

		#[ink::test]
		fn get_judges_works() {
			let msig_court = MsigCourt::new(1, vec![alice()], django());
			assert_eq!(msig_court.get_judges(), vec![alice()]);
		}

		#[ink::test]
		fn get_proposal_works() {
			let mut msig_court = MsigCourt::new(3, vec![alice(), bob(), charlie()], django());
			let proposal = Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] };

			set_next_caller(alice());
			let (key, _) = msig_court.propose(proposal.clone()).expect("propose shouldnt fail");
			assert_eq!(msig_court.get_proposal(key), Some((proposal.clone(), vec![alice()])));

			set_next_caller(bob());
			msig_court.approve(key).expect("approve shouldnt fail");
			assert_eq!(msig_court.get_proposal(key), Some((proposal.clone(), vec![alice(), bob()])));

			set_next_caller(charlie());
			msig_court.approve(key).expect("approve shouldnt fail");
			assert_eq!(msig_court.get_proposal(key), None);

			advance_block(DEFAULT_VETO_PERIOD + 1);
			msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			assert_eq!(msig_court.get_proposal(key), None);
		}

		#[ink::test]
		fn get_proposal_fails_on_not_found() {
			let msig_court = MsigCourt::new(1, vec![alice()], django());
			let key = <ink::env::hash::Blake2x256 as ink::env::hash::HashOutput>::Type::default();
			assert_eq!(msig_court.get_proposal(key), None);
		}

		#[ink::test]
		fn get_veto_authority_works() {
			let veto_authority = django();
			let msig_court = MsigCourt::new(1, vec![alice()], veto_authority);
			assert_eq!(msig_court.get_veto_authority(), veto_authority);
		}

		#[ink::test]
		fn execute_pending_before_veto_period_fails() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, _) = msig_court
				.propose(Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] })
				.expect("propose shouldnt fail");

			let result = msig_court.execute_pending(key);
			assert_eq!(result, Err(Error::StillInVetoPeriod));
		}

		#[ink::test]
		fn execute_pending_after_veto_period_works() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, _) = msig_court
				.propose(Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] })
				.expect("propose shouldnt fail");

			advance_block(DEFAULT_VETO_PERIOD + 1);
			let result = msig_court.execute_pending(key).expect("execute_pending shouldnt fail");
			assert_eq!(result, ProposalState::Executed(Ok(())));
		}

		#[ink::test]
		fn veto_works() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, _) = msig_court
				.propose(Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] })
				.expect("propose shouldnt fail");

			set_next_caller(django());
			msig_court.veto(key).expect("veto shouldnt fail");

			advance_block(DEFAULT_VETO_PERIOD + 1);
			let result = msig_court.execute_pending(key);
			assert_eq!(result, Err(Error::AlreadyVetoed));
		}

		#[ink::test]
		fn veto_only_by_veto_authority() {
			let mut msig_court = MsigCourt::new(1, vec![alice()], django());
			set_next_caller(alice());
			let (key, _) = msig_court
				.propose(Proposal::SetGovernance { threshold: 2, judges: vec![alice(), bob()] })
				.expect("propose shouldnt fail");

			set_next_caller(bob());
			let result = msig_court.veto(key);
			assert_eq!(result, Err(Error::NotVetoAuthority));
		}
	}

	#[cfg(all(test, feature = "e2e-tests"))]
	mod e2e_tests {
		use super::*;
		use ink_e2e::ContractsBackend;

		type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

		#[ink_e2e::test]
		async fn new_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
			let mut constructor = MsigCourtRef::new(1, vec![ink_e2e::alice()], ink_e2e::django());

			let contract = client
				.instantiate("msig_court", &ink_e2e::alice(), &mut constructor)
				.submit()
				.await
				.expect("instantiate failed");

			Ok(())
		}
	}
}
