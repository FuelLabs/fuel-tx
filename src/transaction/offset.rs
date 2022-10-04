use super::{TRANSACTION_CREATE_FIXED_SIZE, TRANSACTION_SCRIPT_FIXED_SIZE};
use crate::{StorageSlot, Transaction, TransactionError};

use fuel_types::bytes::{self, SizedBytes, WORD_SIZE};
use fuel_types::Bytes32;

impl Transaction {
    /// For a serialized transaction of type `Script`, return the bytes offset
    /// of the script
    pub const fn script_offset(&self) -> Result<usize, TransactionError> {
        match self {
            Self::Script { .. } => Ok(TRANSACTION_SCRIPT_FIXED_SIZE),
            Self::Create { .. } | Self::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }

    /// For a serialized transaction of type `Script`, return the bytes offset
    /// of the script data
    pub fn script_data_offset(&self) -> Result<usize, TransactionError> {
        self.metadata()
            .map(|m| {
                m.script_data_offset()
                    .ok_or(TransactionError::FieldDoesNotExist)
            })
            .unwrap_or_else(|| self._script_data_offset())
    }

    pub(crate) fn _script_data_offset(&self) -> Result<usize, TransactionError> {
        match self {
            Self::Script { script, .. } => {
                Ok(TRANSACTION_SCRIPT_FIXED_SIZE + bytes::padded_len(script.as_slice()))
            }
            Self::Create { .. } | Self::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }

    /// Return the offset/length tuple for an input predicate indexed by `index`.
    ///
    /// Return `None` either if `index` is invalid, or if its not a [`Input::CoinPredicate`]
    pub fn input_predicate_offset(&self, index: usize) -> Option<(usize, usize)> {
        self.metadata()
            .map(|m| m.input_predicate_offset(index))
            .unwrap_or_else(|| self._input_predicate_offset(index))
    }

    pub(crate) fn _input_predicate_offset(&self, index: usize) -> Option<(usize, usize)> {
        if let Ok(inputs) = self.inputs() {
            inputs.get(index).and_then(|i| {
                i.predicate_offset()
                    .and_then(|p| self.input_offset(index).map(|o| p + o))
                    .zip(i.predicate_len().map(bytes::padded_len_usize))
            })
        } else {
            None
        }
    }

    /// Return the serialized bytes offset of the input with the provided index
    ///
    /// Return `None` if `index` is invalid
    pub fn input_offset(&self, index: usize) -> Option<usize> {
        self.metadata()
            .map(|m| m.inputs_offset(index))
            .unwrap_or_else(|| self._input_offset(index))
    }

    pub(crate) fn inputs_offset(&self) -> Result<usize, TransactionError> {
        match self {
            Transaction::Script {
                script,
                script_data,
                ..
            } => Ok(TRANSACTION_SCRIPT_FIXED_SIZE
                + bytes::padded_len(script.as_slice())
                + bytes::padded_len(script_data.as_slice())),

            Transaction::Create { storage_slots, .. } => {
                Ok(TRANSACTION_CREATE_FIXED_SIZE + StorageSlot::SLOT_SIZE * storage_slots.len())
            }
            Transaction::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }

    pub(crate) fn _input_offset(&self, index: usize) -> Option<usize> {
        match (self.inputs_offset(), self.inputs()) {
            (Ok(offset), Ok(inputs)) => inputs.get(index).map(|_| {
                inputs
                    .iter()
                    .take(index)
                    .map(|i| i.serialized_size())
                    .sum::<usize>()
                    + offset
            }),
            _ => None,
        }
    }

    /// Return the serialized bytes offset of the output with the provided index
    ///
    /// Return `None` if `index` is invalid
    pub fn output_offset(&self, index: usize) -> Option<usize> {
        self.metadata()
            .map(|m| m.outputs_offset(index))
            .unwrap_or_else(|| self._output_offset(index))
    }

    pub(crate) fn outputs_offset(&self) -> usize {
        match self {
            Transaction::Script {
                script,
                script_data,
                inputs,
                ..
            } => {
                TRANSACTION_SCRIPT_FIXED_SIZE
                    + bytes::padded_len(script.as_slice())
                    + bytes::padded_len(script_data.as_slice())
                    + inputs.iter().map(|i| i.serialized_size()).sum::<usize>()
            }

            Transaction::Create {
                storage_slots,
                inputs,
                ..
            } => {
                TRANSACTION_CREATE_FIXED_SIZE
                    + StorageSlot::SLOT_SIZE * storage_slots.len()
                    + inputs.iter().map(|i| i.serialized_size()).sum::<usize>()
            }
            Transaction::Mint { .. } => {
                WORD_SIZE // Identifier
                + WORD_SIZE // Output Size
            }
        }
    }

    pub(crate) fn _output_offset(&self, index: usize) -> Option<usize> {
        let offset = self.outputs_offset();
        self.outputs().iter().nth(index).map(|_| {
            self.outputs()
                .iter()
                .take(index)
                .map(|i| i.serialized_size())
                .sum::<usize>()
                + offset
        })
    }

    /// Return the serialized bytes offset of the witness with the provided
    /// index
    ///
    /// Return `None` if `index` is invalid
    pub fn witness_offset(&self, index: usize) -> Option<usize> {
        self.metadata()
            .map(|m| m.witnesses_offset(index))
            .unwrap_or_else(|| self._witness_offset(index))
    }

    pub(crate) fn witnesses_offset(&self) -> Result<usize, TransactionError> {
        match self {
            Transaction::Script {
                script,
                script_data,
                inputs,
                outputs,
                ..
            } => Ok(TRANSACTION_SCRIPT_FIXED_SIZE
                + bytes::padded_len(script.as_slice())
                + bytes::padded_len(script_data.as_slice())
                + inputs.iter().map(|i| i.serialized_size()).sum::<usize>()
                + outputs.iter().map(|o| o.serialized_size()).sum::<usize>()),

            Transaction::Create {
                storage_slots,
                inputs,
                outputs,
                ..
            } => Ok(TRANSACTION_CREATE_FIXED_SIZE
                + StorageSlot::SLOT_SIZE * storage_slots.len()
                + inputs.iter().map(|i| i.serialized_size()).sum::<usize>()
                + outputs.iter().map(|o| o.serialized_size()).sum::<usize>()),
            Transaction::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }

    pub(crate) fn _witness_offset(&self, index: usize) -> Option<usize> {
        match (self.witnesses_offset(), self.witnesses()) {
            (Ok(offset), Ok(witnesses)) => witnesses.get(index).map(|_| {
                witnesses
                    .iter()
                    .take(index)
                    .map(|i| i.serialized_size())
                    .sum::<usize>()
                    + offset
            }),
            _ => None,
        }
    }

    /// For a serialized transaction of type `Script`, return the bytes offset
    /// of the receipts root
    pub const fn receipts_root_offset(&self) -> Result<usize, TransactionError> {
        match self {
            Self::Script { .. } => Ok(TRANSACTION_SCRIPT_FIXED_SIZE - Bytes32::LEN),
            Self::Create { .. } | Self::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }
}
