use near_sdk::{env, near_bindgen, AccountId, Balance, PromiseResult};

use crate::*;
#[near_bindgen]
impl Contract {
    pub fn claim_token_callback(
        &mut self,
        receiver_id: AccountId,
        amount: Balance,
        event_id: EventId,
    ) {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                //update total, list sponser of event, and map sponser_to_sponse
                match self.events.get(&event_id) {
                    Some(mut res) => {
                        res.total -= amount;
                        res.sponsers = res
                            .sponsers
                            .iter()
                            .map(|item| item.clone())
                            .filter(|item| *item != receiver_id)
                            .collect();
                        self.events.insert(&event_id, &res);
                        self.sponser_to_sponse.remove(&receiver_id);
                    }
                    None => env::panic_str("EventId is not Found"),
                }
            }
            PromiseResult::Failed => {
                env::panic_str("user claim token failed");
            }
        }
    }
}
