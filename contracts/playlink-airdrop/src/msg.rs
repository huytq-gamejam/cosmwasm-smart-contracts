use crate::helpers::{AirdropCampaign, Asset};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub max_batch_size: Uint64,
    pub fee_per_batch: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetOperators {
        operators: Vec<String>,
        is_operators: Vec<bool>,
    },
    SetMaxBatchSize {
        new_size: Uint64,
    },
    SetFeePerBatch {
        new_fee: Uint128,
    },
    CreateAirdropCampaign {
        campaign_id: String,
        assets: Vec<Asset>,
        starting_time: Uint64,
    },
    UpdateCampaign {
        campaign_id: String,
        assets: Vec<Asset>,
        starting_time: Uint64,
    },
    Airdrop {
        campaign_id: String,
        asset_indexes: Vec<Uint64>,
        recipients: Vec<String>,
    },
    WithdrawAirdropFee {
        recipient: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    EstimateAirdropFee { num_assets: Uint64 },

    #[returns(AirdropCampaign)]
    GetCampaignById { campaign_id: String },
}
