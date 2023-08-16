use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::{
    env, near_bindgen, require, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault,
};
use std::collections::HashSet;

pub type EventId = String;
pub const FT_TRANSFER_GAS: Gas = Gas(10_000_000_000_000);
mod callback;
mod event;
mod external;
mod internal;
mod test;
mod utils;
use event::*;
use utils::*;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum Prefix {
    ListEvent,
    Events,
    SponserToSponse,
    ClientToEventId,
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
        };
        let mut list_event_id = HashSet::new();
        list_event_id.insert(event_id.clone());
        let client_event = ClientEvent {
            events: list_event_id,
        };
        self.list_event.insert(&event_id);
        self.client_to_event_id.insert(&owner, &client_event);
        self.events.insert(&event_id, &event);
        event
    }

    #[payable]
    pub fn sponse_native(&mut self, event_id: EventId, amount: U128) {
        if self.check_exist_event(&event_id) {
            assert_at_least_one_yocto();
            let amount: u128 = amount.into();
            let sender_id = env::predecessor_account_id();
            let attached_deposit = env::attached_deposit();
            let event = self.events.get(&event_id).unwrap();
            require!(
                event.iat <= env::block_timestamp(),
                "The event hasn't happened yet"
            );
            require!(event.exp >= env::block_timestamp(), "The event has ended");
            require!(
                attached_deposit == amount,
                "The attached_deposit must equal to the amount"
            );

            self.internal_deposit(&sender_id, &event_id, amount)
        } else {
            env::panic_str("EventId not exist");
        }
    }

    #[payable]
    pub fn more_sponse_native(&mut self, event_id: EventId, amount: U128) {
        if self.check_exist_event(&event_id) {
            assert_at_least_one_yocto();
            let amount: u128 = amount.into();
            let sender_id = env::predecessor_account_id();
            let attached_deposit = env::attached_deposit();
            let event = self.events.get(&event_id).unwrap();
            require!(
                event.iat <= env::block_timestamp(),
                "The event hasn't happened yet"
            );
            require!(event.exp >= env::block_timestamp(), "The event has ended");
            require!(
                attached_deposit == amount,
                "The attached_deposit must equal to the amount"
            );
            self.more_deposit(&sender_id, &event_id, amount)
        } else {
            env::panic_str("EventId not exist");
        }
    }

    #[payable]
    pub fn claim(&mut self, event_id: &EventId) {
        match self.events.get(event_id) {
            Some(res) => {
                if res.status == Status::Cancel {
                    assert_fee_storage_deposit();
                    let init_storage = env::storage_usage();
                    let receiver_id = env::signer_account_id();
                    match self.sponser_to_sponse.get(&receiver_id) {
                        Some(res) => {
                            let amount = res
                                .map_event_amount
                                .get(event_id)
                                .unwrap_or_else(|| env::panic_str("EventId is invalid"));
                            self.claim_token(receiver_id, *amount, event_id.clone());
                        }
                        None => {}
                    }
                    refund_deposit(init_storage);
                }
            }
            None => {
                env::panic_str("EventId not exist");
            }
        }
    }

    pub fn cancel_events(&mut self, event_id: EventId) {
        if self.check_exist_event(&event_id) {
            assert_at_least_one_yocto();
            let mut event = self.events.get(&event_id).unwrap();
            require!(
                event.iat <= env::block_timestamp(),
                "The event hasn't happened yet"
            );
            require!(event.exp >= env::block_timestamp(), "The event has ended");
            event.status = Status::Cancel;
            self.events.insert(&event_id, &event);
        } else {
            env::panic_str("EventId not exist");
        }
    }

    pub fn pause_delete_event(&mut self, event_id: &EventId) {
        match self.events.get(&event_id) {
            Some(mut res) => {
                res.pause = true;
                //update
                self.events.insert(event_id, &res);
            }
            None => env::panic_str("EventId is not a valid"),
        }
    }

    pub fn get_all_events(&self) -> Vec<(EventId, String)> {
        self.internal_get_all_events()
    }

    pub fn get_all_active_events(&self) -> Vec<(EventId, String)> {
        self.internal_get_all_active_events()
    }

    pub fn get_all_unactive_events(&self) -> Vec<(EventId, String)> {
        self.internal_get_all_unactive_events()
    }

    pub fn get_sponsed(&self) -> Vec<(EventId, String, Balance)> {
        self.internal_get_sponsed()
    }

    pub fn get_all_sponser_event(&self, event_id: EventId) -> Vec<AccountId> {
        self.internal_get_all_sponser_event(event_id)
    }

    pub fn get_total_token_event(&self, event_id: &EventId) -> Balance {
        self.internal_get_total_token_event(event_id)
    }
}
