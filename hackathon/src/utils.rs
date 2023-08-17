use crate::*;
use near_sdk::Promise;

pub(crate) fn assert_at_least_one_yocto() {
    assert!(
        env::attached_deposit() >= 1,
        "Required attached deposit of at least 1 yoctoNEAR"
    )
}

// pub(crate) fn assert_fee_storage_deposit() {
//     assert!(
//         env::attached_deposit() >= parse_near!("0.00125 N"),
//         "You have to deposit greater or equa to 0.00125 Near"
//     )
// }

pub(crate) fn refund_deposit(init_storage: u64) {
    let finals_storage = env::storage_usage();
    let required_cost = env::storage_byte_cost() * Balance::from(finals_storage - init_storage);
    let attached_deposit = env::attached_deposit();

    assert!(
        attached_deposit >= required_cost,
        "Must attach yoctoNear to cover {} storage",
        required_cost
    );

    let refund = attached_deposit - required_cost;

    if refund > 0 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}
