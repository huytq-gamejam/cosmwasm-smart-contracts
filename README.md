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

### Compile and optimize all smart contracts

Create the deployment script in the `scripts/deploy` folder and use the following command to deploy:

```shell
$ ts-node scripts/deploy/<contract-name>.ts
```