import dotenv from "dotenv";
import { execSync } from 'child_process';
import fs from "fs";
import { calculateFee } from '@cosmjs/stargate';
import { JsonObject } from '@cosmjs/cosmwasm-stargate';
import { NETWORKS } from '../config';
import { initialize } from './base';
import contractAddresses from "../contract-addresses.json";

dotenv.config();

const MNEMONIC = process.env.MNEMONIC;

export type ContractInfo = {
  contractName: string,
  initParams: JsonObject;
};

export const baseDeploy = async (infos: ContractInfo[], network: string) => {

  // Validate network
  let networkConfig = NETWORKS.find(n => n.name === network);
  if (!networkConfig) {
    console.error(`Unknown network: ${network}`);
    return;
  }

  // Compile and optimize wasm bytecodes
  let compileResult = execSync("cargo wasm");
  let optimizeResult = execSync(
    `sudo docker run --rm -v "$(pwd)":/code \\
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \\
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \\
      cosmwasm/rust-optimizer-arm64:0.12.8`
  ).toString();

  // Prepare deployer account
  let [account, client] = await initialize(networkConfig).setup(MNEMONIC);
  let balance = await client.getBalance(account, networkConfig.feeToken);
  console.log(`Deployer: ${account}`);
  console.log(`Balance: ${balance.amount}${balance.denom}`);

  for (const contractInfo of infos) {
    const contractName = contractInfo.contractName;
    const initParams = contractInfo.initParams;

    let wasmPath = `artifacts/${contractName}-aarch64.wasm`;
    if (!fs.existsSync(wasmPath)) {
      console.error(`Unknown contract: ${contractName}`);
      continue;
    }

    // Upload wasm binary
    let wasm = fs.readFileSync(wasmPath);
    let uploadFee = calculateFee(networkConfig.fees.upload, networkConfig.gasPrice);
    let uploadResult = await client.upload(account, wasm, uploadFee);
    console.log(`${contractName} code ID: ${uploadResult.codeId}`);

    // Instantiate contract
    let instantiateResponse = await client.instantiate(
      account,
      uploadResult.codeId,
      initParams,
      contractName,
      calculateFee(networkConfig.fees.init, networkConfig.gasPrice)
    );
    console.log(`${contractName}: ${instantiateResponse.contractAddress}`);

    // Save the contract address for later use
    if (!contractAddresses[network])
      contractAddresses[network] = {};
    contractAddresses[network][contractName] = instantiateResponse.contractAddress;
    fs.writeFileSync("contract-addresses.json", JSON.stringify(contractAddresses, null, "\t"));
    console.log("Finish!");
  }
};