use fuel_tx::*;
use fuel_tx_test_helpers::TransactionFactory;

use std::str::FromStr;

#[test]
fn to_from_str() {
    TransactionFactory::from_seed(1295)
        .take(20)
        .for_each(|(tx, _)| {
            let tx_p = tx.to_string();
            let tx_p = Transaction::from_str(&tx_p).expect("failed to restore tx");

            assert_eq!(tx, tx_p);
        });
}
