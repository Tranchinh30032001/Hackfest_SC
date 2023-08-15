use std::collections::HashSet;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::{
    env, near_bindgen, require, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault,
};

pub type EventId = String;
pub const FT_TRANSFER_GAS: Gas = Gas(10_000_000_000_000);
mod event;
mod external;
mod internal;
mod utils;
use event::*;
use utils::*;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum Prefix {
    ListEvent,
    Events,
    SponserToSponse,
    ClientToEventId,
    SponserToToken,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub events: LookupMap<EventId, Event>,
    pub list_event: UnorderedSet<EventId>,
    //client -> [eventId]
    pub client_to_event_id: LookupMap<AccountId, ClientEvent>,
    //sponser -> sponse
    pub sponser_to_sponse: LookupMap<AccountId, Sponse>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Just initial once time");
        //todo check only initialize one time
        let this = Self {
            owner_id: owner_id.clone(),
            events: LookupMap::new(Prefix::Events.try_to_vec().unwrap()),
            sponser_to_sponse: LookupMap::new(Prefix::SponserToSponse.try_to_vec().unwrap()),
            client_to_event_id: LookupMap::new(Prefix::ClientToEventId.try_to_vec().unwrap()),
            list_event: UnorderedSet::new(Prefix::ListEvent.try_to_vec().unwrap()),
        };
        this
    }

    #[payable]
    pub fn create_event(&mut self, event_id: String, name_event: String, duration: u64) -> Event {
        assert_at_least_one_yocto();
        let time_start = env::block_timestamp();
        let time_end = time_start + duration;
        let owner = env::signer_account_id();
        let event = Event {
            id: event_id.clone(),
            owner: owner.clone(),
            name: name_event.clone(),
            iat: time_start,
            exp: time_end,
            total: 0,
            status: Status::Active,
            pause: false,
            sponsers: vec![],
            map_sponser_tokens: LookupMap::new(Prefix::SponserToToken.try_to_vec().unwrap()),
        };
        let mut list_event_id = HashSet::new();
        list_event_id.insert(event_id.clone());
        let client_event = ClientEvent {
            events: list_event_id,
        };
        self.list_event.insert(&event_id);
        self.client_to_event_id.insert(&owner, &client_event);
        event
    }

    #[payable]
    pub fn sponse_native(&mut self, event_id: EventId, amount: U128) {
        assert_at_least_one_yocto();
        let amount: u128 = amount.into();
        let sender_id = env::predecessor_account_id();
        let attached_deposit = env::attached_deposit();
        require!(
            attached_deposit == amount,
            "The attached_deposit must equal to the amount"
        );
        self.internal_deposit(&sender_id, &event_id, amount)
    }

    #[payable]
    pub fn update_sponse_native(&mut self, event_id: EventId, amount: U128) {
        assert_at_least_one_yocto();
        let amount: u128 = amount.into();
        let sender_id = env::predecessor_account_id();
        let attached_deposit = env::attached_deposit();
        require!(
            attached_deposit == amount,
            "The attached_deposit must equal to the amount"
        );
        self.update_deposit(&sender_id, &event_id, amount)
    }

    #[payable]
    pub fn claim(&mut self, event_id: &EventId) {
        match self.events.get(event_id) {
            Some(res) => {
                if res.status == Status::Cancel {
                    assert_fee_storage_deposit();
                    let init_storage = env::storage_usage();
                    let attached_deposit = env::attached_deposit();
                    let receiver_id = env::signer_account_id();
                    match self.sponser_to_sponse.get(&receiver_id) {
                        Some(res) => {
                            let amount = res
                                .map_event_amount
                                .get(event_id)
                                .unwrap_or_else(|| env::panic_str("EventId is invalid"));
                        }
                        None => {}
                    }
                    refund_deposit(init_storage);
                }
            }
            None => {}
        }
    }
}
