# Liberland Bridge Relay
## Starting on testnet

TBD

## Starting for development

### Development using docker:

``` bash
> docker-compose -f docker-compose.dev.yml up -d
```

Cleanup db (for example after restarting eth/liberland nodes):

``` bash
> docker-compose -f docker-compose.dev.yml rm -vs relay
> docker-compose -f docker-compose.dev.yml up -d
```

### Development local:
Run the below command to get relay sub-commands:
``` bash
cargo run  -- --help
```

How to run a local relay?

1. Make sure Liberland blockchain and EVM blockchain are running in the background
2. Init new relay config using:
3. Run the relay using the below command:
``` bash
cargo run  -- --config config.toml run
```

Add -v to the above command to run the relay in verbose mode.