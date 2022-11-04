#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    coins, entry_point, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw1155::Cw1155ExecuteMsg;
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use cw721::Cw721ExecuteMsg;

use crate::{
    error::PlaylinkAirdropErr,
    helpers::{AirdropCampaign, Asset, AssetType, NATIVE_DENOM},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{AirdropPlatform, AIRDROP_PLATFORM, ALL_CAMPAIGNS, OPERATORS},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:playlink-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, PlaylinkAirdropErr> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let airdrop_platform = AirdropPlatform {
        admin: info.sender.clone(),
        max_match_size: msg.max_batch_size,
        fee_per_batch: msg.fee_per_batch,
    };
    AIRDROP_PLATFORM.save(deps.storage, &airdrop_platform)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("max_batch_size", msg.max_batch_size.to_string())
        .add_attribute("fee_per_batch", msg.fee_per_batch.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, PlaylinkAirdropErr> {
    match msg {
        ExecuteMsg::SetOperators {
            operators,
            is_operators,
        } => execute::set_operators(deps, info, operators, is_operators),
        ExecuteMsg::CreateAirdropCampaign {
            campaign_id,
            assets,
            starting_time,
        } => execute::create_airdrop_campaign(deps, env, info, campaign_id, assets, starting_time),
        ExecuteMsg::UpdateCampaign {
            campaign_id,
            assets,
            starting_time,
        } => execute::update_campaign(deps, env, info, campaign_id, assets, starting_time),
        ExecuteMsg::Airdrop {
            campaign_id,
            asset_indexes,
            recipients,
        } => execute::airdrop(deps, env, info, campaign_id, asset_indexes, recipients),
        ExecuteMsg::WithdrawAirdropFee { recipient } => {
            execute::withdraw_airdrop_fee(deps, env, info, recipient)
        }
    }
}

pub mod execute {
    use super::{query::estimate_airdrop_fee, *};

    pub fn set_operators(
        deps: DepsMut,
        info: MessageInfo,
        operators: Vec<String>,
        is_operators: Vec<bool>,
    ) -> Result<Response, PlaylinkAirdropErr> {
        if info.sender != AIRDROP_PLATFORM.load(deps.storage)?.admin {
            return Err(PlaylinkAirdropErr::NotAdmin {
                account: info.sender.into(),
            });
        }
        if operators.len() != is_operators.len() {
            return Err(PlaylinkAirdropErr::LengthMismatch {});
        }
        for (i, new_operator) in operators.iter().enumerate() {
            if *is_operators.get(i).unwrap() {
                OPERATORS.save(deps.storage, deps.api.addr_validate(&new_operator)?, &true)?;
            } else {
                OPERATORS.save(deps.storage, deps.api.addr_validate(&new_operator)?, &false)?;
            }
        }
        Ok(Response::new().add_attribute("action", "set_operators"))
    }

    pub fn create_airdrop_campaign(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        campaign_id: String,
        assets: Vec<Asset>,
        starting_time: u64,
    ) -> Result<Response, PlaylinkAirdropErr> {
        // Check if campaign exists
        if ALL_CAMPAIGNS.has(deps.storage, campaign_id.clone()) {
            return Err(PlaylinkAirdropErr::CampaignAlreadyCreated { campaign_id });
        }

        // Check payment
        let airdrop_fee = estimate_airdrop_fee(deps.as_ref(), assets.len() as u64)?;
        let mut payment = 0;
        match info
            .funds
            .iter()
            .find(|c| c.denom == String::from(NATIVE_DENOM))
        {
            Some(coin) => payment = coin.amount.u128(),
            _ => (),
        }
        if payment < airdrop_fee {
            return Err(PlaylinkAirdropErr::InsufficientAirdropFee {
                fee: airdrop_fee,
                denom: String::from(NATIVE_DENOM),
            });
        } else {
            // Return excess
            BankMsg::Send {
                to_address: info.sender.clone().into(),
                amount: coins(payment - airdrop_fee, NATIVE_DENOM),
            };
        }

        // Validate data
        if env.block.time.seconds() >= starting_time {
            return Err(PlaylinkAirdropErr::LowStartingTime {});
        }
        for asset in assets.iter() {
            if asset.asset_type as u8 > 2 {
                return Err(PlaylinkAirdropErr::InvalidAssetType {
                    asset_type: asset.asset_type.clone(),
                });
            }
            if asset.asset_type == AssetType::CW20 && asset.asset_id != String::from("") {
                return Err(PlaylinkAirdropErr::InvalidAssetId {
                    asset_id: asset.asset_id.clone(),
                });
            }
            if asset.asset_type == AssetType::CW721 && asset.available_amount != 1 {
                return Err(PlaylinkAirdropErr::InvalidAssetAmount {
                    asset_amount: asset.available_amount,
                });
            }
        }

        // Create new airdrop campaign
        let max_batch_size = AIRDROP_PLATFORM.load(deps.storage)?.max_match_size;
        ALL_CAMPAIGNS.save(
            deps.storage,
            campaign_id.clone(),
            &AirdropCampaign {
                campaign_id,
                creator: info.sender,
                assets: assets.clone(),
                max_batch_size,
                starting_time,
                total_available_assets: assets.iter().map(|asset| asset.available_amount).sum(),
                airdrop_fee,
            },
        )?;

        Ok(Response::new().add_attribute("action", "create_airdrop_campaign"))
    }

    pub fn update_campaign(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        campaign_id: String,
        assets: Vec<Asset>,
        starting_time: u64,
    ) -> Result<Response, PlaylinkAirdropErr> {
        // Make sure that this campaign exists
        if !ALL_CAMPAIGNS.has(deps.storage, campaign_id.clone()) {
            return Err(PlaylinkAirdropErr::CampaignNotExists { campaign_id });
        }
        let campaign = ALL_CAMPAIGNS.load(deps.storage, campaign_id.clone())?;

        // Check campaign ownership
        if campaign.creator != info.sender {
            return Err(PlaylinkAirdropErr::NotCampaignCreator {
                campaign_creator: campaign.creator.into(),
            });
        }

        // Make sure that this campaign has not started yet
        if env.block.time.seconds() >= campaign.starting_time {
            return Err(PlaylinkAirdropErr::UpdateNotAllowed {
                starting_time: campaign.starting_time,
            });
        }

        // Check payment
        let new_airdrop_fee = estimate_airdrop_fee(deps.as_ref(), assets.len() as u64)?;
        let mut payment = 0;
        let mut messages: Vec<SubMsg> = vec![];
        match info
            .funds
            .iter()
            .find(|c| c.denom == String::from(NATIVE_DENOM))
        {
            Some(coin) => payment += coin.amount.u128(),
            _ => (),
        }
        if new_airdrop_fee > campaign.airdrop_fee {
            if payment < new_airdrop_fee - campaign.airdrop_fee {
                return Err(PlaylinkAirdropErr::InsufficientAirdropFee {
                    fee: new_airdrop_fee - campaign.airdrop_fee,
                    denom: String::from(NATIVE_DENOM),
                });
            } else {
                // Return excess
                messages.push(SubMsg::new(BankMsg::Send {
                    to_address: info.sender.clone().into(),
                    amount: coins(
                        payment + campaign.airdrop_fee - new_airdrop_fee,
                        NATIVE_DENOM,
                    ),
                }));
            }
        }

        // Validate data
        if env.block.time.seconds() >= starting_time {
            return Err(PlaylinkAirdropErr::LowStartingTime {});
        }
        for asset in assets.iter() {
            if asset.asset_type as u8 > 2 {
                return Err(PlaylinkAirdropErr::InvalidAssetType {
                    asset_type: asset.asset_type.clone(),
                });
            }
            if asset.asset_type == AssetType::CW20 && asset.asset_id != "" {
                return Err(PlaylinkAirdropErr::InvalidAssetId {
                    asset_id: asset.asset_id.clone(),
                });
            }
            if asset.asset_type == AssetType::CW721 && asset.available_amount != 1 {
                return Err(PlaylinkAirdropErr::InvalidAssetAmount {
                    asset_amount: asset.available_amount,
                });
            }
        }

        // Update campaign info
        let max_batch_size = AIRDROP_PLATFORM.load(deps.storage)?.max_match_size;
        ALL_CAMPAIGNS.save(
            deps.storage,
            campaign_id.clone(),
            &AirdropCampaign {
                campaign_id,
                creator: info.sender,
                assets: assets.clone(),
                max_batch_size,
                starting_time,
                total_available_assets: assets.iter().map(|asset| asset.available_amount).sum(),
                airdrop_fee: new_airdrop_fee,
            },
        )?;

        Ok(Response::new()
            .add_attribute("action", "update_campaign")
            .add_submessages(messages))
    }

    pub fn airdrop(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        campaign_id: String,
        asset_indexes: Vec<u64>,
        recipients: Vec<String>,
    ) -> Result<Response, PlaylinkAirdropErr> {
        // Only operators can airdrop
        if !OPERATORS.has(deps.storage, info.sender.clone()) {
            return Err(PlaylinkAirdropErr::NotOperator {
                account: info.sender.into(),
            });
        }

        // Make sure that this campaign exists
        if !ALL_CAMPAIGNS.has(deps.storage, campaign_id.clone()) {
            return Err(PlaylinkAirdropErr::CampaignNotExists { campaign_id });
        }
        let mut campaign = ALL_CAMPAIGNS.load(deps.storage, campaign_id.clone())?;

        // Make sure that this campaign has started
        if env.block.time.seconds() < campaign.starting_time {
            return Err(PlaylinkAirdropErr::CampaignNotStarts { campaign_id });
        }

        // Validate data
        if asset_indexes.len() != recipients.len() {
            return Err(PlaylinkAirdropErr::LengthMismatch {});
        }
        if asset_indexes.len() as u64 > campaign.max_batch_size {
            return Err(PlaylinkAirdropErr::TooManyAssetsAirdropped {
                num_assets: asset_indexes.len() as u64,
            });
        }

        // Airdrop
        let mut messages: Vec<SubMsg> = vec![];
        for (i, asset_index) in asset_indexes.iter().enumerate() {
            if *asset_index as usize >= campaign.assets.len() {
                return Err(PlaylinkAirdropErr::IndexOutOfBound {
                    index: *asset_index,
                });
            }
            let asset = campaign.assets.get_mut(*asset_index as usize).unwrap();
            let recipient = deps
                .api
                .addr_validate(recipients.get(i).unwrap().as_str())?
                .into();
            match asset.asset_type {
                AssetType::CW20 => {
                    let message = Cw20ExecuteMsg::TransferFrom {
                        owner: campaign.creator.clone().into(),
                        recipient,
                        amount: Uint128::from(asset.available_amount),
                    };
                    let airdrop_msg = SubMsg::new(WasmMsg::Execute {
                        contract_addr: asset.asset_address.clone().into(),
                        msg: to_binary(&message)?,
                        funds: vec![],
                    });
                    messages.push(airdrop_msg);
                }
                AssetType::CW721 => {
                    let message = Cw721ExecuteMsg::TransferNft {
                        recipient,
                        token_id: asset.asset_id.clone(),
                    };
                    let airdrop_msg = SubMsg::new(WasmMsg::Execute {
                        contract_addr: asset.asset_address.clone().into(),
                        msg: to_binary(&message)?,
                        funds: vec![],
                    });
                    messages.push(airdrop_msg);
                }
                AssetType::CW1155 => {
                    let message = Cw1155ExecuteMsg::SendFrom {
                        from: campaign.creator.clone().into(),
                        to: recipient,
                        token_id: asset.asset_id.clone(),
                        value: Uint128::from(asset.available_amount),
                        msg: None,
                    };
                    let airdrop_msg = SubMsg::new(WasmMsg::Execute {
                        contract_addr: asset.asset_address.clone().into(),
                        msg: to_binary(&message)?,
                        funds: vec![],
                    });
                    messages.push(airdrop_msg);
                }
            }
            campaign.total_available_assets -= asset.available_amount;
            asset.available_amount = 0;
        }

        // Update status or remove
        if campaign.total_available_assets > 0 {
            ALL_CAMPAIGNS.save(deps.storage, campaign_id, &campaign)?;
        } else {
            ALL_CAMPAIGNS.remove(deps.storage, campaign_id);
        }

        Ok(Response::new()
            .add_attribute("action", "airdrop")
            .add_submessages(messages))
    }

    pub fn withdraw_airdrop_fee(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
    ) -> Result<Response, PlaylinkAirdropErr> {
        if info.sender != AIRDROP_PLATFORM.load(deps.storage)?.admin {
            return Err(PlaylinkAirdropErr::NotAdmin {
                account: info.sender.into(),
            });
        }
        let message = SubMsg::new(BankMsg::Send {
            to_address: deps.api.addr_validate(recipient.as_str())?.into(),
            amount: vec![deps
                .querier
                .query_balance(env.contract.address, String::from(NATIVE_DENOM))?],
        });
        Ok(Response::new()
            .add_attribute("action", "withdraw_airdrop_fee")
            .add_submessage(message))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::EstimateAirdropFee { num_assets } => {
            to_binary(&query::estimate_airdrop_fee(deps, num_assets)?)
        }
        QueryMsg::GetCampaignById { campaign_id } => {
            to_binary(&query::get_campaign_by_id(deps, campaign_id)?)
        }
    }
}

pub mod query {
    use super::*;

    pub fn get_campaign_by_id(deps: Deps, campaign_id: String) -> StdResult<AirdropCampaign> {
        return ALL_CAMPAIGNS.load(deps.storage, campaign_id);
    }

    pub fn estimate_airdrop_fee(deps: Deps, num_assets: u64) -> StdResult<u128> {
        let platform = AIRDROP_PLATFORM.load(deps.storage)?;
        let num_required_batches =
            (num_assets + platform.max_match_size - 1) / platform.max_match_size;
        Ok(num_required_batches as u128 * platform.fee_per_batch)
    }
}
