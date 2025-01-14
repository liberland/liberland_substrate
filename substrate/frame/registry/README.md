 # Liberland Registry Pallet

 ## Overview

 Registry pallet is a general-purpose pallet for tracking and registering
 data about abstract entities.

 In Liberland, it's used to implement Company Registry.

 ## Terminology

 * Entity - object, identified with EntityId, that has data attached to it and can be registered
   at a registrar
 * Registrar - AccountId that can register Entities in Registry.
 * Registry - database of Entities and their data that were registered.
 * Deposit - amount of Currency that gets reserved when requesting registration - refunded data
   is removed
 * Owner - AccountId that can request registrations for Entity

 ## Entity Lifecycle

 1. Entity is created by owner with `request_entity()` call. This assigns an
    EntityId and requests first registration.
 2. If Registrar approves Entity's data, they call `register_entity()` which
    moves data from Registration Request to the Registry.
 3. To update the data or register at additional Registry, Entity's owner can
    use `request_registration()` call.
 4. Entity can be removed from Registry by the Registrar - deposit will be refunded.

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

 * `request_entity(registry, data, editable)` requires deposit for length of `data` parameter.
   Will immediately reserve the required deposit.
 * `request_registration(registry, data, editable)` requires deposit for length of `data`
   parameter. Will refund deposit of previous pending request (if any) and immediately reserve
   the new required deposit.
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
 * `EntityOrigin` - origin of entities - must return AccountId on success
 * `EntityData` - type that will be used to store and process Entities data
 * `EntityId` - type that will be used to identify Entities - usually `u32` or bigger unsigned
   int type
 * `WeightInfo` - see [Substrate docs](https://docs.substrate.io/reference/how-to-guides/weights/use-custom-weights/)

 ## Genesis Config

 * `registries`: registries that should be preset on genesis
 * `entities`: entities that should exist on genesis - will collect deposits from owners

 ## Interface

 ### Dispatchable Functions

 * `add_registry`: Adds a new registrar
 * `request_entity`: Creates Entity, assigns EntityId and requests first registration
 * `request_registration`: Requests an additional or updated registration of Entity in Registry
 * `cancel_request`: Cancels registration request
 * `unregister`: Removes Entity from given Registry
 * `register_entity`: Adds Entity to the Registry
 * `set_registered_entity`: Updates Entity data in given Registry


 License: MIT