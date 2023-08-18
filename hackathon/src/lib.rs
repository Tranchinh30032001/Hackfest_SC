use external::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault,
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
    pub fn active_usdt(&mut self) {
        let attached_deposit = env::attached_deposit();
        assert_fee_storage_deposit();
        ext_ft_storage::ext("usdt.fakes.testnet".parse().unwrap())
            .with_attached_deposit(attached_deposit)
            .with_static_gas(FT_TRANSFER_GAS)
            .storage_deposit(Some(env::current_account_id()), None)
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(FT_TRANSFER_GAS)
                    .storage_deposit_callback_add_token("usdt.fakes.testnet".parse().unwrap()),
            );
    }

    #[payable]
    pub fn create_event(&mut self, event_id: String, name_event: String) -> Event {
        assert_at_least_one_yocto();
        let owner = env::signer_account_id();
        let event = Event {
            id: event_id.clone(),
            owner: owner.clone(),
            name: name_event.clone(),
            total_near: 0,
            total_usdt: 0,
            status: Status::Active,
            sponsers: vec![],
        };
        match self.client_to_event_id.get(&owner) {
            Some(mut res) => {
                res.events.insert(event_id.clone());
                self.client_to_event_id.insert(&owner, &res);
            }
            None => {
                let mut list_event_id = HashSet::new();
                list_event_id.insert(event_id.clone());
                let client_event = ClientEvent {
                    events: list_event_id,
                };
                self.client_to_event_id.insert(&owner, &client_event);
            }
        }

        self.list_event.insert(&event_id);
        self.events.insert(&event_id, &event);
        event
    }

    #[payable]
    pub fn sponse_native(&mut self, event_id: EventId, amount: U128) {
        if self.check_exist_event(&event_id) {
            assert_at_least_one_yocto();
            let amount: u128 = amount.into();
            let sender_id = env::signer_account_id();
            let attached_deposit = env::attached_deposit();
            require!(
                attached_deposit == amount,
                "The attached_deposit must equal to the amount"
            );
            self.internal_sponse(&sender_id, &event_id, amount, Token::NEAR);
        } else {
            env::panic_str("EventId not exist");
        }
    }

    //cần kiểm tra xem, user có đủ số lượng usdt để sponse hay không.
    //và khi họ sponse thì có cần gửi kèm near để làm fee hay không.
    #[payable]
    pub fn sponse_usdt(&mut self, event_id: EventId, amount: U128) {
        if self.check_exist_event(&event_id) {
            assert_at_least_one_yocto();
            let amount: u128 = amount.into();
            let sender_id = env::signer_account_id();
            self.internal_sponse(&sender_id, &event_id, amount, Token::USDT);
        } else {
            env::panic_str("EventId not exist");
        }
    }

    #[payable]
    pub fn more_sponse_native(&mut self, event_id: EventId, amount: U128) {
        if self.check_exist_event(&event_id) {
            assert_at_least_one_yocto();
            let amount: u128 = amount.into();
            let sender_id = env::signer_account_id();
            let attached_deposit = env::attached_deposit();
            require!(
                attached_deposit == amount,
                "The attached_deposit must equal to the amount"
            );
            self.internal_more_sponse(&sender_id, &event_id, amount, Token::NEAR);
        } else {
            env::panic_str("EventId not exist");
        }
    }

    #[payable]
    pub fn more_sponse_usdt(&mut self, event_id: EventId, amount: U128) {
        if self.check_exist_event(&event_id) {
            assert_at_least_one_yocto();
            let amount: u128 = amount.into();
            let sender_id = env::signer_account_id();
            self.internal_more_sponse(&sender_id, &event_id, amount, Token::USDT);
        } else {
            env::panic_str("EventId not exist");
        }
    }

    pub fn finish_event(&mut self, event_id: EventId) {
        if self.check_exist_event(&event_id) {
            if env::signer_account_id() == self.owner_id {
                match self.events.get(&event_id) {
                    Some(mut res) => {
                        res.status = Status::Finish;
                        self.events.insert(&event_id, &res);
                    }
                    None => {
                        env::panic_str("Error");
                    }
                }
            }
        } else {
            env::panic_str("EventId not exist");
        }
    }

    #[payable]
    pub fn claim(&mut self, event_id: &EventId) {
        match self.events.get(event_id) {
            Some(res) => {
                if res.status == Status::Cancel {
                    assert_at_least_one_yocto();
                    let init_storage = env::storage_usage();
                    let receiver_id = env::signer_account_id();
                    match self.internal_unwrap_balance(&receiver_id, event_id) {
                        Ok(amount) => {
                            self.claim_token_near(
                                &receiver_id,
                                amount.token_near,
                                event_id.clone(),
                            );
                            self.claim_token_usdt(
                                &receiver_id,
                                amount.token_usdt,
                                event_id.clone(),
                            );
                        }
                        Err(_) => env::panic_str("You havn't sponse this event yet"),
                    }
                    refund_deposit(init_storage);
                } else {
                    env::panic_str("This event has not been canceled so you cannot withdraw token");
                }
            }
            None => {
                env::panic_str("EventId not exist");
            }
        }
    }
    #[payable]
    pub fn cancel_events(&mut self, event_id: EventId) {
        if self.check_exist_event(&event_id) {
            assert_at_least_one_yocto();
            require!(
                self.check_owner_event(&event_id),
                "You are not allowed to cancel"
            );
            let mut event = self.events.get(&event_id).unwrap();
            event.status = Status::Cancel;
            self.events.insert(&event_id, &event);
        } else {
            env::panic_str("EventId not exist");
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

    // trả về tất cả các event mà 1 client đã tạo.
    pub fn get_all_event_client(&self) -> Vec<(EventId, String)> {
        let account_id = env::signer_account_id();
        let result = self.internal_get_all_event_client(account_id);
        result
    }

    // hàm này trả về 1 vector tuple gồm event_id, name_event, và balance mà sponser đã sponse.
    pub fn get_sponsed(&self) -> Vec<(EventId, String, Amount)> {
        self.internal_get_sponsed()
    }

    // hàm này trả về danh sách các sponser đã sponse cho 1 event cụ thể.
    pub fn get_all_sponser_event(&self, event_id: EventId) -> Vec<AccountId> {
        self.internal_get_all_sponser_event(event_id)
    }

    // trả về số lượng token mà các sponser đã sponse vào 1 event cụ thể.
    pub fn get_total_token_event(&self, event_id: &EventId) -> Amount {
        self.internal_get_total_token_event(event_id)
    }

    pub fn watch_detail_event(&self, event_id: &EventId) -> Event {
        self.internal_watch_detail_event(event_id)
    }
}
