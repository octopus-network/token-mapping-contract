use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::upgrade::Ownable;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    assert_one_yocto, assert_self, env, ext_contract, near_bindgen, AccountId, PanicOnDefault,
    PromiseOrValue,
};

near_sdk::setup_alloc!();

const T_GAS: u64 = 1_000_000_000_000;

#[ext_contract(ext_appchain_anchor)]
trait AppchainAnchor {
    fn sync_basedata_of_wrapped_appchain_token(
        &mut self,
        metadata: FungibleTokenMetadata,
        premined_beneficiary: AccountId,
        premined_balance: U128,
    );
}

#[ext_contract(ext_self)]
trait WrappedAppchainTokenSelf {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct WrappedAppchainToken {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
}

#[near_bindgen]
impl WrappedAppchainToken {
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        premined_beneficiary: ValidAccountId,
        premined_balance: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized.");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            owner_id: owner_id.clone().into(),
        };
        this.token
            .internal_register_account(&env::current_account_id());
        this.token
            .internal_register_account(premined_beneficiary.as_ref());
        this.internal_mint(premined_beneficiary.clone(), premined_balance);
        ext_appchain_anchor::sync_basedata_of_wrapped_appchain_token(
            metadata,
            premined_beneficiary.to_string(),
            premined_balance,
            &owner_id,
            0,
            80_000_000_000_000,
        );
        this
    }
    ///
    #[payable]
    pub fn mint(&mut self, account_id: ValidAccountId, amount: U128) {
        self.assert_owner();
        self.storage_deposit(Some(account_id.clone()), None);
        self.internal_mint(account_id, amount);
    }
    //
    fn internal_mint(&mut self, account_id: ValidAccountId, amount: U128) {
        self.token
            .internal_deposit(&env::current_account_id(), amount.into());
        ext_self::ft_transfer(
            account_id.to_string(),
            amount,
            None,
            &env::current_account_id(),
            1,
            10 * T_GAS,
        );
    }
    ///
    #[payable]
    pub fn burn(&mut self, account_id: ValidAccountId, amount: U128) {
        assert_one_yocto();
        self.assert_owner();
        self.token
            .internal_withdraw(account_id.as_ref(), amount.into());
    }
    ///
    pub fn set_icon(&mut self, icon: String) {
        assert_self();
        let mut metadata = self.metadata.get().unwrap();
        metadata.icon = Some(icon);
        self.metadata.set(&metadata);
    }
}

#[near_bindgen]
impl Ownable for WrappedAppchainToken {
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

near_contract_standards::impl_fungible_token_core!(WrappedAppchainToken, token);
near_contract_standards::impl_fungible_token_storage!(WrappedAppchainToken, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for WrappedAppchainToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
