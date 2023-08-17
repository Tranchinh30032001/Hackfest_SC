use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId};

#[ext_contract(ext_self)]
pub trait CallbackSelf {
    fn claim_token_callback(&mut self, receiver_id: AccountId, amount: Balance, event_id: EventId);
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
}
