use near_sdk::serde_json;
use token_mapping_contract::DepositPurpose;

mod common;

const TOTAL_SUPPLY: u128 = 100_000_000;

#[test]
pub fn test_normal_actions() {
    let total_supply = common::to_ft_amount(TOTAL_SUPPLY);
    let (root, trade_in_token, trade_out_token, token_mapping_contract, users) =
        common::init(total_supply);
    //
    //
    //
    common::ft_transfer_call_fungible_token(
        &root,
        &token_mapping_contract.user_account,
        total_supply / 20,
        serde_json::ser::to_string(&DepositPurpose::DepositTradeOutToken).unwrap(),
        &trade_out_token,
    );
    let balance_of_token_mapping_contract =
        common::get_ft_balance_of(&token_mapping_contract.user_account, &trade_out_token);
    assert_eq!(balance_of_token_mapping_contract.0, total_supply / 20);
    //
    //
    //
    let trade_in_token_balance_before_convert =
        common::get_ft_balance_of(&users[0], &trade_in_token);
    let trade_out_token_balance_before_convert =
        common::get_ft_balance_of(&users[0], &trade_out_token);
    common::register_user_to_fungible_token(&users[0], &trade_out_token);
    common::ft_transfer_call_fungible_token(
        &users[0],
        &token_mapping_contract.user_account,
        total_supply / 20,
        serde_json::ser::to_string(&DepositPurpose::ConvertToTradeOutToken).unwrap(),
        &trade_in_token,
    );
    let trade_in_token_balance_after_convert =
        common::get_ft_balance_of(&users[0], &trade_in_token);
    let trade_out_token_balance_after_convert =
        common::get_ft_balance_of(&users[0], &trade_out_token);
    assert_eq!(
        trade_in_token_balance_after_convert.0,
        trade_in_token_balance_before_convert.0 - total_supply / 20
    );
    assert_eq!(
        trade_out_token_balance_after_convert.0,
        trade_out_token_balance_before_convert.0 + total_supply / 20
    );
    //
    //
    //
    common::register_user_to_fungible_token(&users[1], &trade_out_token);
    common::ft_transfer_call_fungible_token(
        &users[1],
        &token_mapping_contract.user_account,
        total_supply / 20,
        serde_json::ser::to_string(&DepositPurpose::ConvertToTradeOutToken).unwrap(),
        &trade_in_token,
    );

}
