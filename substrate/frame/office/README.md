 # Liberland Office Pallet

 ## Overview

 Office pallet is a general-purpose pallet for executing external calls using
 a single AccountId by a centrally managed set of Accounts.

 ## Terminology

 * PalletId - Id that will be used to derive AccountId for dispatching calls
 * Admin - AccountId that can add/update/remove clerks
 * Clerk - AccountId that's authorized to execute calls using this pallet

 ## Pallet Config

 * `PalletId` - PalletId that's used to derive AccountId for dispatching external calls
 * `AdminOrigin` - origin for checking admin - must return AccountId on success
 * `ForceOrigin` - origin that can add/remove clerks and update admin without the admin AccountId check
 * `CallFilter` - InstanceFilter for filtering calls by clerk - see mock.rs for example
 * `WeightInfo` - see [Substrate docs](https://docs.substrate.io/reference/how-to-guides/weights/use-custom-weights/)

 ## Genesis Config

 * `admin`: Initial admin
 * `clerks`: Initial clerks

 ## Interface

 ### Dispatchable Functions

 * `set_admin`: Change admin
 * `set_clerk`: Add or update clerk
 * `remove_clerk`: Remove clerk
 * `execute`: Execute an external call using this pallet as origin

 License: MIT/