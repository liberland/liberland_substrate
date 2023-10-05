# Liberland Legislation Pallet

## Overview

The Liberland Legislation pallet handles adding and removing legislations.

### Terminology

- **Tier:** Lower tier legislations are more important then higher tiers.
- **Id:** Unique identifier of a legislation inside a tier. Composed of:
    - **Year**
    - **Index**
- **Section:** Part of legislation that can be amended, repealed or referenced directly.
- **Headcount veto:** Process of legislation repeal driven by citizens.

### Headcount Veto

Legislation pallet allows citizens to submit their veto for given legislation.
After the required percentage of vetos (different for each tier) of vetos is
collected, it's possible to trigger the headcount veto which removes given
legislation.

## Interface

### Dispatchable Functions

- `add_legislation` - Adds a new legislation.
- `amend_legislation` - Change existing section or add a new section to existing legislation.
- `repeal_legislation` - Repeals whole legislation (all sections).
- `repeal_legislation_section` - Repeals single legislation.
- `submit_veto` - Registers veto for given legislation (or its specific section) for the signer.
- `revert_veto` - Removes veto for given legislation (or its specific section) for the signer.
- `trigger_headcount_veto` - Repeals legislation (all sections) if veto count requirements are met for it.
- `trigger_section_headcount_veto` - Repeals legislation section if veto count requirements are met for it.


License: MIT