import { baseDeploy } from '../base-deploy';

const MAX_BATCH_SIZE = "3";
const FEE_PER_BATCH = "1";

let deploy = async () => {
  await baseDeploy(
    "playlink_airdrop",
    { max_batch_size: MAX_BATCH_SIZE, fee_per_batch: FEE_PER_BATCH },
    "juno"
  );
};

deploy();