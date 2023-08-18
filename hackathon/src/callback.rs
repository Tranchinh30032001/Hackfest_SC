use near_sdk::{env, near_bindgen, AccountId, Balance, PromiseResult};

use crate::*;
#[near_bindgen]
impl Contract {
    pub fn claim_token_callback(
        &mut self,
        receiver_id: AccountId,
        amount: Balance,
        event_id: EventId,
        token: Token,
    ) {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                //update total, list sponser of event, and map sponser_to_sponse
                match self.events.get(&event_id) {
                    Some(mut res) => {
                        if token == Token::NEAR {
                            res.total_near -= amount;
                        } else {
                            res.total_usdt -= amount;
                        }
                        res.sponsers = res
                            .sponsers
                            .iter()
                            .map(|item| item.clone())
                            .filter(|item| *item != receiver_id)
                            .collect();
                        self.events.insert(&event_id, &res);
                        self.handle_sponser_claim(receiver_id, event_id);
                    }
                    None => env::panic_str("EventId is not Found"),
                }
            }
            PromiseResult::Failed => {
                env::panic_str("user claim token failed");
            }
        }
    }

    pub(crate) fn handle_sponser_claim(&mut self, sponser_id: AccountId, event_id: EventId) {
        match self.sponser_to_sponse.get(&sponser_id) {
            Some(mut res) => {
                if res.events.len() == 1 {
                    self.sponser_to_sponse.remove(&sponser_id);
                } else {
                    res.events.remove(&event_id);
                    res.map_event_amount.remove(&event_id);
                    self.sponser_to_sponse.insert(&sponser_id, &res);
                }
            }
            None => env::panic_str("You haven't sponse this event before"),
        }
    }

    pub fn storage_deposit_callback_add_token(&mut self, token_id: AccountId) {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                //add whitelisted token of smart contract
                log!("storage deposit successfully");
            }
            PromiseResult::Failed => {
                env::panic_str("storage_deposit for owner failed");
            }
        }
    }
}
