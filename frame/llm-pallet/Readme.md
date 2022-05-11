## Liberland Merit(LLM) Pallet


### Current Functionality:   
*  Minting 0.9% of the total supply per year to the treasury
*  Keeping track of Minted amount  
*  Sending and Recieving  
*  Locking in currency
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
As an assembly member you can approve a transfer of LLM         


### Storage, balances and keeping track of LLM      
LLM pallet has 2 different storage types:      

##### LLMBalance    
Stores Account and Balances in storagemap, you can query by account  

![Polkadot Js Query user account](account_query.png) 


![Polkadot Js Treasury](treasury_account.png)


##### MintedAmount    
Keeps track of the amount of current minted(/created) amount of LLM   
This can be queried and will return a number(u64)      



https://docs.substrate.io/v3/runtime/storage/    


### Debugging: 

run node: 

```shell
$ ./target/release/substrate --dev --unsafe-rpc-external --
```

#### Check amount of minted llm
Polkadot.js apps > Developer > Chainstate > llm > minted amount

![Polkadot Js](minted_amount_query.png)  



Treasury address:
5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z