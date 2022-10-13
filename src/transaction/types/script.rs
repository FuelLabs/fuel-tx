use crate::transaction::field::{
    GasLimit, GasPrice, Inputs, Maturity, Outputs, ReceiptsRoot, Script as ScriptField, ScriptData,
    Witnesses,
};
use crate::transaction::validation::{validate_common_part, Validatable};
use crate::transaction::Chargeable;
use crate::{Cacheable, ConsensusParameters, Input, Output, ValidationError, Witness};
use derivative::Derivative;
use fuel_types::bytes::{SizedBytes, WORD_SIZE};
use fuel_types::{bytes, Bytes32, Word};

#[cfg(feature = "std")]
use std::io;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use fuel_types::bytes::SerializableVec;

#[derive(Default, Debug, Clone, Derivative)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derivative(Eq, PartialEq, Hash)]
pub struct Script {
    pub(crate) gas_price: Word,
    pub(crate) gas_limit: Word,
    pub(crate) maturity: Word,
    pub(crate) script: Vec<u8>,
    pub(crate) script_data: Vec<u8>,
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,
    pub(crate) witnesses: Vec<Witness>,
    pub(crate) receipts_root: Bytes32,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    // TODO: Add metadata
    pub(crate) metadata: Option<()>,
}

#[cfg(feature = "std")]
impl crate::UniqueIdentifier for Script {
    fn id(&self) -> Bytes32 {
        // TODO: Add metadata
        let mut clone = self.clone();

        // Empties fields that should be zero during the signing.
        *clone.receipts_root_mut() = Default::default();
        clone.inputs_mut().iter_mut().for_each(Input::prepare_sign);
        clone
            .outputs_mut()
            .iter_mut()
            .for_each(Output::prepare_sign);
        clone.witnesses_mut().clear();

        fuel_crypto::Hasher::hash(clone.to_bytes().as_slice())
    }
}

impl Chargeable for Script {
    #[inline(always)]
    fn metered_bytes_size(&self) -> usize {
        // Just use the default serialized size for now until
        // the compressed representation for accounting purposes
        // is defined. Witness data should still be excluded.
        self.witnesses_offset()
    }
}

impl Validatable for Script {
    #[cfg(feature = "std")]
    fn validate_signatures(&self) -> Result<(), ValidationError> {
        use crate::UniqueIdentifier;

        let id = self.id();

        self.inputs()
            .iter()
            .enumerate()
            .try_for_each(|(index, input)| input.validate_signature(index, &id, &self.witnesses))?;

        Ok(())
    }

    fn validate_without_signatures(
        &self,
        block_height: Word,
        parameters: &ConsensusParameters,
    ) -> Result<(), ValidationError> {
        validate_common_part(self, block_height, parameters)?;

        if self.script.len() > parameters.max_script_length as usize {
            Err(ValidationError::TransactionScriptLength)?;
        }

        if self.script_data.len() > parameters.max_script_data_length as usize {
            Err(ValidationError::TransactionScriptDataLength)?;
        }

        self.outputs
            .iter()
            .enumerate()
            .try_for_each(|(index, output)| match output {
                Output::ContractCreated { .. } => {
                    Err(ValidationError::TransactionScriptOutputContractCreated { index })
                }
                _ => Ok(()),
            })?;

        Ok(())
    }
}

impl Cacheable for Script {
    fn is_computed(&self) -> bool {
        self.metadata.is_some()
    }

    fn precompute(&mut self) {
        // TODO: Add metadata
    }
}

impl SizedBytes for Script {
    fn serialized_size(&self) -> usize {
        // TODO: Add metadata
        self.witnesses_offset()
            + self
                .witnesses()
                .iter()
                .map(|w| w.serialized_size())
                .sum::<usize>()
    }
}

impl GasPrice for Script {
    #[inline(always)]
    fn gas_price(&self) -> &Word {
        &self.gas_price
    }

    #[inline(always)]
    fn gas_price_mut(&mut self) -> &mut Word {
        &mut self.gas_price
    }

    #[inline(always)]
    fn gas_price_offset(&self) -> usize {
        // Before `Script` transaction should be `TransactionRepr`, but it is handled by the
        // `Transaction` type itself.
        //
        // #Note : If you need offset from `Transaction`, it should be `Transaction::offset()` + `Script::*_offset`.
        0
    }
}

impl GasLimit for Script {
    #[inline(always)]
    fn gas_limit(&self) -> &Word {
        &self.gas_limit
    }

    #[inline(always)]
    fn gas_limit_mut(&mut self) -> &mut Word {
        &mut self.gas_limit
    }

    #[inline(always)]
    fn gas_limit_offset(&self) -> usize {
        self.gas_price_offset() + WORD_SIZE
    }
}

impl Maturity for Script {
    #[inline(always)]
    fn maturity(&self) -> &Word {
        &self.maturity
    }

    #[inline(always)]
    fn maturity_mut(&mut self) -> &mut Word {
        &mut self.maturity
    }

    #[inline(always)]
    fn maturity_offset(&self) -> usize {
        self.gas_limit_offset() + WORD_SIZE
    }
}

impl ReceiptsRoot for Script {
    #[inline(always)]
    fn receipts_root(&self) -> &Bytes32 {
        &self.receipts_root
    }

    #[inline(always)]
    fn receipts_root_mut(&mut self) -> &mut Bytes32 {
        &mut self.receipts_root
    }

    #[inline(always)]
    fn receipts_root_offset(&self) -> usize {
        self.maturity_offset() + WORD_SIZE
            + WORD_SIZE // Script size
            + WORD_SIZE // Script data size
            + WORD_SIZE // Inputs size
            + WORD_SIZE // Outputs size
            + WORD_SIZE // Witnesses size
    }
}

impl ScriptField for Script {
    #[inline(always)]
    fn script(&self) -> &Vec<u8> {
        &self.script
    }

    #[inline(always)]
    fn script_mut(&mut self) -> &mut Vec<u8> {
        &mut self.script
    }

    #[inline(always)]
    fn script_offset(&self) -> usize {
        self.receipts_root_offset() + Bytes32::LEN // Receipts root
    }
}

impl ScriptData for Script {
    #[inline(always)]
    fn script_data(&self) -> &Vec<u8> {
        &self.script_data
    }

    #[inline(always)]
    fn script_data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.script_data
    }

    #[inline(always)]
    fn script_data_offset(&self) -> usize {
        // TODO: Add metadata
        self.script_offset() + bytes::padded_len(self.script.as_slice())
    }
}

impl Inputs for Script {
    #[inline(always)]
    fn inputs(&self) -> &Vec<Input> {
        &self.inputs
    }

    #[inline(always)]
    fn inputs_mut(&mut self) -> &mut Vec<Input> {
        &mut self.inputs
    }

    #[inline(always)]
    fn inputs_offset(&self) -> usize {
        // TODO: Add metadata
        self.script_data_offset() + bytes::padded_len(self.script_data.as_slice())
    }

    #[inline(always)]
    fn inputs_offset_at(&self, idx: usize) -> Option<usize> {
        // TODO: Add metadata
        if idx < self.inputs.len() {
            Some(
                self.inputs_offset()
                    + self
                        .inputs()
                        .iter()
                        .take(idx)
                        .map(|i| i.serialized_size())
                        .sum::<usize>(),
            )
        } else {
            None
        }
    }

    #[inline(always)]
    fn inputs_predicate_offset_at(&self, idx: usize) -> Option<(usize, usize)> {
        self.inputs().get(idx).and_then(|input| {
            input
                .predicate_offset()
                .and_then(|predicate| self.inputs_offset_at(idx).map(|inputs| inputs + predicate))
                .zip(input.predicate_len().map(bytes::padded_len_usize))
        })
    }
}

impl Outputs for Script {
    #[inline(always)]
    fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }

    #[inline(always)]
    fn outputs_mut(&mut self) -> &mut Vec<Output> {
        &mut self.outputs
    }

    #[inline(always)]
    fn outputs_offset(&self) -> usize {
        // TODO: Add metadata
        self.inputs_offset()
            + self
                .inputs()
                .iter()
                .map(|i| i.serialized_size())
                .sum::<usize>()
    }

    #[inline(always)]
    fn outputs_offset_at(&self, idx: usize) -> Option<usize> {
        // TODO: Add metadata
        if idx < self.outputs.len() {
            Some(
                self.outputs_offset()
                    + self
                        .outputs()
                        .iter()
                        .take(idx)
                        .map(|i| i.serialized_size())
                        .sum::<usize>(),
            )
        } else {
            None
        }
    }
}

impl Witnesses for Script {
    #[inline(always)]
    fn witnesses(&self) -> &Vec<Witness> {
        &self.witnesses
    }

    #[inline(always)]
    fn witnesses_mut(&mut self) -> &mut Vec<Witness> {
        &mut self.witnesses
    }

    #[inline(always)]
    fn witnesses_offset(&self) -> usize {
        // TODO: Add metadata
        self.outputs_offset()
            + self
                .outputs()
                .iter()
                .map(|i| i.serialized_size())
                .sum::<usize>()
    }

    #[inline(always)]
    fn witnesses_offset_at(&self, idx: usize) -> Option<usize> {
        // TODO: Add metadata
        if idx < self.witnesses.len() {
            Some(
                self.witnesses_offset()
                    + self
                        .witnesses()
                        .iter()
                        .take(idx)
                        .map(|i| i.serialized_size())
                        .sum::<usize>(),
            )
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
impl io::Read for Script {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.serialized_size();
        if buf.len() < n {
            return Err(bytes::eof());
        }

        let Script {
            gas_price,
            gas_limit,
            maturity,
            receipts_root,
            script,
            script_data,
            inputs,
            outputs,
            witnesses,
            ..
        } = self;

        let mut buf = {
            let buf = bytes::store_number_unchecked(buf, *gas_price);
            let buf = bytes::store_number_unchecked(buf, *gas_limit);
            let buf = bytes::store_number_unchecked(buf, *maturity);
            let buf = bytes::store_number_unchecked(buf, script.len() as Word);
            let buf = bytes::store_number_unchecked(buf, script_data.len() as Word);
            let buf = bytes::store_number_unchecked(buf, inputs.len() as Word);
            let buf = bytes::store_number_unchecked(buf, outputs.len() as Word);
            let buf = bytes::store_number_unchecked(buf, witnesses.len() as Word);
            let buf = bytes::store_array_unchecked(buf, receipts_root);

            let (_, buf) = bytes::store_raw_bytes(buf, script.as_slice())?;
            let (_, buf) = bytes::store_raw_bytes(buf, script_data.as_slice())?;

            buf
        };

        for input in self.inputs.iter_mut() {
            let input_len = input.read(buf)?;
            buf = &mut buf[input_len..];
        }

        for output in self.outputs.iter_mut() {
            let output_len = output.read(buf)?;
            buf = &mut buf[output_len..];
        }

        for witness in self.witnesses.iter_mut() {
            let witness_len = witness.read(buf)?;
            buf = &mut buf[witness_len..];
        }

        Ok(n)
    }
}

#[cfg(feature = "std")]
impl io::Write for Script {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut n = crate::consts::TRANSACTION_SCRIPT_FIXED_SIZE - WORD_SIZE;
        if buf.len() < n {
            return Err(bytes::eof());
        }

        // Safety: buffer size is checked
        let (gas_price, buf) = unsafe { bytes::restore_number_unchecked(buf) };
        let (gas_limit, buf) = unsafe { bytes::restore_number_unchecked(buf) };
        let (maturity, buf) = unsafe { bytes::restore_number_unchecked(buf) };
        let (script_len, buf) = unsafe { bytes::restore_usize_unchecked(buf) };
        let (script_data_len, buf) = unsafe { bytes::restore_usize_unchecked(buf) };
        let (inputs_len, buf) = unsafe { bytes::restore_usize_unchecked(buf) };
        let (outputs_len, buf) = unsafe { bytes::restore_usize_unchecked(buf) };
        let (witnesses_len, buf) = unsafe { bytes::restore_usize_unchecked(buf) };
        let (receipts_root, buf) = unsafe { bytes::restore_array_unchecked(buf) };

        let receipts_root = receipts_root.into();

        let (size, script, buf) = bytes::restore_raw_bytes(buf, script_len)?;
        n += size;

        let (size, script_data, mut buf) = bytes::restore_raw_bytes(buf, script_data_len)?;
        n += size;

        let mut inputs = vec![Input::default(); inputs_len];
        for input in inputs.iter_mut() {
            let input_len = input.write(buf)?;
            buf = &buf[input_len..];
            n += input_len;
        }

        let mut outputs = vec![Output::default(); outputs_len];
        for output in outputs.iter_mut() {
            let output_len = output.write(buf)?;
            buf = &buf[output_len..];
            n += output_len;
        }

        let mut witnesses = vec![Witness::default(); witnesses_len];
        for witness in witnesses.iter_mut() {
            let witness_len = witness.write(buf)?;
            buf = &buf[witness_len..];
            n += witness_len;
        }

        *self = Script {
            gas_price,
            gas_limit,
            maturity,
            receipts_root,
            script,
            script_data,
            inputs,
            outputs,
            witnesses,
            metadata: None,
        };

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inputs.iter_mut().try_for_each(|input| input.flush())?;
        self.outputs
            .iter_mut()
            .try_for_each(|output| output.flush())?;
        self.witnesses
            .iter_mut()
            .try_for_each(|witness| witness.flush())?;

        Ok(())
    }
}
