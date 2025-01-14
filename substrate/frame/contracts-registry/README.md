# Contract Registry Pallet

This pallet was designed to allow users to create text contracts that contain parties. These contracts may be signed by parties as well as by judges. 

## Overview
Any user can create a contract and add some parties to it. These parties may sign the given contract to 'make it valid' and prevent it from removal. 

The contract may be also signed by a judge to make it 'valid' in front of Liberland law.

## Interface

### Dispatchable Functions

#### Public

Basic actions:
- `judge_sign_contract` - Sign the contract as a judge. This will prevent the contract from deletion. This method will sign the contract as a signer AccountId.
- `create_contract` - Anyone may call this method to create a contract. This contract will contain data as well as parties.
- `party_sign_contract` - Only party can call this method. This method will sign the contract as a signer AccountId.
- `remove_contract` - Anyone can call this method and remove a given contract. Remove is possible only if the contract is not signed by anyone.

#### Root

- `add_judge` - Add judge as an AccountId. 
- `remove_judge` - Only root can call this method. This method removes a given judge, leaving his signatures.

#### Data

- Contracts - map containing contract index as well as contract data like content, parties, creator, deposit. (ContractIndex, {
    data: Vec&lt;u8&gt;,
    parties: Vec&lt;AccountId&gt;
    creator: AccountId,
    deposit: Balance
})
- PartiesSignatures - map containing contract index as vec of signatures. (ContractIndex, Vec&lt;AccountId&gt;)
- JudgesSignatures - map containing contract index as vec of signatures. (ContractIndex, Vec&lt;AccountId&gt;)
- Judges - map containing contract AccountIds and boolean. (AccountId, bool)