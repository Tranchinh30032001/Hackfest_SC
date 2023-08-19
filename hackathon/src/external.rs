use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId};

#[ext_contract(ext_self)]
pub trait CallbackSelf {
    fn claim_token_callback_near(
        &mut self,
        receiver_id: &AccountId,
        amount: Balance,
        event_id: &EventId,
    );
    fn claim_token_callback_usdt(
        &mut self,
        receiver_id: &AccountId,
        amount: Balance,
        event_id: &EventId,
    );
    fn balance_of_callback(&self, account_id: &AccountId) -> Balance;
    fn storage_deposit_callback_add_token(&mut self);
}
#[ext_contract(ext_ft_fungible_token)]
pub trait FungibleTokenCore {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    );
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_balance_of(&self, account_id: AccountId);
}

#[ext_contract(ext_ft_storage)]
pub trait StorageManagement {
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>);
}
