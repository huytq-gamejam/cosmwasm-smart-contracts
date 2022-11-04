#[cfg(test)]
mod tests {
    use crate::{
        helpers::{AirdropCampaign, Asset, AssetType, NATIVE_DENOM},
        msg::{ExecuteMsg, QueryMsg},
    };
    use cosmwasm_std::{coins, Addr, BlockInfo, Coin, Empty, Timestamp, Uint128};
    use cw_multi_test::{App, AppBuilder, ContractWrapper, Executor};

    const ADMIN: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4a";
    const OPERATOR: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4b";
    const CAMPAIGN_CREATOR: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4c";
    const WINNER_1: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4d";
    const WINNER_2: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4e";
    const CAMPAIGN_ID: &str = "01BX5ZZKBKACTAV9WEVGEMMVRY";

    fn mock_blockchain() -> App {
        AppBuilder::new().build(|router, _, storage| {
            let accounts = vec![ADMIN, OPERATOR, CAMPAIGN_CREATOR];
            for account in accounts.iter() {
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked(*account),
                        coins(1000, NATIVE_DENOM),
                    )
                    .unwrap();
            }
        })
    }

    fn proper_instantiate() -> (App, Addr, Addr, Addr, Addr) {
        let mut blockchain = mock_blockchain();
        let cw20_id = blockchain.store_code(Box::new(ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        )));
        let cw721_id = blockchain.store_code(Box::new(ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        )));
        let cw1155_id = blockchain.store_code(Box::new(ContractWrapper::new(
            cw1155_base::contract::execute,
            cw1155_base::contract::instantiate,
            cw1155_base::contract::query,
        )));
        let airdrop_id = blockchain.store_code(Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )));

        let cw20_address = blockchain
            .instantiate_contract(
                cw20_id,
                Addr::unchecked(ADMIN),
                &cw20_base::msg::InstantiateMsg {
                    name: String::from("Tether USD"),
                    symbol: String::from("USDT"),
                    decimals: 6,
                    initial_balances: vec![cw20::Cw20Coin {
                        address: String::from(Addr::unchecked(CAMPAIGN_CREATOR)),
                        amount: Uint128::new(1000),
                    }],
                    mint: None,
                    marketing: None,
                },
                &[],
                "cw20_token",
                None,
            )
            .unwrap();

        let cw721_address = blockchain
            .instantiate_contract(
                cw721_id,
                Addr::unchecked(ADMIN),
                &cw721_base::msg::InstantiateMsg {
                    name: String::from("We All Survived Death"),
                    symbol: String::from("WASD"),
                    minter: String::from(Addr::unchecked(ADMIN)),
                },
                &[],
                "cw721_token",
                None,
            )
            .unwrap();

        let cw1155_address = blockchain
            .instantiate_contract(
                cw1155_id,
                Addr::unchecked(ADMIN),
                &cw1155_base::msg::InstantiateMsg {
                    minter: String::from(Addr::unchecked(ADMIN)),
                },
                &[],
                "cw1155_token",
                None,
            )
            .unwrap();

        let airdrop_address = blockchain
            .instantiate_contract(
                airdrop_id,
                Addr::unchecked(ADMIN),
                &crate::msg::InstantiateMsg {
                    max_batch_size: 3,
                    fee_per_batch: 1,
                },
                &[],
                "playlink_airdrop",
                None,
            )
            .unwrap();

        (
            blockchain,
            cw20_address,
            cw721_address,
            cw1155_address,
            airdrop_address,
        )
    }

    mod playlink_airdrop {
        use super::*;

        #[test]
        fn playlink_airdrop_test() {
            let (mut blockchain, cw20_address, cw721_address, cw1155_address, airdrop_address) =
                proper_instantiate();

            /* ================= Mint some CW721 and CW1155 tokens ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    cw721_address.clone(),
                    &cw721_base::msg::ExecuteMsg::<cw721_base::Extension, Empty>::Mint(
                        cw721_base::MintMsg::<cw721_base::Extension> {
                            token_id: String::from("8888"),
                            owner: String::from(Addr::unchecked(CAMPAIGN_CREATOR)),
                            token_uri: Some(String::from("https://ipfs.io/ipfs/Q")),
                            extension: None,
                        },
                    ),
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    cw721_address.clone(),
                    &cw721_base::msg::ExecuteMsg::<cw721_base::Extension, Empty>::Mint(
                        cw721_base::MintMsg::<cw721_base::Extension> {
                            token_id: String::from("9999"),
                            owner: String::from(Addr::unchecked(CAMPAIGN_CREATOR)),
                            token_uri: Some(String::from("https://ipfs.io/ipfs/Q")),
                            extension: None,
                        },
                    ),
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    cw1155_address.clone(),
                    &cw1155::Cw1155ExecuteMsg::Mint {
                        to: String::from(Addr::unchecked(CAMPAIGN_CREATOR)),
                        token_id: String::from("1234"),
                        value: Uint128::from(15_u128),
                        msg: None,
                    },
                    &[],
                )
                .unwrap();

            /* ================= Set up operators ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    airdrop_address.clone(),
                    &ExecuteMsg::SetOperators {
                        operators: vec![String::from(Addr::unchecked(OPERATOR))],
                        is_operators: vec![true],
                    },
                    &[],
                )
                .unwrap();

            /* ================= Create a new campaign ================= */
            let mut campaign_starting_time = blockchain.block_info().time.seconds() + 20 * 60; // This campaign will start 20 minute later
            blockchain
                .execute_contract(
                    Addr::unchecked(CAMPAIGN_CREATOR),
                    airdrop_address.clone(),
                    &ExecuteMsg::CreateAirdropCampaign {
                        campaign_id: String::from(CAMPAIGN_ID),
                        assets: vec![
                            Asset {
                                asset_type: AssetType::CW20,
                                asset_address: cw20_address.clone(),
                                asset_id: String::from(""),
                                available_amount: 100,
                            },
                            Asset {
                                asset_type: AssetType::CW20,
                                asset_address: cw20_address.clone(),
                                asset_id: String::from(""),
                                available_amount: 150,
                            },
                            Asset {
                                asset_type: AssetType::CW721,
                                asset_address: cw721_address.clone(),
                                asset_id: String::from("1234"),
                                available_amount: 1,
                            },
                        ],
                        starting_time: campaign_starting_time,
                    },
                    &[Coin {
                        amount: Uint128::from(20_u128),
                        denom: String::from(NATIVE_DENOM),
                    }],
                )
                .unwrap();

            let mut campaign: AirdropCampaign = blockchain
                .wrap()
                .query_wasm_smart(
                    airdrop_address.clone(),
                    &QueryMsg::GetCampaignById {
                        campaign_id: String::from(CAMPAIGN_ID),
                    },
                )
                .unwrap();
            assert_eq!(campaign.campaign_id, CAMPAIGN_ID);
            assert_eq!(campaign.creator, Addr::unchecked(CAMPAIGN_CREATOR));
            assert_eq!(campaign.assets.len(), 3);
            assert_eq!(campaign.assets.get(0).unwrap().asset_address, cw20_address);
            assert_eq!(campaign.assets.get(2).unwrap().asset_address, cw721_address);
            assert_eq!(campaign.max_batch_size, 3);
            assert_eq!(campaign.starting_time, campaign_starting_time);
            assert_eq!(campaign.total_available_assets, 251);
            assert_eq!(campaign.airdrop_fee, 1);

            /* ================= Update this campaign ================= */
            campaign_starting_time = blockchain.block_info().time.seconds() + 8 * 60; // This campaign will start 8 minute later
            blockchain
                .execute_contract(
                    Addr::unchecked(CAMPAIGN_CREATOR),
                    airdrop_address.clone(),
                    &ExecuteMsg::UpdateCampaign {
                        campaign_id: String::from(CAMPAIGN_ID),
                        assets: vec![
                            Asset {
                                asset_type: AssetType::CW721,
                                asset_address: cw721_address.clone(),
                                asset_id: String::from("9999"),
                                available_amount: 1,
                            },
                            Asset {
                                asset_type: AssetType::CW20,
                                asset_address: cw20_address.clone(),
                                asset_id: String::from(""),
                                available_amount: 180,
                            },
                            Asset {
                                asset_type: AssetType::CW20,
                                asset_address: cw20_address.clone(),
                                asset_id: String::from(""),
                                available_amount: 100,
                            },
                            Asset {
                                asset_type: AssetType::CW721,
                                asset_address: cw721_address.clone(),
                                asset_id: String::from("8888"),
                                available_amount: 1,
                            },
                            Asset {
                                asset_type: AssetType::CW1155,
                                asset_address: cw1155_address.clone(),
                                asset_id: String::from("1234"),
                                available_amount: 15,
                            },
                        ],
                        starting_time: campaign_starting_time,
                    },
                    &[Coin {
                        amount: Uint128::from(25_u128),
                        denom: String::from(NATIVE_DENOM),
                    }],
                )
                .unwrap();

            campaign = blockchain
                .wrap()
                .query_wasm_smart(
                    airdrop_address.clone(),
                    &QueryMsg::GetCampaignById {
                        campaign_id: String::from(CAMPAIGN_ID),
                    },
                )
                .unwrap();
            assert_eq!(campaign.campaign_id, CAMPAIGN_ID);
            assert_eq!(campaign.creator, Addr::unchecked(CAMPAIGN_CREATOR));
            assert_eq!(campaign.assets.len(), 5);
            assert_eq!(campaign.assets.get(0).unwrap().asset_address, cw721_address);
            assert_eq!(campaign.assets.get(2).unwrap().asset_address, cw20_address);
            assert_eq!(campaign.max_batch_size, 3);
            assert_eq!(campaign.starting_time, campaign_starting_time);
            assert_eq!(campaign.total_available_assets, 297);
            assert_eq!(campaign.airdrop_fee, 2);

            /* ================= Approve assets ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(CAMPAIGN_CREATOR),
                    cw20_address.clone(),
                    &cw20::Cw20ExecuteMsg::IncreaseAllowance {
                        spender: airdrop_address.clone().into(),
                        amount: Uint128::from(280_u128),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(CAMPAIGN_CREATOR),
                    cw721_address.clone(),
                    &cw721::Cw721ExecuteMsg::Approve {
                        spender: airdrop_address.clone().into(),
                        token_id: String::from("9999"),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(CAMPAIGN_CREATOR),
                    cw721_address.clone(),
                    &cw721::Cw721ExecuteMsg::Approve {
                        spender: airdrop_address.clone().into(),
                        token_id: String::from("8888"),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(CAMPAIGN_CREATOR),
                    cw1155_address.clone(),
                    &cw1155::Cw1155ExecuteMsg::ApproveAll {
                        operator: airdrop_address.clone().into(),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
            let allowance: cw20::AllowanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    cw20_address.clone(),
                    &cw20::Cw20QueryMsg::Allowance {
                        owner: String::from(Addr::unchecked(CAMPAIGN_CREATOR)),
                        spender: airdrop_address.clone().into(),
                    },
                )
                .unwrap();
            assert_eq!(allowance.allowance, Uint128::from(280_u128));

            /* ================= Airdrop CW20 assets ================= */
            let current_block = blockchain.block_info();
            blockchain.set_block(BlockInfo {
                height: current_block.height + 1,
                time: Timestamp::from_seconds(current_block.time.seconds() + 10 * 60), // Forward blockchain clock by 10 minute
                chain_id: current_block.chain_id,
            });
            blockchain
                .execute_contract(
                    Addr::unchecked(OPERATOR),
                    airdrop_address.clone(),
                    &ExecuteMsg::Airdrop {
                        campaign_id: String::from(CAMPAIGN_ID),
                        asset_indexes: vec![1, 2],
                        recipients: vec![
                            String::from(Addr::unchecked(WINNER_1)),
                            String::from(Addr::unchecked(WINNER_2)),
                        ],
                    },
                    &[],
                )
                .unwrap();
            let winner1_cw20_balance: cw20::BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    cw20_address.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: String::from(Addr::unchecked(WINNER_1)),
                    },
                )
                .unwrap();
            let winner2_cw20_balance: cw20::BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    cw20_address,
                    &cw20::Cw20QueryMsg::Balance {
                        address: String::from(Addr::unchecked(WINNER_2)),
                    },
                )
                .unwrap();
            campaign = blockchain
                .wrap()
                .query_wasm_smart(
                    airdrop_address.clone(),
                    &QueryMsg::GetCampaignById {
                        campaign_id: String::from(CAMPAIGN_ID),
                    },
                )
                .unwrap();
            assert_eq!(winner1_cw20_balance.balance.u128(), 180);
            assert_eq!(winner2_cw20_balance.balance.u128(), 100);
            assert_eq!(campaign.total_available_assets, 17);
            assert_eq!(campaign.assets.get(0).unwrap().available_amount, 1);
            assert_eq!(campaign.assets.get(1).unwrap().available_amount, 0);
            assert_eq!(campaign.assets.get(2).unwrap().available_amount, 0);
            assert_eq!(campaign.assets.get(3).unwrap().available_amount, 1);
            assert_eq!(campaign.assets.get(4).unwrap().available_amount, 15);

            /* ================= Airdrop CW721 and CW1155 assets ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(OPERATOR),
                    airdrop_address.clone(),
                    &ExecuteMsg::Airdrop {
                        campaign_id: String::from(CAMPAIGN_ID),
                        asset_indexes: vec![0, 3, 4],
                        recipients: vec![
                            String::from(Addr::unchecked(WINNER_1)),
                            String::from(Addr::unchecked(WINNER_2)),
                            String::from(Addr::unchecked(WINNER_1)),
                        ],
                    },
                    &[],
                )
                .unwrap();
            let cw721_owner1: cw721::OwnerOfResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    cw721_address.clone(),
                    &cw721::Cw721QueryMsg::OwnerOf {
                        token_id: String::from("9999"),
                        include_expired: None,
                    },
                )
                .unwrap();
            let cw721_owner2: cw721::OwnerOfResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    cw721_address,
                    &cw721::Cw721QueryMsg::OwnerOf {
                        token_id: String::from("8888"),
                        include_expired: None,
                    },
                )
                .unwrap();
            let cw1155_balance: cw1155::BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    cw1155_address,
                    &cw1155::Cw1155QueryMsg::Balance {
                        owner: String::from(Addr::unchecked(WINNER_1)),
                        token_id: String::from("1234"),
                    },
                )
                .unwrap();
            blockchain
                .wrap()
                .query_wasm_smart::<AirdropCampaign>(
                    airdrop_address.clone(),
                    &QueryMsg::GetCampaignById {
                        campaign_id: String::from(CAMPAIGN_ID),
                    },
                )
                .unwrap_err(); // Make sure that this campaign is removed when all assets are airdropped
            assert_eq!(cw721_owner1.owner, String::from(Addr::unchecked(WINNER_1)));
            assert_eq!(cw721_owner2.owner, String::from(Addr::unchecked(WINNER_2)));
            assert_eq!(cw1155_balance.balance.u128(), 15_u128);

            /* ================= Admin withdraws all airdrop fee ================= */
            let contract_balance1 = blockchain
                .wrap()
                .query_balance(airdrop_address.clone(), NATIVE_DENOM)
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    airdrop_address.clone(),
                    &ExecuteMsg::WithdrawAirdropFee {
                        recipient: String::from(Addr::unchecked(ADMIN)),
                    },
                    &[],
                )
                .unwrap();
            let contract_balance2 = blockchain
                .wrap()
                .query_balance(airdrop_address, NATIVE_DENOM)
                .unwrap();
            assert_eq!(contract_balance1.amount.u128(), 2);
            assert_eq!(contract_balance2.amount.u128(), 0);
        }
    }
}
