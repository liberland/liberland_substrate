# Liberland Blockchain    
<p>
<center>

  <img style="max-height: 250px;" alt="Liberland blockchain node" title="Liberland Logo" src="Liberland_official_znak.png">
</center>
</p>


[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/liberland/liberland_node)](https://github.com/liberland/liberland_node/tags) [![Twitter](https://img.shields.io/badge/Twitter-gray?logo=twitter)](https://twitter.com/liberland)


### Join Liberland's Technical Node Operator group   
https://matrix.to/#/!YzbTfsgCDANzhNLYpW:matrix.org?via=matrix.org  


### How to run a Liberland Hayek node:    
```shell

wget https://github.com/liberland/liberland_substrate/releases/download/v0.1/liberland_x86_linux_binary
chmod +x liberland_x86_linux_binary
wget https://github.com/liberland/liberland_substrate/raw/main/customSpecRaw.json

./liberland_x86_linux_binary --chain customSpecRaw.json --bootnodes /ip4/162.55.230.227/tcp/30333/p2p/12D3KooWAFgVLpK4PueUo7sZw1hFfwf4VGR77DduAJLiWTaMUSWZ --base-path /tmp/hayek


```   
Clone our github repository, compile the node with cargo build --release, navigate into our scripts directory and run out validator  
setup script called run_validator.sh



View live chain:   
https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fl2.laissez-faire.trade#/explorer


Documentation can be found in the (docs/)[docs/] folder


