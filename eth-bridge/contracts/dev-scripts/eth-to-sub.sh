#!/bin/bash

set -exuo pipefail

# fake mint some tokens
cast rpc anvil_impersonateAccount 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707
cast rpc anvil_setBalance 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 9999999999999999999
cast send 0xdc64a140aa3e981100a9beca4e685f962f0cf6c9 --unlocked --from=0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 'mint(address,uint256)' 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266 9999999999999
cast rpc anvil_stopImpersonatingAccount 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707

# approve bridge
cast send 0xdc64a140aa3e981100a9beca4e685f962f0cf6c9 --private-key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 'approve(address,uint256)' 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 9999999999999

# burn tokens
cast send 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 --private-key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 'burn(uint256,bytes32)' 10 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d

