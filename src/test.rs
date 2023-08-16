#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use crate::Contract;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, AccountId, VMContext};

    fn get_context() -> VMContext {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(accounts(1))
            .predecessor_account_id(accounts(1))
            .is_view(false)
            .storage_usage(100000);
        builder.build()
    }
}
