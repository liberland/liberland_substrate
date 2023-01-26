# Liberland Merit(LLM) Pallet

## Overview

Liberland Merit is a Liberland currency that gives political power to citizens.

The LLM pallet handles:

* creating LLM asset in `pallet-assets` on genesis
* LLM release from **Vault** to **Treasury**
* locking, a.k.a. politipooling the LLM for use in politics
* veryfing citizenship status

## LLM lifecycle

On Genesis (see `fn create_llm`):

* LLM is created in `pallet-assets`
* configured `TotalSupply` amount of LLM is created and transferred to **Vault**
* configured `PreReleasedAmount` is transferred from **Vault** to **Treasury**

On yearly basis (see `fn try_release`):

* 90% of **Vault** balance is transferred to **Treasury**

Accounts are free to locks in politics, a.k.a. politipool any amount of LLM at any time.

Accounts may unlock 10% of locked LLM once every `Withdrawlock` duration (see [Genesis Config](#genesis-config)), but it will suspend their politics rights for `Electionlock` duration.

Accounts may freely transfer their not-locked LLM to other accounts.

### Special accounts:

* **Treasury**:
    * gets `PreReleasedAmount` LLM on genesis and 10% of **Vault** balance periodically (_LLM Release Event_)
    * derived from PalletID `py/trsry`: `5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z`

* **Vault**:
    * gets initial supply of LLM on genesis
    * releases 10% of it's balance to **Trasury** on LLM Release Event (yearly)
    * derived from PalletID `llm/safe`: `5EYCAe5hvejUE1BUTDSnxDfCqVkADRicSKqbcJrduV1KCDmk`

* **Politipool**,
    * gets LLM locked in politics by other accounts (`politics_lock`)
    * releases locked LLM back on `politics_unlock`
    * derived from PalletID `politilock`: `5EYCAe5ijGqt3WEM9aKUBdth51NEBNz9P84NaUMWZazzWt7c`

## Internal Storage:

* `NextRelease`: block number for next LLM Release Event (transfer of 10% from **Vault** to **Treasury**)
* `LLMPolitics`: amount of LLM each account has allocated into politics
* `Withdrawlock`: block number until which account can't do another `politics_unlock`
* `Electionlock`: block number until which account can't participate in politics directly

## Runtime config

* `RuntimeEvent`: Event type to use.
* `AssetId`: Type of AssetId.
* `TotalSupply`: Total amount of LLM to be created on genesis. That's all LLM that will ever exit. It will be stored in **Vault**.
* `PreReleasedAmount`: Amount of LLM that should be released (a.k.a. transferred from **Vault** to **Treasury**) on genesis.

## Genesis Config

* `unpooling_withdrawlock_duration`: duration, in seconds, for which additional unlocks should be locked after `politics_unlock`
* `unpooling_electionlock_duration`: duration, in seconds, for which politics rights should be suspended after `politics_unlock`

## Interface

### Dispatchable Functions

#### Public

These calls can be made from any _Signed_ origin.

* `fake_send`: Release LLM from **Vault** to specific account. Development only, to be removed/restricted.
* `send_llm`: Transfer LLM. Wrapper over `pallet-assets`' `transfer`.
* `politics_lock`: Lock LLM into politics pool, a.k.a. politipool.
* `politics_unlock`: Unlock 10% of locked LLM. Can't be called again for a WithdrawalLock period. Affects political rights for an ElectionLock period.
* `approve_transfer`: As an assembly member you can approve a transfer of LLM. Not implemented.

#### Restricted

* `treasury_llm_transfer`: Transfer LLM from treasury to specified account. Can only be called by selected accounts and Senate.

### Public functions

* `llm_id`: Asset ID of the LLM asset for `pallet-assets`
* `get_llm_vault_account`: AccountId of **Vault** account. **Vault** account stores all LLM created initially on genesis and releases it to treasury on LLM Release Events.
* `get_llm_treasury_account`: AccountId of **Treasury** account. **Treasury** accounts receives prereleased amount of LLM on genesis and part of LLM from **Vault** on LLM Release Events.
* `get_llm_politipool_account`: AccountId of **Politipool** account. **Politipool** account stores LLM locked in politics by all other accounts.

### LLM trait

LLM pallet implements LLM trait with following functions available for other pallets:

* `check_pooled_llm`: Checks if given account has any LLM locked in politics.
* `is_election_unlocked`: Checks if given account has rights to participate in politics unlocked. They may be locked after `politics_unlock`. This does NOT check if account is a valid citizen - use `CitizenshipChecker` trait for that.
* `get_politi_pooled_amount`: Get total amount of locked LLM across all accounts.
* `get_llm_politics`: Get amount of locked LLM for given account.

### CitizenshipChecker trait

LLM pallet implements CitizenshipChecker trait with following functions available for other pallets:

* `ensure_politics_allowed`: Checks if given account can participate in politics actions. It verifies that it's a valid citizen, doesn't have election rights locked and has 5000 LLM locked in politics.


### Approved Multisig llm transfers



![Polkadot Js Treasury](treasury_account_query.png)

![Polkadot Js Treasury](check_multisig.png)

![Polkadot Js Treasury](treasuryllm_transfer_with_multisig.png)

![Polkadot Js Treasury](multisig_send_tx.png)

![Polkadot Js Treasury](pending_multisig.png)

![Polkadot Js Treasury](approve_multisig.png)

![Polkadot Js Treasury](pasted_multisig_approved_data.png)

![Polkadot Js Treasury](after_multisig.png)


License: MIT
