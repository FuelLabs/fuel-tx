//! A checked transaction is type-wrapper for transactions which have been validated.
//! It is impossible to construct a checked transaction without performing necessary validation.
//!
//! This allows the VM to accept transactions that have been already verified upstream,
//! and consolidates logic around fee calculations and free balances.

use crate::ValidationError::InsufficientFeeAmount;
use crate::{ConsensusParameters, Input, Output, Transaction, ValidationError};
use alloc::collections::BTreeMap;
use fuel_types::{AssetId, Word};

pub struct CheckedTransaction {
    /// The transaction that was validated
    transaction: Transaction,
    /// The mapping of initial free balances
    initial_free_balances: BTreeMap<AssetId, Word>,
    /// The block height this tx was verified with
    block_height: Word,
    /// Max potential fee
    max_fee: Word,
    /// Min guaranteed fee
    min_fee: Word,
}

impl CheckedTransaction {
    pub fn check(
        transaction: Transaction,
        block_height: Word,
        params: &ConsensusParameters,
    ) -> Result<Self, ValidationError> {
        // fully validate transaction (with signature)
        transaction.validate(block_height, params)?;
        // validate fees and compute free balances
        let AvailableBalances {
            free_balances: initial_free_balances,
            max_fee,
            min_fee,
        } = Self::_initial_free_balances(&transaction, params)?;

        Ok(CheckedTransaction {
            transaction,
            initial_free_balances,
            block_height,
            max_fee,
            min_fee,
        })
    }

    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    pub fn free_balances(&self) -> impl Iterator<Item = (&AssetId, &Word)> {
        self.initial_free_balances.iter()
    }

    pub fn block_height(&self) -> Word {
        self.block_height
    }

    pub fn max_fee(&self) -> Word {
        self.max_fee
    }

    pub fn min_fee(&self) -> Word {
        self.min_fee
    }

    fn _initial_free_balances(
        transaction: &Transaction,
        params: &ConsensusParameters,
    ) -> Result<AvailableBalances, ValidationError> {
        let mut balances = BTreeMap::<AssetId, Word>::new();

        // Add up all the inputs for each asset ID
        for (asset_id, amount) in transaction.inputs().iter().filter_map(|input| match input {
            Input::CoinPredicate {
                asset_id, amount, ..
            }
            | Input::CoinSigned {
                asset_id, amount, ..
            } => Some((asset_id, amount)),
            _ => None,
        }) {
            *balances.entry(*asset_id).or_default() += amount;
        }

        // Reduce by unavailable balances

        // compute fee deductions

        // calculate the fee with used metered bytes + gas limit
        let factor = params.gas_price_factor as f64;

        let bytes = transaction
            .byte_price()
            .checked_mul(transaction.metered_bytes_size() as Word)
            .ok_or(ValidationError::ArithmeticOverflow)?;

        let bytes = (bytes as f64 / factor).ceil() as Word;

        let gas = transaction
            .gas_price()
            .checked_mul(transaction.gas_limit())
            .ok_or(ValidationError::ArithmeticOverflow)? as f64;
        let gas = (gas / factor).ceil() as Word;

        let fee = bytes
            .checked_add(gas)
            .ok_or(ValidationError::ArithmeticOverflow)?;

        let base_asset = AssetId::default();
        let base_asset_balance = balances.get_mut(&base_asset).ok_or(InsufficientFeeAmount {
            expected: fee,
            provided: 0,
        })?;
        // subtract total fee from base asset balance
        *base_asset_balance = base_asset_balance
            .checked_sub(fee)
            .ok_or(InsufficientFeeAmount {
                expected: fee,
                provided: *base_asset_balance,
            })?;

        // reduce free balances by coin outputs
        for (asset_id, amount) in transaction
            .outputs()
            .iter()
            .filter_map(|output| match output {
                Output::Coin {
                    asset_id, amount, ..
                } => Some((asset_id, amount)),
                _ => None,
            })
        {
            let balance = balances.get_mut(asset_id).ok_or(
                ValidationError::TransactionOutputCoinAssetIdNotFound(*asset_id),
            )?;
            *balance =
                balance
                    .checked_sub(*amount)
                    .ok_or(ValidationError::InsufficientInputAmount {
                        asset: *asset_id,
                        expected: *amount,
                        provided: *balance,
                    })?;
        }

        Ok(AvailableBalances {
            free_balances: balances,
            max_fee: fee,
            min_fee: bytes,
        })
    }
}

struct AvailableBalances {
    free_balances: BTreeMap<AssetId, Word>,
    max_fee: Word,
    min_fee: Word,
}

impl AsRef<Transaction> for CheckedTransaction {
    fn as_ref(&self) -> &Transaction {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionBuilder;
    use crate::ValidationError::InsufficientInputAmount;
    use fuel_crypto::SecretKey;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn checked_tx_accepts_valid_tx() {
        // simple smoke test that valid txs can be checked
        let rng = &mut StdRng::seed_from_u64(2322u64);
        let tx = valid_tx(rng);

        let checked = CheckedTransaction::check(tx.clone(), 0, &ConsensusParameters::DEFAULT)
            .expect("Expected valid transaction");

        assert_eq!(checked.transaction(), &tx);
    }

    #[test]
    fn max_fee() {
        // verify max fee a transaction can consume based on gas limit is correct
        let rng = &mut StdRng::seed_from_u64(2322u64);
        let tx = valid_tx(rng);

        let checked = CheckedTransaction::check(tx.clone(), 0, &ConsensusParameters::DEFAULT)
            .expect("Expected valid transaction");

        let factor = ConsensusParameters::DEFAULT.gas_price_factor as f64;
        let bytes =
            f64::ceil(tx.metered_bytes_size() as f64 * tx.byte_price() as f64 / factor) as u64;
        let gas = f64::ceil(tx.gas_limit() as f64 * tx.gas_price() as f64 / factor) as u64;

        assert_eq!(bytes + gas, checked.max_fee);
    }

    #[test]
    fn min_fee() {
        // verify max fee a transaction can consume based on gas limit is correct
        let rng = &mut StdRng::seed_from_u64(2322u64);
        let tx = valid_tx(rng);

        let checked = CheckedTransaction::check(tx.clone(), 0, &ConsensusParameters::DEFAULT)
            .expect("Expected valid transaction");

        let factor = ConsensusParameters::DEFAULT.gas_price_factor as f64;
        let bytes =
            f64::ceil(tx.metered_bytes_size() as f64 * tx.byte_price() as f64 / factor) as u64;

        assert_eq!(bytes, checked.min_fee);
    }

    #[test]
    fn checked_tx_rejects_invalid_tx() {
        // simple smoke test that invalid txs cannot be checked
        let rng = &mut StdRng::seed_from_u64(2322u64);
        let asset = rng.gen();
        let tx = TransactionBuilder::script(vec![], vec![])
            .gas_price(1)
            .gas_limit(100)
            .add_input(Input::coin_signed(
                rng.gen(),
                rng.gen(),
                1_000,
                rng.gen(),
                0,
                0,
            ))
            .add_input(Input::contract(rng.gen(), rng.gen(), rng.gen(), rng.gen()))
            .add_output(Output::contract(1, rng.gen(), rng.gen()))
            .add_output(Output::coin(rng.gen(), 10, asset))
            .add_output(Output::change(rng.gen(), 0, asset))
            .finalize();

        let checked = CheckedTransaction::check(tx.clone(), 0, &ConsensusParameters::DEFAULT)
            .err()
            .expect("Expected invalid transaction");

        // assert that tx without a valid signature fails
        assert_eq!(
            ValidationError::InputCoinWitnessIndexBounds { index: 0 },
            checked
        );
    }

    #[test]
    fn checked_tx_fails_when_provided_fees_dont_cover_byte_costs() {
        let rng = &mut StdRng::seed_from_u64(2322u64);

        let input_amount = 100;
        let gas_price = 0;
        let factor = 1;

        let params = ConsensusParameters::default().with_gas_price_factor(factor);

        // make byte price too high for the input amount
        let byte_price = factor;

        let secret_key = SecretKey::random(rng);
        let asset = AssetId::default();

        let transaction = TransactionBuilder::script(vec![], vec![])
            .gas_price(gas_price)
            .byte_price(byte_price)
            .add_unsigned_coin_input(rng.gen(), &secret_key, input_amount, asset, 0)
            .add_output(Output::change(rng.gen(), 0, asset))
            .finalize();

        let err = CheckedTransaction::check(transaction, 0, &params)
            .err()
            .expect("insufficient fee amount expected");

        let provided = match err {
            ValidationError::InsufficientFeeAmount { provided, .. } => provided,
            _ => panic!("expected insufficient fee amount; found {:?}", err),
        };

        assert_eq!(provided, input_amount);
    }

    #[test]
    fn checked_tx_fails_when_provided_fees_dont_cover_gas_costs() {
        let rng = &mut StdRng::seed_from_u64(2322u64);

        let input_amount = 10;
        let factor = 1;

        let params = ConsensusParameters::default().with_gas_price_factor(factor);

        // make gas price too high for the input amount
        let gas_price = factor;
        let gas_limit = input_amount + 1;
        let byte_price = 0;
        let secret_key = SecretKey::random(rng);

        let transaction = TransactionBuilder::script(vec![], vec![])
            .gas_price(gas_price)
            .gas_limit(gas_limit)
            .byte_price(byte_price)
            .add_unsigned_coin_input(rng.gen(), &secret_key, input_amount, AssetId::default(), 0)
            .add_output(Output::change(rng.gen(), 0, AssetId::default()))
            .finalize();

        let err = CheckedTransaction::check(transaction, 0, &params)
            .err()
            .expect("insufficient fee amount expected");

        let provided = match err {
            InsufficientFeeAmount { provided, .. } => provided,
            _ => panic!("expected insufficient fee amount; found {:?}", err),
        };

        assert_eq!(provided, input_amount);
    }

    #[test]
    fn bytes_fee_cant_overflow() {
        let rng = &mut StdRng::seed_from_u64(2322u64);

        let input_amount = 1000;
        let gas_price = 0;
        // make byte price too high for the input amount
        let byte_price = Word::MAX;
        let secret_key = SecretKey::random(rng);

        let params = ConsensusParameters::default().with_gas_price_factor(1);

        let transaction = TransactionBuilder::script(vec![], vec![])
            .gas_price(gas_price)
            .byte_price(byte_price)
            .add_unsigned_coin_input(rng.gen(), &secret_key, input_amount, AssetId::default(), 0)
            .add_output(Output::change(rng.gen(), 0, AssetId::default()))
            .finalize();

        let err = CheckedTransaction::check(transaction, 0, &params)
            .err()
            .expect("overflow expected");

        assert_eq!(err, ValidationError::ArithmeticOverflow.into());
    }

    #[test]
    fn gas_fee_cant_overflow() {
        let rng = &mut StdRng::seed_from_u64(2322u64);
        let input_amount = 1000;
        let gas_price = Word::MAX;
        let gas_limit = 2;
        // make byte price too high for the input amount
        let byte_price = 0;
        let secret_key = SecretKey::random(rng);

        let params = ConsensusParameters::default().with_gas_price_factor(1);

        let transaction = TransactionBuilder::script(vec![], vec![])
            .gas_price(gas_price)
            .gas_limit(gas_limit)
            .byte_price(byte_price)
            .add_unsigned_coin_input(rng.gen(), &secret_key, input_amount, AssetId::default(), 0)
            .add_output(Output::change(rng.gen(), 0, AssetId::default()))
            .finalize();

        let err = CheckedTransaction::check(transaction, 0, &params)
            .err()
            .expect("overflow expected");

        assert_eq!(err, ValidationError::ArithmeticOverflow.into());
    }

    #[test]
    fn total_fee_cant_overflow() {
        let rng = &mut StdRng::seed_from_u64(2322u64);
        // ensure that total fee can't overflow as a result of adding the gas fee and byte fee
        let input_amount = 1000;

        let gas_price = Word::MAX;
        let gas_limit = 1;

        // make byte price too high for the input amount
        let byte_price = Word::MAX;
        let secret_key = SecretKey::random(rng);

        let params = ConsensusParameters::default().with_gas_price_factor(1);

        let transaction = TransactionBuilder::script(vec![], vec![])
            .gas_price(gas_price)
            .gas_limit(gas_limit)
            .byte_price(byte_price)
            .add_unsigned_coin_input(rng.gen(), &secret_key, input_amount, AssetId::default(), 0)
            .add_output(Output::change(rng.gen(), 0, AssetId::default()))
            .finalize();

        let err = CheckedTransaction::check(transaction, 0, &params)
            .err()
            .expect("overflow expected");

        assert_eq!(err, ValidationError::ArithmeticOverflow.into());
    }

    #[test]
    fn check_tx_fails_if_asset_is_overspent_by_coin_output() {
        let input_amount = 1_000;
        let rng = &mut StdRng::seed_from_u64(2322u64);
        let secret = SecretKey::random(rng);
        let any_asset = rng.gen();
        let tx = TransactionBuilder::script(vec![], vec![])
            .gas_price(1)
            .gas_limit(100)
            // base asset
            .add_unsigned_coin_input(rng.gen(), &secret, input_amount, AssetId::default(), 0)
            .add_output(Output::change(rng.gen(), 0, AssetId::default()))
            // arbitrary spending asset
            .add_unsigned_coin_input(rng.gen(), &secret, input_amount, any_asset, 0)
            .add_output(Output::coin(rng.gen(), input_amount + 1, any_asset))
            .add_output(Output::change(rng.gen(), 0, any_asset))
            .finalize();

        let checked = CheckedTransaction::check(tx.clone(), 0, &ConsensusParameters::DEFAULT)
            .err()
            .expect("Expected valid transaction");

        assert_eq!(
            InsufficientInputAmount {
                asset: any_asset,
                expected: input_amount + 1,
                provided: input_amount
            },
            checked
        );
    }

    fn valid_tx(rng: &mut StdRng) -> Transaction {
        let asset = AssetId::default();
        TransactionBuilder::script(vec![], vec![])
            .gas_price(1)
            .gas_limit(100)
            .add_unsigned_coin_input(rng.gen(), &rng.gen(), 1_000, asset, 0)
            .add_input(Input::contract(rng.gen(), rng.gen(), rng.gen(), rng.gen()))
            .add_output(Output::contract(1, rng.gen(), rng.gen()))
            .add_output(Output::coin(rng.gen(), 10, asset))
            .add_output(Output::change(rng.gen(), 0, asset))
            .finalize()
    }
}
