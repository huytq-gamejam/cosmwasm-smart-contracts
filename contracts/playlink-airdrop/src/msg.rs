use crate::helpers::AirdropCampaign;
use crate::helpers::Asset;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub max_batch_size: u64,
    pub fee_per_batch: u128,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetOperators {
        operators: Vec<String>,
        is_operators: Vec<bool>,
    },
    CreateAirdropCampaign {
        campaign_id: String,
        assets: Vec<Asset>,
        starting_time: u64,
    },
    UpdateCampaign {
        campaign_id: String,
        assets: Vec<Asset>,
        starting_time: u64,
    },
    Airdrop {
        campaign_id: String,
        asset_indexes: Vec<u64>,
        recipients: Vec<String>,
    },
    WithdrawAirdropFee {
        recipient: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(u128)]
    EstimateAirdropFee { num_assets: u64 },

    #[returns(AirdropCampaign)]
    GetCampaignById { campaign_id: String },
}
