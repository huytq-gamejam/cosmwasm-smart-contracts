## For developers

### Add new CosmWasm smart contracts

```shell
$ cd contracts
$ cargo generate --git https://github.com/CosmWasm/cosmwasm-template.git --name <contract-name>
```

### Test all smart contracts

```shell
$ cargo unit-test
```