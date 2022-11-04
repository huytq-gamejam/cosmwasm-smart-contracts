use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::helpers::AirdropCampaign;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AirdropPlatform {
    pub admin: Addr,
    pub max_match_size: u64,
    pub fee_per_batch: u128,
}

pub const AIRDROP_PLATFORM: Item<AirdropPlatform> = Item::new("airdrop_platform");
pub const ALL_CAMPAIGNS: Map<String, AirdropCampaign> = Map::new("all_campaigns");
pub const OPERATORS: Map<Addr, bool> = Map::new("operators");
