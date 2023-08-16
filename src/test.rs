#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use crate::event::Event;
    use crate::Contract;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, log, testing_env, AccountId, VMContext};

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

    #[test]
    fn test_create_event() {
        let mut context = get_context();
        context.attached_deposit = 1;
        context.signer_account_id = accounts(2);
        testing_env!(context);
        let mut contract = Contract::new(accounts(1));
        contract.create_event(String::from("001"), String::from("Panana"), 60000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);
        let event = Event {
            id: String::from("001"),
            owner: accounts(2),
            name: String::from("Panana"),
            iat: 0,
            exp: 60000,
            total: 0,
            status: crate::event::Status::Active,
            pause: false,
            sponsers: vec![],
        };
        let mut result = Vec::new();
        result.push((String::from("001"), String::from("Panana")));

        assert_eq!(contract.list_event.len(), 2); // test list_event

        assert_eq!(contract.events.get(&String::from("001")).unwrap(), event); //test events
    }

    #[test]
    fn test_get_all_event_client() {
        let mut context = get_context();
        context.attached_deposit = 1;
        context.signer_account_id = accounts(2);
        testing_env!(context);
        let mut contract = Contract::new(accounts(1));
        contract.create_event(String::from("001"), String::from("Panana"), 60000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);
        contract.create_event(String::from("006"), String::from("AHAHA6"), 60000);
        contract.create_event(String::from("005"), String::from("AHAHA5"), 60000);
        contract.create_event(String::from("007"), String::from("AHAHA5"), 60000);
        assert_eq!(contract.get_all_event_client().len(), 5);
    }

    #[test]
    fn test_get_all_event() {
        let mut context = get_context();
        context.attached_deposit = 1;
        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));
        contract.create_event(String::from("001"), String::from("Panana"), 60000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);

        context.signer_account_id = accounts(0);
        contract.create_event(String::from("003"), String::from("Panana3"), 60000);
        contract.create_event(String::from("004"), String::from("AHAHA4"), 60000);

        let mut result = Vec::new();
        result.push((String::from("001"), String::from("Panana")));
        result.push((String::from("002"), String::from("AHAHA")));
        result.push((String::from("003"), String::from("Panana3")));
        result.push((String::from("004"), String::from("AHAHA4")));

        assert_eq!(contract.get_all_events(), result);
    }

    #[test]
    fn test_sponse_native() {
        let mut context = get_context();
        context.attached_deposit = 5_000;
        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));

        contract.create_event(String::from("001"), String::from("Panana"), 60000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);
        contract.create_event(String::from("003"), String::from("AHAHA"), 60000);

        contract.sponse_native(String::from("001"), U128(5000));
        contract.sponse_native(String::from("002"), U128(5000));
        contract.sponse_native(String::from("003"), U128(5000));

        context.signer_account_id = accounts(0);
        testing_env!(context.clone());
        contract.sponse_native(String::from("001"), U128(5000));

        context.signer_account_id = accounts(1);
        testing_env!(context);
        contract.sponse_native(String::from("001"), U128(5000));

        // so sánh số lượng các event_id mà sponser tham gia.
        assert_eq!(
            contract
                .sponser_to_sponse
                .get(&accounts(1))
                .unwrap()
                .events
                .len(),
            1
        );
        // so sánh số lượng sponser trong 1 event cụ thể.
        assert_eq!(
            contract
                .events
                .get(&String::from("001"))
                .unwrap()
                .sponsers
                .len(),
            3
        );
    }

    #[test] // test ham get_sponsed
    fn test_get_sponsed() {
        let mut context = get_context();
        context.attached_deposit = 5_000;
        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));
        contract.create_event(String::from("001"), String::from("Panana"), 60000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);
        contract.create_event(String::from("003"), String::from("AHAHA"), 60000);
        contract.sponse_native(String::from("001"), U128(5000));
        contract.sponse_native(String::from("002"), U128(5000));
        contract.sponse_native(String::from("003"), U128(5000));

        assert_eq!(contract.get_sponsed().len(), 3);
    }

    #[test]
    fn test_get_all_sponser_event() {
        let mut context = get_context();
        context.attached_deposit = 5_000;

        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));
        contract.create_event(String::from("001"), String::from("Panana"), 60000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);
        contract.create_event(String::from("003"), String::from("AHAHA"), 60000);

        contract.sponse_native(String::from("001"), U128(5000));

        context.signer_account_id = accounts(0);
        testing_env!(context.clone());
        contract.sponse_native(String::from("001"), U128(5000));

        context.signer_account_id = accounts(1);
        testing_env!(context);
        contract.sponse_native(String::from("001"), U128(5000));

        assert_eq!(contract.get_all_sponser_event(String::from("001")).len(), 3);
    }

    #[test]
    fn test_get_total_token_event() {
        let mut context = get_context();
        context.attached_deposit = 5_000;

        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));
        contract.create_event(String::from("001"), String::from("Panana"), 60000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);
        contract.create_event(String::from("003"), String::from("AHAHA"), 60000);

        contract.sponse_native(String::from("001"), U128(5000));

        context.signer_account_id = accounts(0);
        testing_env!(context.clone());
        contract.sponse_native(String::from("001"), U128(5000));

        context.signer_account_id = accounts(1);
        testing_env!(context);
        contract.sponse_native(String::from("001"), U128(5000));

        assert_eq!(contract.get_total_token_event(&String::from("001")), 15000);
    }

    #[test]
    fn test_get_all_active_events() {
        let mut context = get_context();
        context.attached_deposit = 1;
        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));
        contract.create_event(String::from("001"), String::from("Panana"), 10000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 20000);
        contract.create_event(String::from("006"), String::from("AHAHA6"), 30000);
        contract.create_event(String::from("005"), String::from("AHAHA5"), 40000);
        contract.create_event(String::from("007"), String::from("AHAHA5"), 50000);

        context.block_timestamp = 35000;
        testing_env!(context);
        assert_eq!(contract.get_all_active_events().len(), 5);
    }

    #[test]
    fn test_get_all_unactive_events() {
        let mut context = get_context();
        context.attached_deposit = 1;
        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));
        contract.create_event(String::from("001"), String::from("Panana"), 10000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 20000);
        contract.create_event(String::from("006"), String::from("AHAHA6"), 30000);
        contract.create_event(String::from("005"), String::from("AHAHA5"), 40000);
        contract.create_event(String::from("007"), String::from("AHAHA5"), 50000);

        contract.cancel_events(String::from("001"));
        testing_env!(context);
        assert_eq!(contract.get_all_unactive_events().len(), 1);
    }

    #[test]
    fn test_more_sponse_native() {
        let mut context = get_context();
        context.attached_deposit = 5_000;
        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));

        contract.create_event(String::from("001"), String::from("Panana"), 60000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);
        contract.create_event(String::from("003"), String::from("AHAHA"), 60000);

        contract.sponse_native(String::from("001"), U128(5000));
        // contract.sponse_native(String::from("002"), U128(5000));
        // contract.sponse_native(String::from("003"), U128(5000));

        context.attached_deposit = 20000;
        testing_env!(context.clone());
        contract.more_sponse_native(String::from("001"), U128(20000));
        contract.more_sponse_native(String::from("001"), U128(20000));
        contract.more_sponse_native(String::from("001"), U128(20000));

        assert_eq!(
            contract
                .sponser_to_sponse
                .get(&accounts(2))
                .unwrap()
                .map_event_amount
                .get(&String::from("001"))
                .unwrap()
                .clone(),
            65000
        );
    }

    #[test]
    #[should_panic(expected = "The event has ended")]
    fn test_more_sponse_native_overtime() {
        let mut context = get_context();
        context.attached_deposit = 5_000;
        context.signer_account_id = accounts(2);
        testing_env!(context.clone());
        let mut contract = Contract::new(accounts(1));

        contract.create_event(String::from("001"), String::from("Panana"), 10000);
        contract.create_event(String::from("002"), String::from("AHAHA"), 60000);

        contract.sponse_native(String::from("001"), U128(5000));
        contract.sponse_native(String::from("002"), U128(5000));
        // contract.sponse_native(String::from("003"), U128(5000));

        context.attached_deposit = 20000;
        context.block_timestamp = 30000;
        testing_env!(context.clone());
        contract.more_sponse_native(String::from("001"), U128(20000));

        assert_eq!(
            contract
                .sponser_to_sponse
                .get(&accounts(2))
                .unwrap()
                .map_event_amount
                .get(&String::from("001"))
                .unwrap()
                .clone(),
            25000
        );
    }
}
