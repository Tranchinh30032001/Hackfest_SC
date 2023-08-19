use crate::EventId;
use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::PromiseOrValue;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    AccountId, Balance,
};
use std::collections::{HashMap, HashSet};
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Status {
    Pending,
    Active,
    Finish,
    Cancel,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Token {
    USDT,
    NEAR,
}
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Event {
    pub id: String,
    pub owner: AccountId,
    pub name: String,
    pub total_near: Balance,
    pub total_usdt: Balance,
    pub status: Status,
    pub sponsers: Vec<AccountId>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientEvent {
    pub events: HashSet<EventId>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Sponse {
    pub events: HashSet<EventId>,
    pub map_event_amount: HashMap<EventId, Amount>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Amount {
    pub token_near: Balance,
    pub token_usdt: Balance,
}
impl Contract {
    pub(crate) fn check_exist_event(&self, event_id: &EventId) -> bool {
        match self.events.get(event_id) {
            Some(_) => true,
            None => false,
        }
    }
    pub(crate) fn check_owner_event(&self, event_id: &EventId) -> bool {
        match self.events.get(event_id) {
            Some(res) => res.owner == env::signer_account_id(),
            None => {
                env::panic_str("EventId is not found");
            }
        }
    }
    pub(crate) fn internal_watch_detail_event(&self, event_id: &EventId) -> Event {
        match self.events.get(&event_id) {
            Some(res) => res,
            None => {
                env::panic_str("EventId is not found");
            }
        }
    }
}
#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        log!(
            "in {} tokens from @{} ft_on_transfer",
            amount.0,
            sender_id.as_ref()
        );
        let token_id = env::predecessor_account_id();
        env::log_str(format!("token_id:{}", token_id).as_str());
        // case user deposit FT with msg on the format: event event_id
        // msg == event_id or more event_id
        if msg != "" {
            let split_msg: Vec<&str> = msg.split(" ").collect();
            if split_msg.len() == 1 {
                if self.check_exist_event(&msg) {
                    let sender_id = env::signer_account_id();
                    let result = self.internal_sponse(&sender_id, &msg, amount.into(), Token::USDT);
                    if result {
                        PromiseOrValue::Value(U128(0))
                    } else {
                        PromiseOrValue::Value(amount)
                    }
                } else {
                    env::panic_str("EventId not exist");
                }
            } else {
                let event_id = split_msg.iter().nth(1).unwrap_or_else(|| {
                    env::panic_str(
                        "The message that the user deposited is not in the correct format",
                    )
                });
                let result = self.internal_more_sponse_usdt(
                    &sender_id,
                    &String::from(*event_id),
                    amount.into(),
                );
                if result {
                    PromiseOrValue::Value(U128(0))
                } else {
                    PromiseOrValue::Value(amount)
                }
            }
        } else {
            log!("Fungible token is unvalid");
            PromiseOrValue::Value(amount)
        }
    }
}
