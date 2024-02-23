 # Liberland Custom Account Pallet

 ## Overview

 Custom Account pallet is a general-purpose pallet for executing arbitrary calls
 using preconfigured AccountId as Origin. It allows filtering who can execute
 calls (via configurable EnsureOrigin) and filtering what calls can be made.
 ## Pallet Config

 * `PalletId` - PalletId that's used to derive AccountId for dispatching calls
 * `ExecuteOrigin` - origin that can execute calls
 * `CallFilter` - Contains for filtering calls by clerk - see mock.rs for example
 * `WeightInfo` - see [Substrate docs](https://docs.substrate.io/reference/how-to-guides/weights/use-custom-weights/)


 ## Interface

 ### Dispatchable Functions

 * `execute`: Execute an external call

 License: MIT.