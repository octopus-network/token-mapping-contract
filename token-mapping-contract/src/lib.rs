use near_contract_standards::upgrade::Ownable;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, serde_json, AccountId, PanicOnDefault, PromiseOrValue,
};

near_sdk::setup_alloc!();

const T_GAS: u64 = 1_000_000_000_000;
const GAS_FOR_FT_TRANSFER: u64 = 10 * T_GAS;

#[ext_contract(ext_fungible_token)]
trait FungibleTokenContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct TokenMappingContract {
    owner_id: AccountId,
    trade_in_token_contract: AccountId,
    trade_out_token_contract: AccountId,
    trade_out_token_deposit: u128,
    exchange_rate: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum DepositPurpose {
    DepositTradeOutToken,
    ConvertToTradeOutToken,
}

#[near_bindgen]
impl TokenMappingContract {
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        trade_in_token_contract: ValidAccountId,
        trade_out_token_contract: ValidAccountId,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized.");
        Self {
            owner_id: owner_id.to_string(),
            trade_in_token_contract: trade_in_token_contract.to_string(),
            trade_out_token_contract: trade_out_token_contract.to_string(),
            trade_out_token_deposit: 0,
            exchange_rate: 100,
        }
    }
    ///
    pub fn change_exchange_rate(&mut self, exchange_rate: u32) {
        self.assert_owner();
        assert!(
            exchange_rate != self.exchange_rate,
            "The convert rate is not changed."
        );
        self.exchange_rate = exchange_rate;
    }
    /// Callback function for `ft_transfer_call` of NEP-141 compatible contracts
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let deposit_purpose: DepositPurpose = match serde_json::from_str(msg.as_str()) {
            Ok(purpose) => purpose,
            Err(_) => {
                log!(
                    "Invalid msg '{}' attached in `ft_transfer_call`. Return deposit.",
                    msg
                );
                return PromiseOrValue::Value(amount);
            }
        };
        match deposit_purpose {
            DepositPurpose::DepositTradeOutToken => {
                assert!(
                    env::predecessor_account_id().eq(&self.trade_out_token_contract),
                    "Invalid deposit account for trade out token. Return deposit."
                );
                self.trade_out_token_deposit += amount.0;
                log!(
                    "Received trade out token deposit {} from '@{}'.",
                    amount.0,
                    &sender_id,
                );
            }
            DepositPurpose::ConvertToTradeOutToken => {
                assert!(
                    env::predecessor_account_id().eq(&self.trade_in_token_contract),
                    "Invalid deposit account for trade in token. Return deposit."
                );
                let send_out_amount = match amount.0.checked_mul(self.exchange_rate.into()) {
                    Some(amount) => amount / 100,
                    None => {
                        log!("Send out amount overflow. Return deposit.");
                        return PromiseOrValue::Value(amount);
                    }
                };
                assert!(send_out_amount > 0, "Send out amount is too small.");
                assert!(
                    send_out_amount <= self.trade_out_token_deposit,
                    "Send out amount is too big."
                );
                self.trade_out_token_deposit -= send_out_amount;
                ext_fungible_token::ft_transfer(
                    sender_id.clone(),
                    U128::from(send_out_amount),
                    None,
                    &self.trade_out_token_contract,
                    1,
                    GAS_FOR_FT_TRANSFER,
                );
                log!(
                    "Tokens are converted for holder '@{}'. Received token amount: {} -> Send out token amount: {}",
                    &sender_id,
                    amount.0,
                    send_out_amount,
                );
            }
        }
        PromiseOrValue::Value(0.into())
    }
}

#[near_bindgen]
impl Ownable for TokenMappingContract {
    //
    fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }
    //
    fn set_owner(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = owner_id;
    }
}
