# Liberland Registry Pallet

## Overview

Registry pallet is a general-purpose pallet for tracking and registering
data about abstract entities.

In Liberland, it's used to implement Company Registry

## Terminology

* Entity - object, identified with AccountId, that can have data attached to it and can be
  registered at a registrar
* Registrar - AccountId that can register Entities in Registry. Each registrar has it's own
  Registry.
* Registry - database of Entities and their data that were registered at given Registrar.
* Deposit - amount of Currency that gets reserved when setting Entity data or requesting
  registration - can be withdrawn when data is removed

## Entity Lifecycle

1. Entity must be created using the `set_entity()` call. This creates
   entity and assigns requested data to it.
2. If Entity wishes to be registered at given registrar:
    1. It must call `request_registration()` to deposit Currency for registration at given
registrar and,     2. The registrar must call `register_entity()` to actually add Entity's data
to the Registry 3. If Entity doesn't need any new registrations, it can call
   `clear_entity()` which will refund the deposit. It doesn't remove the Entity
   from Registries

To update data in Registry, Entity has to follow the same path as on first registration.

## Deposits

Registry pallet requires deposits to cover the cost of storing data in
current blockchain state (which isn't pruned). These deposits always matches
the amount of stored data:
* if there's no prior deposit, total required deposit will be reserved
* if there's a deposit, but now we want to store more data, the additional difference will be
  reserved
* if there's a deposit, but now we want to store less or no data, the excess will be refunded

Formula for required deposit to store `N` bytes of data:
```
deposit = BaseDeposit + N * ByteDeposit
```

Deposits are separate for the current data (`set_entity`, `clear_identity` calls) and for each
registry (as data is stored separately as well).

* `set_entity(data, editable)` requires deposit length of `data` parameter. Will immediately
  reserve/refund any difference.
* `clear_identity()` will refund complete deposit for current data.
* `unregister()` will refund deposit for data at given registrar.
* `request_registration()` will calculate required deposit based on maximum of the current data
  and data stored at given registrar (may be 0). Will immediately reserve if needed, but will
  not refund any excess (see `refund()` for that)
* `refund()` will calculate required deposit based on data stored at given registrar and refund
  any excess deposit.

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

* `registrars`: registrars that should be preset on genesis

## Interface

### Dispatchable Functions

#### Public

These calls can be made from any _Signed_ origin.

* `add_registrar`: Adds a new registrar
* `set_entity`: Adds Entity if needed and sets its current data
* `clear_entity`: Removes current Entity data - doesn't remove Entity from Registries
* `unregister`: Removes Entity from given Registry - called by Registrar
* `request_registration`: Deposits Currency required to register current data (see `set_entity`)
  at given Registry
* `register_entity`: Adds Entity to the Registry
* `refund`: Refunds any excess deposit for given Registry
* `set_registered_entity`: Sets Entity data in given Registry


License: MIT