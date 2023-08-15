use std::collections::HashMap;

use crate::{external::ext_ft_fungible_token, *};

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

    pub(crate) fn internal_deposit(
        &mut self,
        account_id: &AccountId,
        event_id: &EventId,
        amount: Balance,
    ) {
        if self.check_exist_event(event_id) {
            match self.internal_unwrap_balance(account_id, event_id) {
                Ok(_) => {
                    todo!();
                }
                Err(_) => {
                    let mut map_event_amount = HashMap::new();
                    map_event_amount.insert(event_id.clone(), amount);
                    let mut events = HashSet::new();
                    events.insert(event_id.clone());
                    let sponse = Sponse {
                        events,
                        map_event_amount,
                    };
                    self.sponser_to_sponse.insert(account_id, &sponse);
                }
            }
        } else {
            env::panic_str("EventId not exist");
        }
    }

    pub(crate) fn update_deposit(
        &mut self,
        account_id: &AccountId,
        event_id: &EventId,
        amount: Balance,
    ) {
        if self.check_exist_event(event_id) {
            match self.internal_unwrap_balance(account_id, event_id) {
                Ok(balance) => {
                    if let Some(new_balance) = balance.checked_add(amount) {
                        match self.sponser_to_sponse.get(account_id) {
                            Some(mut sponse) => {
                                // overwrite
                                sponse
                                    .map_event_amount
                                    .insert(event_id.clone(), new_balance);
                            }
                            None => env::panic_str("You hasn't deposit this event yet"),
                        }
                    }
                }
                Err(err) => env::panic_str(&err),
            }
        } else {
            env::panic_str("EventId not exist");
        }
    }

    pub(crate) fn pause_delete_event(&mut self, event_id: &EventId) {
        match self.events.get(&event_id) {
            Some(mut res) => {
                res.pause = true;
                //update
                self.events.insert(event_id, &res);
            }
            None => env::panic_str("EventId is not a valid"),
        }
    }

    pub(crate) fn claim_token(&self, receiver_id: AccountId, amount: Balance, token_id: AccountId) {
        // check transfer thanh cong roi moi update lai reward cung nhu balance owner.
        ext_ft_fungible_token::ext(token_id.clone())
            .with_attached_deposit(1)
            .with_static_gas(FT_TRANSFER_GAS)
            .ft_transfer(receiver_id.clone(), amount.into(), None)
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(FT_TRANSFER_GAS)
                    // if success update reward and owner
                    .claim_token_callback(receiver_id, &token_id, amount),
            );
    }

    pub(crate) fn get_all_events(&self) -> Vec<(EventId, String)> {
        let arrEvent = self.list_event.to_vec();
        let result = arrEvent
            .iter()
            .map(|item| {
                let name_event = self.events.get(item).unwrap().name;
                return (item.clone(), name_event);
            })
            .collect();
        result
    }

    pub(crate) fn get_all_active_events(&self) -> Vec<(EventId, String)> {
        let arrEvent = self.list_event.to_vec();
        let result = arrEvent
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

    pub(crate) fn get_all_un_active_events(&self) -> Vec<(EventId, String)> {
        let arrEvent = self.list_event.to_vec();
        let result = arrEvent
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

    pub(crate) fn get_sponsed(&self) -> Vec<(EventId, String, Balance)> {
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

    pub(crate) fn get_all_sponser_event(&self, event_id: EventId) -> Vec<AccountId> {
        match self.events.get(&event_id) {
            Some(res) => {
                return res.sponsers;
            }
            None => env::panic_str("EventId is not a valid"),
        }
    }

    pub(crate) fn get_total_token_event(&self, event_id: &EventId) -> Balance {
        match self.events.get(&event_id) {
            Some(res) => {
                return res.total;
            }
            None => env::panic_str("EventId is not a valid"),
        }
    }
}
