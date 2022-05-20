# Liberland Blockchain    
<p>
<center>

  <img style="max-height: 250px;" alt="Liberland blockchain node" title="Liberland Logo" src="Liberland_official_znak.png">
</center>
</p>


[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/liberland/liberland_node)](https://github.com/liberland/liberland_node/tags) [![Twitter](https://img.shields.io/badge/Twitter-gray?logo=twitter)](https://twitter.com/liberland)


### How to run a validator:    
```shell
$ git clone https://github.com/liberland/liberland_substrate && cd liberland_substrate/ && cargo build --release 
$ cd scripts/ && bash run_validator.sh
```   
Clone our github repository, compile the node with cargo build --release, navigate into our scripts directory and run out validator  
setup script called run_validator.sh



View live chain:   
https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fl2.laissez-faire.trade#/explorer


Documentation can be found in the (docs/)[docs/] folder


