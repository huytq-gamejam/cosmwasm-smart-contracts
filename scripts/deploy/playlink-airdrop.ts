import dotenv from "dotenv";
import { NETWORKS } from '../../config';
import { initialize } from '../base';
import { baseDeploy, ContractInfo } from '../base-deploy';

dotenv.config();

const NETWORK = "juno";
const MNEMONIC = process.env.MNEMONIC;

let deploy = async () => {
  let network = NETWORKS.find(n => n.name === NETWORK);
  if (!network) {
    console.error(`Unknown network: ${NETWORK}`);
    return;
  }
  let [account] = await initialize(network).setup(MNEMONIC);
  let contracts: ContractInfo[] = [
    {
      contractName: "cw20_token",
      initParams: {
        name: "Tether USD",
        symbol: "USDT",
        decimals: 6,
        initial_balances: [{ address: account, amount: "1000000" }],
      }
    },
    {
      contractName: "cw721_token",
      initParams: {
        name: "We All Survived Death",
        symbol: "WASD",
        minter: account,
      }
    },
    {
      contractName: "cw1155_token",
      initParams: { minter: account }
    },
    {
      contractName: "playlink_airdrop",
      initParams: { max_batch_size: "3", fee_per_batch: "1" }
    }
  ];
  await baseDeploy(contracts, NETWORK);
};

deploy();