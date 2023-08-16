use crate::EventId;
use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    AccountId, Balance,
};
use std::collections::{HashMap, HashSet};
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum Status {
    Pending,
    Active,
    Finish,
    Cancel,
}
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Event {
    pub id: String,
    pub owner: AccountId,
    pub name: String,
    pub iat: u64,
    pub exp: u64,
    pub total: u128,
    pub status: Status,
    pub pause: bool,
    pub sponsers: Vec<AccountId>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientEvent {
    pub events: HashSet<EventId>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Sponse {
    pub events: HashSet<EventId>,
    pub map_event_amount: HashMap<EventId, Balance>,
}
impl Contract {
    pub(crate) fn check_exist_event(&self, event_id: &EventId) -> bool {
        match self.events.get(event_id) {
            Some(_) => true,
            None => false,
        }
    }
}
