## Liberland Merit(LLM) Pallet


### Current Functionality:   
*  Minting 0.9% of the total supply per year to the treasury
*  Keeping track of Minted amount  
*  Sending and Recieving  
*  [MIT License](https://mit-license.org/)   

pub fn send_llm(
pub fn lock_llm(origin: OriginFor<T
pub fn unlock_llm(origin: OriginFor
pub fn createllm(origin: OriginFor<
pub fn delegated_transfer(origin: O
pub fn approve_transfer(origin: Ori



### Debugging: 

run node: ./target/release/substrate --dev --unsafe-rpc-external --

```shell
$ 
```

#### Check amount of minted llm
Polkadot.js apps > Developer > Chainstate > llm > minted amount

![Polkadot Js](minted_amount_query.png)  

