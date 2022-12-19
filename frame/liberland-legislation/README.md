# Liberland Legislation Pallet

## Overview

The Liberland Legislation pallet handles adding and removing legislations.

### Terminology

- **Tier:** Lower tier legislations are more important then higher tiers.
- **Index:** Unique identifier of a legislation inside a tier.
- **Headcount veto:** Process of legislation repeal driven by citizens.

### Headcount Veto

Legislation pallet allows citizens to submit their veto for given legislation.
After the required percentage of vetos (different for each tier) of vetos is
collected, it's possible to trigger the headcount veto which removes given
legislation.

## Interface

### Dispatchable Functions

#### Signed Origin

Basic actions:
- `submit_veto` - Registers veto for given legislation for the signer.
- `revert_veto` - Removes veto for given legislation for the signer.
- `trigger_headcount_veto` - Removes legislation if veto count requirements are met for it.

#### Root origin

- `add_law` - Adds a new legislation.
- `repeal_law` - Removes legislation.

License: MIT