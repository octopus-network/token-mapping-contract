use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::json_types::U128;
use near_sdk_sim::{
    call, deploy, init_simulator, lazy_static_include, runtime::GenesisConfig, to_yocto, view,
    ContractAccount, ExecutionResult, UserAccount,
};

use num_format::{Locale, ToFormattedString};

use mock_fungible_token::MockFungibleTokenContract;
use token_mapping_contract::TokenMappingContractContract;

lazy_static_include::lazy_static_include_bytes! {
    TOKEN_WASM_BYTES => "../out/mock_fungible_token.wasm",
    TOKEN_MAPPING_CONTRACT_WASM_BYTES => "../out/token_mapping_contract.wasm",
}

// Register the given `user` to fungible_token
pub fn register_user_to_fungible_token(
    account: &UserAccount,
    contract: &ContractAccount<MockFungibleTokenContract>,
) {
    let result = call!(
        account,
        contract.storage_deposit(Option::from(account.valid_account_id()), Option::None),
        near_sdk::env::storage_byte_cost() * 125,
        near_sdk_sim::DEFAULT_GAS / 2
    );
    print_execution_result("register_user_to_fungible_token", &result);
    result.assert_success();
}

pub fn ft_transfer_fungible_token(
    sender: &UserAccount,
    receiver: &UserAccount,
    amount: u128,
    fungible_token: &ContractAccount<MockFungibleTokenContract>,
) {
    let result = call!(
        sender,
        fungible_token.ft_transfer(
            receiver.valid_account_id(),
            U128::from(amount),
            Option::None
        ),
        1,
        near_sdk_sim::DEFAULT_GAS
    );
    print_execution_result("ft_transfer_fungible_token", &result);
    result.assert_success();
}

pub fn ft_transfer_call_fungible_token(
    sender: &UserAccount,
    receiver: &UserAccount,
    amount: u128,
    msg: String,
    fungible_token: &ContractAccount<MockFungibleTokenContract>,
) -> ExecutionResult {
    let result = call!(
        sender,
        fungible_token.ft_transfer_call(
            receiver.valid_account_id(),
            U128::from(amount),
            Option::None,
            msg.clone()
        ),
        1,
        near_sdk_sim::DEFAULT_GAS
    );
    print_execution_result("ft_transfer_call_fungible_token", &result);
    result.assert_success();
    result
}

fn get_genesis_config() -> GenesisConfig {
    let mut genesis_config = GenesisConfig::default();
    genesis_config.block_prod_time = 86400 * 1_000_000_000;
    genesis_config
}

pub fn init(
    total_supply: u128,
) -> (
    UserAccount,
    ContractAccount<MockFungibleTokenContract>,
    ContractAccount<MockFungibleTokenContract>,
    ContractAccount<TokenMappingContractContract>,
    Vec<UserAccount>,
) {
    let root = init_simulator(Some(get_genesis_config()));
    let mut users: Vec<UserAccount> = Vec::new();
    //
    // Deploy and initialize contracts
    //
    let trade_in_ft_metadata = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "TradeInToken".to_string(),
        symbol: "TIT".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 18,
    };
    let trade_out_ft_metadata = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "TradeOutToken".to_string(),
        symbol: "TOT".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 18,
    };
    let trade_in_token = deploy! {
        contract: MockFungibleTokenContract,
        contract_id: "trade_in_token",
        bytes: &TOKEN_WASM_BYTES,
        signer_account: root,
        init_method: new(root.valid_account_id(), U128::from(total_supply), trade_in_ft_metadata)
    };
    let trade_out_token = deploy! {
        contract: MockFungibleTokenContract,
        contract_id: "trade_out_token",
        bytes: &TOKEN_WASM_BYTES,
        signer_account: root,
        init_method: new(root.valid_account_id(), U128::from(total_supply), trade_out_ft_metadata)
    };
    let token_mapping_contract = deploy! {
        contract: TokenMappingContractContract,
        contract_id: "token_mapping_contract",
        bytes: &TOKEN_MAPPING_CONTRACT_WASM_BYTES,
        signer_account: root,
        init_method: new(root.valid_account_id(), trade_in_token.valid_account_id(), trade_out_token.valid_account_id())
    };
    //
    // Register token mapping contract to token contracts
    //
    register_user_to_fungible_token(&token_mapping_contract.user_account, &trade_in_token);
    register_user_to_fungible_token(&token_mapping_contract.user_account, &trade_out_token);
    //
    // Create users and transfer a certain amount of OCT token to them
    //
    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    register_user_to_fungible_token(&alice, &trade_in_token);
    ft_transfer_fungible_token(&root, &alice, total_supply / 10, &trade_in_token);
    users.push(alice);
    let bob = root.create_user("bob".to_string(), to_yocto("100"));
    register_user_to_fungible_token(&bob, &trade_in_token);
    ft_transfer_fungible_token(&root, &bob, total_supply / 10, &trade_in_token);
    users.push(bob);
    let charlie = root.create_user("charlie".to_string(), to_yocto("100"));
    register_user_to_fungible_token(&charlie, &trade_in_token);
    ft_transfer_fungible_token(&root, &charlie, total_supply / 10, &trade_in_token);
    users.push(charlie);
    let dave = root.create_user("dave".to_string(), to_yocto("100"));
    register_user_to_fungible_token(&dave, &trade_in_token);
    ft_transfer_fungible_token(&root, &dave, total_supply / 10, &trade_in_token);
    users.push(dave);
    let eve = root.create_user("eve".to_string(), to_yocto("100"));
    register_user_to_fungible_token(&eve, &trade_in_token);
    ft_transfer_fungible_token(&root, &eve, total_supply / 10, &trade_in_token);
    users.push(eve);
    //
    // Return initialized objects
    //
    (
        root,
        trade_in_token,
        trade_out_token,
        token_mapping_contract,
        users,
    )
}

pub fn get_ft_balance_of(
    user: &UserAccount,
    token_contract: &ContractAccount<MockFungibleTokenContract>,
) -> U128 {
    let view_result = view!(token_contract.ft_balance_of(user.valid_account_id()));
    if view_result.is_err() {
        println!("{:#?}", view_result);
    }
    assert!(view_result.is_ok());
    view_result.unwrap_json::<U128>()
}

pub fn to_ft_amount(amount: u128) -> u128 {
    let bt_decimals_base = (10 as u128).pow(18);
    amount * bt_decimals_base
}

pub fn print_execution_result(function_name: &str, result: &ExecutionResult) {
    println!(
        "Gas burnt of function '{}': {}",
        function_name,
        result.gas_burnt().to_formatted_string(&Locale::en)
    );
    let results = result.promise_results();
    for sub_result in results {
        if let Some(sub_result) = sub_result {
            if sub_result.is_ok() {
                let logs = sub_result.logs();
                if logs.len() > 0 {
                    println!("{:#?}", logs);
                }
            } else {
                println!("{:#?}", sub_result.outcome());
            }
        }
    }
    if result.is_ok() {
        let logs = result.logs();
        if logs.len() > 0 {
            println!("{:#?}", logs);
        }
    } else {
        println!("{:#?}", result.outcome());
    }
}
