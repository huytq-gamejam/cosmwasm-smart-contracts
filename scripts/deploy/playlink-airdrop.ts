import dotenv from "dotenv";
import fs from "fs";
import { baseDeploy, compile, ContractInfo } from '../base-deploy';
import contractAddresses from "../../contract-addresses.json";

dotenv.config();

const NETWORK = "juno";
const ADMIN = "juno10w2pwzxaacsj508ma5ruz5wnhn83tld78z5cjp";

const CW20_TOKEN = "cw20_token";
const CW721_TOKEN = "cw721_token";
const CW1155_TOKEN = "cw1155_token";
const PLAYLINK_AIRDROP = "playlink_airdrop";

let deploy = async () => {
  let contracts: ContractInfo[] = [
    {
      contractName: CW20_TOKEN,
      initParams: {
        name: "Tether USD",
        symbol: "USDT",
        decimals: 6,
        initial_balances: [{ address: ADMIN, amount: "1000000" }],
      }
    },
    {
      contractName: CW721_TOKEN,
      initParams: {
        name: "We All Survived Death",
        symbol: "WASD",
        minter: ADMIN,
      }
    },
    {
      contractName: CW1155_TOKEN,
      initParams: { minter: ADMIN }
    },
    {
      contractName: PLAYLINK_AIRDROP,
      initParams: { max_batch_size: "3", fee_per_batch: "1" }
    }
  ];

  await compile();

  for (const contract of contracts) {
    let contractAddress = await baseDeploy(contract, NETWORK);
    if (!contractAddresses[NETWORK])
      contractAddresses[NETWORK] = {} as any;
    contractAddresses[NETWORK][contract.contractName] = contractAddress;
  }

  fs.writeFileSync("contract-addresses.json", JSON.stringify(contractAddresses, null, "\t"));
  console.log("Finish!");
};

deploy();