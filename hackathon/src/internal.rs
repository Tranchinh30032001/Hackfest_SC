use std::collections::HashMap;

use near_sdk::Promise;

use crate::external::ext_self;
use crate::*;

impl Contract {
    pub(crate) fn internal_unwrap_balance(
        &self,
        account_id: &AccountId,
        event_id: &EventId,
    ) -> Result<Balance, String> {
        match self.sponser_to_sponse.get(account_id) {
            Some(sponse) => match sponse.map_event_amount.get(event_id) {
                Some(balance) => {
                    return Ok(*balance);
                }
                None => Err(String::from("Invalid amount")),
            },
            None => Err(String::from("EventId not found")),
        }
    }

    pub(crate) fn internal_sponse(
        &mut self,
        account_id: &AccountId,
        event_id: &EventId,
        amount: Balance,
    ) {
        match self.internal_unwrap_balance(account_id, event_id) {
            Ok(_) => env::panic_str("You have deposited this event before"),
            Err(_) => {
                let mut map_event_amount = HashMap::new();
                map_event_amount.insert(event_id.clone(), amount);

                match self.sponser_to_sponse.get(&account_id) {
                    Some(mut res) => {
                        res.events.insert(event_id.clone());
                        res.map_event_amount.insert(event_id.clone(), amount);
                        self.sponser_to_sponse.insert(&account_id, &res);
                    }
                    None => {
                        let mut events = HashSet::new();
                        events.insert(event_id.clone());
                        let sponse = Sponse {
                            events,
                            map_event_amount,
                        };
                        self.sponser_to_sponse.insert(account_id, &sponse);
                    }
                }

                let mut event = self.events.get(&event_id).unwrap();
                event.sponsers.push(account_id.clone());
                event.total += amount;
                self.events.insert(&event_id, &event);
            }
        }
    }

    pub(crate) fn internal_more_sponse(
        &mut self,
        account_id: &AccountId,
        event_id: &EventId,
        amount: Balance,
    ) {
        match self.internal_unwrap_balance(account_id, event_id) {
            Ok(balance) => {
                if let Some(new_balance) = balance.checked_add(amount) {
                    match self.sponser_to_sponse.get(account_id) {
                        Some(mut sponse) => {
                            // overwrite
                            sponse
                                .map_event_amount
                                .insert(event_id.clone(), new_balance);
                            self.sponser_to_sponse.insert(&account_id, &sponse);
                            let mut event = self.events.get(&event_id).unwrap();
                            event.total += amount;
                            self.events.insert(event_id, &event);
                        }
                        None => env::panic_str("You hasn't deposit this event yet"),
                    }
                }
            }
            Err(_) => env::panic_str("You haven't sponse this event before"),
        }
    }

    pub(crate) fn claim_token(&self, receiver_id: AccountId, amount: Balance, event_id: EventId) {
        // check transfer thanh cong roi moi update lai reward cung nhu balance owner.
        Promise::new(receiver_id.clone()).transfer(amount).then(
            ext_self::ext(env::current_account_id())
                .with_static_gas(FT_TRANSFER_GAS)
                .claim_token_callback(receiver_id, amount, event_id),
        );
    }

    pub(crate) fn internal_get_all_events(&self) -> Vec<(EventId, String)> {
        let arr_event = self.list_event.to_vec();
        require!(arr_event.len() > 0, "No record events");
        let result = arr_event
            .iter()
            .map(|item| {
                let name_event = self.events.get(item).unwrap().name;
                return (item.clone(), name_event);
            })
            .collect();
        result
    }

    pub(crate) fn internal_get_all_active_events(&self) -> Vec<(EventId, String)> {
        let arr_event = self.list_event.to_vec();
        require!(arr_event.len() > 0, "No record events active");
        let result = arr_event
            .iter()
            .filter(|item| {
                let event = self
                    .events
                    .get(*item)
                    .unwrap_or_else(|| env::panic_str("Not Valid"));
                return event.status == Status::Active;
            })
            .map(|item| {
                let name_event = self.events.get(item).unwrap().name;
                return (item.clone(), name_event);
            })
            .collect();
        return result;
    }

    pub(crate) fn internal_get_all_unactive_events(&self) -> Vec<(EventId, String)> {
        let arr_event = self.list_event.to_vec();
        require!(arr_event.len() > 0, "No record events un active");
        let result = arr_event
            .iter()
            .filter(|item| {
                let event = self
                    .events
                    .get(*item)
                    .unwrap_or_else(|| env::panic_str("Not Valid"));
                return event.status != Status::Active;
            })
            .map(|item| {
                let name_event = self.events.get(item).unwrap().name;
                return (item.clone(), name_event);
            })
            .collect();
        return result;
    }

    pub(crate) fn internal_get_all_event_client(
        &self,
        account_id: AccountId,
    ) -> Vec<(EventId, String)> {
        match self.client_to_event_id.get(&account_id) {
            Some(res) => {
                let arr_event: Vec<String> = res.events.into_iter().collect();
                let result = arr_event
                    .iter()
                    .map(|item| {
                        let name_event = self.events.get(item).unwrap().name;
                        return (item.clone(), name_event);
                    })
                    .collect();
                return result;
            }
            None => {
                env::panic_str("Client hasn't create any events yet");
            }
        }
    }

    pub(crate) fn internal_get_sponsed(&self) -> Vec<(EventId, String, Balance)> {
        let signer = env::signer_account_id();
        match self.sponser_to_sponse.get(&signer) {
            Some(res) => {
                let result = res
                    .events
                    .iter()
                    .map(|item| {
                        let name_event = self.events.get(item).unwrap().name;
                        let amount = res.map_event_amount.get(item).unwrap();
                        return (item.clone(), name_event, *amount);
                    })
                    .collect();
                result
            }
            None => env::panic_str("You have not deposited in any event ye"),
        }
    }

    pub(crate) fn internal_get_all_sponser_event(&self, event_id: EventId) -> Vec<AccountId> {
        match self.events.get(&event_id) {
            Some(res) => {
                return res.sponsers;
            }
            None => env::panic_str("EventId is not a valid"),
        }
    }

    pub(crate) fn internal_get_total_token_event(&self, event_id: &EventId) -> Balance {
        match self.events.get(&event_id) {
            Some(res) => {
                return res.total;
            }
            None => env::panic_str("EventId is not a valid"),
        }
    }
}
