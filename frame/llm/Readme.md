## Liberland Merit (LLM) Pallet


### Current Functionality:
*  Minting 0.9% of the total supply per year to the treasury
*  Keeping track of the Minted amount
*  Sending and Receiving
*  Locking in currency
*  [MIT License](https://mit-license.org/)
*  for the full feature set, please refer to the [src/lib.rs](src/lib.rs) file


### On-chain Pallet functions:

*  `send_llm`:
Send LLM to a person

*  `lock_llm`:
Freeze current LLM, allowing you to vote

*  `unlock_llm`:
Unlock the frozen assets

*  `createllm`:
Create LLM Asset and premint if the counter is not working

*  `delegated_transfer`:
Request a LLM transfer that needs to be approved by the assembly members

*  `approve_transfer`:
As an assembly member you can approve a transfer of LLM


### Storage, balances and keeping track of LLM
LLM pallet has 2 different storage types:

- `LLMBalance`
- `MintedAmount`

##### `LLMBalance`
Stores Account and Balances in a `StorageMap`, you can query by account

![Polkadot Js Query user account](account_query.png) 


![Polkadot Js Treasury](treasury_account.png)


##### `MintedAmount`
Keeps track of the amount of currently minted(/created) amount of LLM.
This can be queried and will return a `u64` number



https://docs.substrate.io/v3/runtime/storage/ 


### Debugging: 

run node: 

```shell
$ ./target/release/substrate --dev --unsafe-rpc-external --
```

#### Check amount of minted llm
Polkadot.js apps > Developer > Chainstate > llm > minted amount

![Polkadot Js](minted_amount_query.png)


## There are 3 types of owners that interact with llm:

![Minting llm](llm_minting.png)

### Treasury address(py/trsry):
5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z

### LLM Vault(llm/safe): 
5EYCAe5hvejUE1BUTDSnxDfCqVkADRicSKqbcJrduV1KCDmk

### LLM Treasury(llm/trsy):
5EYCAe5hvejUE35Lv2zZBMP1iA41yzs2UoiJuxsCidZPFDzJ


From the start, funds are moved into the llm Vault, the premint is moved into the Treasury, funds are continously moved from the llm vault to the treasury.





### rust build info
```shell
$ rustc -vV
rustc 1.60.0 (7737e0b5c 2022-04-04)
binary: rustc
commit-hash: 7737e0b5c4103216d6fd8cf941b7ab9bdbaace7c
commit-date: 2022-04-04
host: x86_64-unknown-linux-gnu
release: 1.60.0
LLVM version: 14.0.0
```


## Storage:

```
LLMBalance      // llm balance 
LLMPolitics     // allocated in politics, storage is synced and used by other pallets
LLMPoliticsLock // LLM that are frozen in the politics queue
LockedLLM       // locked llm
Withdrawlock    // time lock for withdrawing pooled llm	 
MintedAmount    // Keep track of the amount of minted llm
NextMint        // block number for next llm mint 
Electionlock    // time lock for elections - triggered after unpooling llm
```




### Approved Multisig llm transfers

![Polkadot Js Treasury](treasury_account_query.png)

![Polkadot Js Treasury](check_multisig.png)

![Polkadot Js Treasury](treasuryllm_transfer_with_multisig.png)

![Polkadot Js Treasury](multisig_send_tx.png)

![Polkadot Js Treasury](pending_multisig.png)

![Polkadot Js Treasury](approve_multisig.png)

![Polkadot Js Treasury](pasted_multisig_approved_data.png)

![Polkadot Js Treasury](after_multisig.png)
