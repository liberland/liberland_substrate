## Liberland Merit(LLM) Pallet


### Current Functionality:   
*  Minting 0.9% of the total supply per year to the treasury
*  Keeping track of Minted amount  
*  Sending and Recieving  
*  [MIT License](https://mit-license.org/)   



### On-chain Pallet functions:   

*  send_llm   
Send LLM to a person

*  lock_llm    
Freeze current LLM, allowing you to vote

*  unlock_llm
Unlock the freezed assets

*  createllm
Create LLM Asset and premint if the counter is not working   

*  delegated_transfer
Request a LLM transfer that needs to be approved by the assembly members  


*  approve_transfer
As an assembly member you can approve a transfer   


### Debugging: 

run node: ./target/release/substrate --dev --unsafe-rpc-external --

```shell
$ 
```

#### Check amount of minted llm
Polkadot.js apps > Developer > Chainstate > llm > minted amount

![Polkadot Js](minted_amount_query.png)  

