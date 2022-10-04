use crate::{Input, Output, Transaction, TransactionError, Witness};

use fuel_asm::Word;
use itertools::Itertools;

use alloc::vec::Vec;
use core::hash::Hash;

// TODO https://github.com/FuelLabs/fuel-tx/issues/148
pub(crate) fn next_duplicate<U>(iter: impl Iterator<Item = U>) -> Option<U>
where
    U: PartialEq + Ord + Copy + Hash,
{
    #[cfg(not(feature = "std"))]
    return iter
        .sorted()
        .as_slice()
        .windows(2)
        .filter_map(|u| (u[0] == u[1]).then(|| u[0]))
        .next();

    #[cfg(feature = "std")]
    return iter.duplicates().next();
}

#[cfg(feature = "internals")]
impl Transaction {
    /// Append an input to the transaction. Panics otherwise.
    pub fn add_input(&mut self, input: Input) {
        self._add_input(input).expect("Couldn't add input")
    }

    /// Append an output to the transaction. Panics otherwise.
    pub fn add_output(&mut self, output: Output) {
        self._add_output(output)
    }

    /// Append a witness to the transaction. Panics otherwise.
    pub fn add_witness(&mut self, witness: Witness) {
        self._add_witness(witness).expect("Couldn't add witness")
    }

    /// Set the transaction script, if script variant. Panics otherwise.
    pub fn set_script(&mut self, script: Vec<u8>) {
        self._set_script(script).expect("Couldn't set script")
    }

    /// Set the transaction bytecode, if create variant. Panics otherwise.
    pub fn set_bytecode(&mut self, bytecode: Witness) {
        self._set_bytecode(bytecode).expect("Couldn't set bytecode")
    }

    pub fn inputs_mut(&mut self) -> &mut [Input] {
        self._inputs_mut().expect("Inputs is empty")
    }

    pub fn outputs_mut(&mut self) -> &mut [Output] {
        self._outputs_mut()
    }

    pub fn witnesses_mut(&mut self) -> &mut [Witness] {
        self._witnesses_mut().expect("Witnesses is empty")
    }
}

impl Transaction {
    pub(crate) fn _add_input(&mut self, input: Input) -> Result<(), TransactionError> {
        match self {
            Self::Script { inputs, .. } => Ok(inputs.push(input)),
            Self::Create { inputs, .. } => Ok(inputs.push(input)),
            Self::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }

    pub(crate) fn _add_output(&mut self, output: Output) {
        match self {
            Self::Script { outputs, .. } => outputs.push(output),
            Self::Create { outputs, .. } => outputs.push(output),
            Self::Mint { outputs, .. } => outputs.push(output),
        }
    }

    pub(crate) fn _add_witness(&mut self, witness: Witness) -> Result<(), TransactionError> {
        match self {
            Self::Script { witnesses, .. } => Ok(witnesses.push(witness)),
            Self::Create { witnesses, .. } => Ok(witnesses.push(witness)),
            Self::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }

    pub(crate) fn _set_script(&mut self, _script: Vec<u8>) -> Result<(), TransactionError> {
        match self {
            Self::Script { script, .. } => {
                *script = _script;
                Ok(())
            }
            Self::Create { .. } | Self::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }

    pub(crate) fn _set_bytecode(&mut self, bytecode: Witness) -> Result<(), TransactionError> {
        match self {
            Self::Create {
                bytecode_length,
                bytecode_witness_index,
                witnesses,
                ..
            } => {
                *bytecode_length = (bytecode.as_ref().len() / 4) as Word;
                *bytecode_witness_index = witnesses.len() as u8;

                witnesses.push(bytecode);

                Ok(())
            }
            Self::Script { .. } | Self::Mint { .. } => Err(TransactionError::FieldDoesNotExist),
        }
    }
}
