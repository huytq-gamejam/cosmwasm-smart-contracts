use cosmwasm_std::StdError;
use thiserror::Error;

use crate::helpers::AssetType;

#[derive(Error, Debug)]
pub enum PlaylinkAirdropErr {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("PlaylinkAirdrop: {account:?} is not admin")]
    NotAdmin { account: String },

    #[error("PlaylinkAirdrop: caller is not operator ({account:?})")]
    NotOperator { account: String },

    #[error("PlaylinkAirdrop: lengths mismatch")]
    LengthMismatch {},

    #[error("PlaylinkAirdrop: campaign {campaign_id:?} already exists")]
    CampaignAlreadyCreated { campaign_id: String },

    #[error("PlaylinkAirdrop: insuffient airdrop fee (required {fee:?} {denom:?})")]
    InsufficientAirdropFee { fee: u128, denom: String },

    #[error("PlaylinkAirdrop: starting time too low")]
    LowStartingTime {},

    #[error("PlaylinkAirdrop: invalid asset type ({asset_type:?})")]
    InvalidAssetType { asset_type: AssetType },

    #[error("PlaylinkAirdrop: invalid CW20 asset ID ({asset_id:?})")]
    InvalidAssetId { asset_id: String },

    #[error("PlaylinkAirdrop: invalid CW721 amount ({asset_amount:?})")]
    InvalidAssetAmount { asset_amount: u128 },

    #[error("PlaylinkAirdrop: campaign does not exist ({campaign_id:?})")]
    CampaignNotExists { campaign_id: String },

    #[error("PlaylinkAirdrop: not campaign creator ({campaign_creator:?})")]
    NotCampaignCreator { campaign_creator: String },

    #[error("PlaylinkAirdrop: campaign started, cannot update campaign")]
    UpdateNotAllowed { starting_time: u64 },

    #[error("PlaylinkAirdrop: campaign not start yet ({campaign_id:?})")]
    CampaignNotStarts { campaign_id: String },

    #[error("PlaylinkAirdrop: too many assets airdropped ({num_assets:?})")]
    TooManyAssetsAirdropped { num_assets: u64 },

    #[error("PlaylinkAirdrop: index out of bound ({index:?})")]
    IndexOutOfBound { index: u64 },

    #[error("PlaylinkAirdrop: batch size ({size:?}) must be greater than zero")]
    InvalidMaxBatchSize { size: u64 },
}
