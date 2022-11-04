use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;

pub const NATIVE_DENOM: &str = "flavor";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum AssetType {
    CW20,
    CW721,
    CW1155,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Asset {
    pub asset_type: AssetType,
    pub asset_address: Addr,
    pub asset_id: String,
    pub available_amount: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AirdropCampaign {
    pub campaign_id: String,
    pub creator: Addr,
    pub assets: Vec<Asset>,
    pub max_batch_size: u64,
    pub starting_time: u64,
    pub total_available_assets: u128,
    pub airdrop_fee: u128,
}
