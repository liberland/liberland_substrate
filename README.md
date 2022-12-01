# Liberland Blockchain
<p>
<center>
   <img src="https://lgl.liberland.org/uploads/-/system/appearance/header_logo/1/Liberland_vlajka.png" alt="Liberland Logo" style="height: 68px; width:100px;"/>

</center>
</p>

All code that is committed on behalf of Liberland is distributed under the MIT license. 
You may find a copy of the MIT license [here](https://github.com/liberland/liberland_substrate/blob/main/LICENSE-MIT). Alternatively, you may visit [OpenSource.org](https://opensource.org/licenses/MIT).

[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/liberland/liberland_substrate)](https://github.com/liberland/liberland_substrate/tags) [![Twitter](https://img.shields.io/badge/Twitter-gray?logo=twitter)](https://twitter.com/Liberland_org)


## Documentation:
https://github.com/liberland/liberland.github.io/


### Join Liberland's Technical Node Operator group
https://matrix.to/#/!YzbTfsgCDANzhNLYpW:matrix.org?via=matrix.org


### How to run a Liberland Hazlitt Validator:
https://github.com/liberland/liberland.github.io/blob/main/docs/run_validator.md

## Docker image:
`$ docker pull laissezfaire/liberland-node:0.3.2`


### How to run a Liberland Hazlitt node:
```shell
git clone https://github.com/liberland/liberland_substrate/
cd liberland_substrate && cargo build --release
./target/release/substrate --chain specs/hazlittv3.3.raw --bootnodes /ip4/162.55.230.230/tcp/30333/p2p/12D3KooWPhfahTY7p8pRshMwPbEhp5zAahyu4TwbjXqgGEUoavpr  --base-path /tmp/hazlitt
```

## Current bootnodes:
```
/ip4/162.55.230.230/tcp/30333/p2p/12D3KooWPhfahTY7p8pRshMwPbEhp5zAahyu4TwbjXqgGEUoavpr
```


View live chain:
https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fl2.laissez-faire.trade#/explorer


## Local setup
Install Rust and cargo
Run
cargo build --release
cargo run --release -- --dev

See dev chain at
https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944
