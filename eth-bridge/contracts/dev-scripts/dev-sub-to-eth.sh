#!/bin/bash

set -exuo pipefail

polkadot-js-api --seed "//Alice" tx.llmBridge.setAdmin 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
polkadot-js-api --seed "//Alice" tx.llmBridge.setState Active
polkadot-js-api --seed "//Alice" tx.llmBridge.deposit 1000000000000 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
