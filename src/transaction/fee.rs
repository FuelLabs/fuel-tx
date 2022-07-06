use fuel_asm::{PanicReason, Word};

use crate::{ConsensusParameters, ValidationError};

#[cfg(feature = "std")]
use crate::Transaction;

#[cfg(feature = "std")]
use core::borrow::Borrow;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionFee {
    bytes: Word,
    total: Word,
}

impl From<TransactionFee> for Word {
    fn from(fee: TransactionFee) -> Word {
        fee.total()
    }
}

impl TransactionFee {
    pub const fn new(bytes: Word, total: Word) -> Self {
        Self { bytes, total }
    }

    /// Minimum fee value to pay for the metered bytes
    pub const fn min(&self) -> Word {
        self.bytes
    }

    /// Maximum fee value composed of metered bytes cost + tx gas limit, with price factor
    /// correction
    pub const fn total(&self) -> Word {
        self.total
    }

    /// Convert into a tuple containing the inner min & total fee values
    pub const fn into_inner(self) -> (Word, Word) {
        (self.bytes, self.total)
    }

    /// Attempt to subtract the maximum fee value from a given balance
    pub fn try_deduct_max(&self, balance: Word) -> Result<Word, ValidationError> {
        let fee = self.total();

        balance
            .checked_sub(fee)
            .ok_or(ValidationError::InsufficientFeeAmount {
                expected: fee,
                provided: balance,
            })
    }

    pub fn from_values(
        params: &ConsensusParameters,
        metered_bytes: Word,
        gas_limit: Word,
        gas_price: Word,
    ) -> Result<Self, PanicReason> {
        let factor = params.gas_price_factor as u128;

        // TODO: use native div_ceil once stabilized out from nightly
        let bytes = params.gas_per_byte.checked_mul(metered_bytes);
        let total = bytes
            .and_then(|bytes| bytes.checked_add(gas_limit))
            .and_then(|total| total.checked_mul(gas_price))
            .and_then(|total| num_integer::div_ceil(total as u128, factor).try_into().ok());

        let bytes = bytes
            .and_then(|bytes| bytes.checked_mul(gas_price))
            .and_then(|bytes| num_integer::div_ceil(bytes as u128, factor).try_into().ok());

        bytes
            .zip(total)
            .map(|(bytes, total)| Self::new(bytes, total))
            .ok_or(PanicReason::ArithmeticOverflow)
    }

    #[cfg(feature = "std")]
    pub fn from_tx<T>(params: &ConsensusParameters, tx: T) -> Result<Self, PanicReason>
    where
        T: Borrow<Transaction>,
    {
        let tx = tx.borrow();

        let metered_bytes = tx.metered_bytes_size() as Word;
        let gas_limit = tx.gas_limit();
        let gas_price = tx.gas_price();

        Self::from_values(params, metered_bytes, gas_limit, gas_price)
    }
}
