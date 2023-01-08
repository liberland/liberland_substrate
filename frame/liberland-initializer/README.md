# Liberland Initializer Pallet

## Overview

The Liberland Initializer pallet handles setting up citizenships and LLM
balances in genesis block. Especially useful for setting up dev testnets and
state for unit tests.

## Usage:

Add the `liberland-initializer` pallet to the runtime's `Cargo.toml`. Use `[dev-dependencies]` if it's only for unit tests:

```
pallet-liberland-initializer = { path = "../../../frame/liberland-initializer", default-features = false }
```

Make it a part of the runtime. No parameters needed for the `Config` trait:

```
construct_runtime!(
    pub enum Runtime where
        [...]
    {
        [...]
		LiberlandInitializer: pallet_liberland_initializer,
    }
)

impl pallet_liberland_initializer::Config for Runtime {}
```

Add the `LiberlandInitializerConfig` to your `GenesisConfig`:
```
	GenesisConfig {
        [...]
		liberland_initializer: LiberlandInitializerConfig {
			citizenship_registrar, initial_citizens
		},
    }
```

* `citizenship_registrar: Option<AccountId>`: AccountID of account that should be used as an identity registrar for providing citizenship judgements. If `None`, no registrar will be added and citizenships won't be granted to anyone on genesis (but balances will still be respected).
* `initial_citizens: Vec<(AccountId, Balance, Balance)>`: Vector of `(account: AccountId, total_llm: Balance, politipooled_llm: Balance)` - specifies accounts that should get citizenships together with amount of LLM sent to them and amount of LLM that should be politipooled. Note that politipooled LLM will be taked from the `total_llm`, so `(0, 6000, 5000)` will result in account `0` having `5000` politipooled LLM and `1000` free LLM.

License: MIT