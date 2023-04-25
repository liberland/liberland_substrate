# Liberland Federated Bridge Pallet

## Overview

Federated Bridge Pallet is a part of Substrate <-> ETH bridge designed for
bridging LLM and LLD tokens to Ethereum network.

## Terminology

* Bridge - complete system for transferring LLD/LLM to/from ETH, consisting
  of the bridge pallet, relays, watchers and bridge contract.
* Bridge pallet - part of Substrate chain - handles locking and unlocking
* Bridge contract - contract deployed on ETH - handles minting and burning 
* Mint/burn - create/destroy wrapped token on ETH side
* Lock/unlock - lock LLD/LLM on substrate side while it's wrapped in tokens on ETH side.
* Stop - state in which bridge stops all actions
* Federation - set of relays, capable of minting and unlocking.
* Relay - offchain software that monitors locks on substrate and burns on eth and relays them to the other chain
* Watcher - monitors if relays don't misbehave - has right to stop bridge at any time
* Admin - someone with rights to stop/resume bridge
* Superadmin - account with rights to add/remove relay rights

## Bridge configuration

### Option 1: genesis / chainspec

If you're starting a new chain, preconfigure the bridge using chain spec.
See the GenesisConfig struct for details.

### Option 2: using ForceOrigin

If you're chain is running, but you have some trusted chain authority (like sudo key):
* set the ForceOrigin in runtime config
* using ForceOrigin call `set_admin` and `set_super_admin`
* using SuperAdmin call `add_relay` for all your relays
* using SuperAdmin call `set_votes_required`
* using SuperAdmin or Admin call `add_watcher` for all your watchers
* using SuperAdmin or Admin call `set_state(Active)`

### Option 3: migration

Write migration that will run on Runtime Upgrade that set everythin you
need. See GenesisBuild implementation for example.

## Typical ETH -> Substrate transfer flow

1. User deposits wrapped tokens to eth contract, a Receipt is issued as an event
2. Relays see the Receipt event and relay it to bridge using `vote_withdraw` call
3. User waits until `VotesRequired` votes are cast
4. User calls `withdraw` on the bridge.

## Typical Substrate -> ETH transfer flow

1. User deposits native LLM/LLD to to Bridge pallet using `deposit` call, a
   Receipt is issued as an event
2. Relays see the Receipt event and relay it to Ethereum contract
3. User waits until required number of votes are cast by relays on Ethereum side
4. User calls `withdraw` on the etherum side.

## Economics

Running relays requires paying for resource use (CPU, storage, network) and
both Substrate and Ethereum network fees. As such, they're rewarded on each
successful withdrawal by user.

The fee charged to user on withdrawal is set with `set_fee` call and can be
queried from pallet's storage.

This fee is divided equally between relays that actually cast vote on given
transfer.

## Security

Following security features are implemented substrate-side:
* bridge is stopped if 2 relays claim different details about given Receipt
  (different amount, recipient etc)
* bridge can be stopped by a watcher at any time
* bridge doesn't mint any new funds - only funds stored in bridge (a.k.a.
  existing as wrapped tokens on Eth side) are at risk
* bridge enforces rate-limit on withdrawals
* there's a delay between approval of withdrawal and actually allowing it

## Pallet Config

* `Currency` - Currency in which Fee will be charged (should be the same as
   curreny of fees for extrinsics)
* `Token` - fungible implementing Transfer trait that is bridged between
   networks
* `PalletId` - used to derive AccountId for bridge wallet
* `MaxRelays`
* `MaxWatchers`
* `ForceOrigin` - origin that's authorized to set admin and super admin


## Interface

### Dispatchable Functions

* `deposit`: Entrypoint for starting substrate -> eth transfer
* `vote_withdraw`: Call for relays to vote on approving eth -> substrate transfers
* `withdraw`: Call for users to finish approved eth -> substrate transfer
* `set_fee`: Set withdrawal fee
* `set_votes_required`: Set number of votes required to approve eth -> substrate transfer
* `add_relay`: Adds new relay
* `remove_relay`: Removes relay
* `add_watcher`: Adds new watcher
* `remove_watcher`: Removes watcher
* `set_state`: Allows stopping/resuming bridge
* `emergency_stop`: Call for Watchers to stop bridge in case of detected abuse
* `set_admin`: Sets new admin
* `set_super_admin`: Sets new super admin


License: MIT