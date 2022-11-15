# Liberland Blockchain    
<p>
<center>

  <img style="max-height: 250px;" alt="Liberland blockchain node" title="Liberland Logo" src="Liberland_official_znak.png">
</center>
</p>


All code that is committed on behalf of Liberland is contributed using the MIT license.

[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/liberland/liberland_node)](https://github.com/liberland/liberland_node/tags) [![Twitter](https://img.shields.io/badge/Twitter-gray?logo=twitter)](https://twitter.com/liberland)


## Documentation:  
https://github.com/liberland/liberland.github.io/    


### Join Liberland's Technical Node Operator group   
https://matrix.to/#/!YzbTfsgCDANzhNLYpW:matrix.org?via=matrix.org  


### How to run a Liberland Hayek node:    
https://github.com/liberland/liberland.github.io/blob/main/docs/run_validator.md   



### How to run a Liberland Hayek node:    
```shell
wget https://github.com/liberland/liberland_substrate/releases/download/v0.1/liberland_x86_linux_binary
chmod +x liberland_x86_linux_binary
wget https://github.com/liberland/liberland_substrate/raw/main/customSpecRaw.json

./liberland_x86_linux_binary --chain customSpecRaw.json --bootnodes /ip4/162.55.230.227/tcp/30333/p2p/12D3KooWRdDm7tDTR8uL9CQxvnvXrUBPLJfrKuHJaCLZfWz9WzeY --base-path /tmp/hayek
```   


### How to run a validator:
Start node with validator flag:
```
./liberland_x86_linux_binary --chain customSpecRaw.json  --force-authoring     --validator --in-peers 256 --base-path /home/myuser/liberland_chain/  --name myliberland_validator --bootnodes /ip4/162.55.230.227/tcp/30333/p2p/12D3KooWRdDm7tDTR8uL9CQxvnvXrUBPLJfrKuHJaCLZfWz9WzeY

```

*Generate new keys*:
```
$ curl -H ‘Content-Type: application/json’ --data ‘{ “jsonrpc”:”2.0", “method”:”author_rotateKeys”, “id”:1 }’ http://127.0.0.1:9933 
```

*Submit keys node session keys*:

Go to polkadot.js > Developer > extrinsics > session > setKeys(keys, proof), 

paste in the result in the keys bar and place 0x00 in the proof bar

Make sure the transaction goes throw, wait 6 seconds, you can now restart your node.

Note:
It helps if you nominate your validator to make it come into the validator pool faster. Bond LLD to it.


View live chain:   
https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fl2.laissez-faire.trade#/explorer


Documentation can be found in the (docs/)[docs/] folder


