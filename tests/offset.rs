use fuel_tx::bytes::{Deserializable, SerializableVec};
use fuel_tx::*;

use factory::TransactionFactory;

mod factory;

#[test]
fn iow_offset() {
    TransactionFactory::from_seed(3493)
        .take(100)
        .for_each(|mut tx| {
            let bytes = tx.to_bytes();

            tx.inputs().iter().enumerate().for_each(|(x, i)| {
                let offset = tx.input_offset(x);
                let input =
                    Input::from_bytes(&bytes[offset..]).expect("Failed to deserialize input!");

                assert_eq!(i, &input);
            });
        });
}
