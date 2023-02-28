 # Liberland Registry Pallet

 ## Overview

 Registry pallet is a general-purpose pallet for tracking and registering
 data about abstract entities.

 In Liberland, it's used to implement Company Registry.

 ## Terminology

 * Entity - object, identified with AccountId, that has data attached to it and can be registered
   at a registrar
 * Registrar - AccountId that can register Entities in Registry.
 * Registry - database of Entities and their data that were registered.
 * Deposit - amount of Currency that gets reserved when requesting registration - refunded data
   is removed

 ## Entity Lifecycle

 1. Entity requests registration at Registry using `request_registration()` call. This creates
    and stores a Registration Request and reserves the deposit.
 2. If Registrar approves Entity's data, they call `register_entity()` which
    moves data from Registration Request to the Registry.
 3. To update the data, Entity needs to repeat the same process as for initial registration.
 3. Entity can be removed from Registry by the Registrar - deposit will be refunded.

 ## Deposits

 Registry pallet requires deposits to cover the cost of storing data in
 current blockchain state (which isn't pruned). These deposits always matches
 the amount of stored data:
 * if there's no prior deposit, total required deposit will be reserved
 * if there's a deposit, but now we want to store more data, the additional difference will be
   reserved
 * if there's a deposit, but now we want to store less or no data, the excess will be refunded

 Formula for required deposit to store `N` bytes of data:
 ```ignore
 deposit = BaseDeposit + N * ByteDeposit
 ```

 Deposits are separate for each registry (as data is stored separately as
 well).

 * `request_registration(registry, data, editable)` requires deposit for length of `data`
   parameter. Will immediately reserve/refund any difference.
 * `cancel_request()` will refund complete deposit for given request.
 * `unregister()` will refund deposit for data at given registrar.
 * `register_entity()` will refund old deposit, if any.

 ## Pallet Config

 * `Currency` - pallet implementing NamedReservableCurrency - it will be used for reserving
   deposits
 * `ReserveIdentified` - will be used in Currency to attach reserves to this pallet
 * `MaxRegistrars` - max number of registrars that can be added
 * `BaseDeposit` - see **Deposits** above
 * `ByteDeposit` - see **Deposits** above
 * `AddRegistrarOrigin` - origin that's authorized to add new registrars
 * `RegistrarOrigin` - origin of registrars - must return AccountId on success
 * `EntityOrigin` - origin of entities - must return AccountId on usccess
 * `EntityData` - type that will be used to store and process Entities data
 * `WeightInfo` - see [Substrate docs](https://docs.substrate.io/reference/how-to-guides/weights/use-custom-weights/)

 ## Genesis Config

 * `registries`: registries that should be preset on genesis

 ## Interface

 ### Dispatchable Functions

 #### Public

 These calls can be made from any _Signed_ origin.

 * `add_registrar`: Adds a new registrar
 * `request_registration`: Requests a registration of Entity in Registry
 * `cancel_request`: Cancels registration request
 * `unregister`: Removes Entity from given Registry
 * `register_entity`: Adds Entity to the Registry
 * `set_registered_entity`: Updates Entity data in given Registry


 License: MIT