use fuel_types::Word;

use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ScriptResultRepr {
    Success = 0x00,
    Revert = 0x01,
    OutOfGas = 0x02,
    TransactionValidity = 0x03,
    MemoryOverflow = 0x04,
    ArithmeticOverflow = 0x05,
    ContractNotFound = 0x06,
    MemoryOwnership = 0x07,
    NotEnoughBalance = 0x08,
    ExpectedInternalContext = 0x09,
    ColorNotFound = 0x0a,
    InputNotFound = 0x0b,
    OutputNotFound = 0x0c,
    WitnessNotFound = 0x0d,
    TransactionMaturity = 0x0e,
    InvalidMetadataIdentifier = 0x0f,
    MalformedCallStructure = 0x10,
    ReservedRegisterNotWritable = 0x11,
    ErrorFlag = 0x12,
    InvalidImmediateValue = 0x13,
    ExpectedCoinInput = 0x14,
    MaxMemoryAccess = 0x15,
    MemoryWriteOverlap = 0x16,
    ContractNotInInputs = 0x17,
    InternalBalanceOverflow = 0x18,
    ContractMaxSize = 0x19,
    ExpectedUnallocatedStack = 0x1a,
    MaxStaticContractsReached = 0x1b,
    TransferAmountCannotBeZero = 0x1c,
    ExpectedOutputVariable = 0x1d,
    ExpectedParentInternalContext = 0x1e,
}

impl TryFrom<Word> for ScriptResultRepr {
    type Error = io::Error;

    fn try_from(b: Word) -> Result<Self, Self::Error> {
        match b {
            0x00 => Ok(Self::Success),
            0x01 => Ok(Self::Revert),
            0x02 => Ok(Self::OutOfGas),
            0x03 => Ok(Self::TransactionValidity),
            0x04 => Ok(Self::MemoryOverflow),
            0x05 => Ok(Self::ArithmeticOverflow),
            0x06 => Ok(Self::ContractNotFound),
            0x07 => Ok(Self::MemoryOwnership),
            0x08 => Ok(Self::NotEnoughBalance),
            0x09 => Ok(Self::ExpectedInternalContext),
            0x0a => Ok(Self::ColorNotFound),
            0x0b => Ok(Self::InputNotFound),
            0x0c => Ok(Self::OutputNotFound),
            0x0d => Ok(Self::WitnessNotFound),
            0x0e => Ok(Self::TransactionMaturity),
            0x0f => Ok(Self::InvalidMetadataIdentifier),
            0x10 => Ok(Self::MalformedCallStructure),
            0x11 => Ok(Self::ReservedRegisterNotWritable),
            0x12 => Ok(Self::ErrorFlag),
            0x13 => Ok(Self::InvalidImmediateValue),
            0x14 => Ok(Self::ExpectedCoinInput),
            0x15 => Ok(Self::MaxMemoryAccess),
            0x16 => Ok(Self::MemoryWriteOverlap),
            0x17 => Ok(Self::ContractNotInInputs),
            0x18 => Ok(Self::InternalBalanceOverflow),
            0x19 => Ok(Self::ContractMaxSize),
            0x1a => Ok(Self::ExpectedUnallocatedStack),
            0x1b => Ok(Self::MaxStaticContractsReached),
            0x1c => Ok(Self::TransferAmountCannotBeZero),
            0x1d => Ok(Self::ExpectedOutputVariable),
            0x1e => Ok(Self::ExpectedParentInternalContext),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The provided identifier is invalid!",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-types", derive(serde::Serialize, serde::Deserialize))]
pub enum ScriptResult {
    Success,
    Revert,
    OutOfGas,
    TransactionValidity,
    MemoryOverflow(Word),
    ArithmeticOverflow,
    ContractNotFound,
    MemoryOwnership(Word),
    NotEnoughBalance,
    ExpectedInternalContext,
    ColorNotFound,
    InputNotFound(Word),
    OutputNotFound(Word),
    WitnessNotFound(Word),
    TransactionMaturity(Word),
    InvalidMetadataIdentifier,
    MalformedCallStructure,
    ReservedRegisterNotWritable(Word),
    ErrorFlag,
    InvalidImmediateValue,
    ExpectedCoinInput(Word),
    MaxMemoryAccess(Word),
    MemoryWriteOverlap(Word),
    ContractNotInInputs(Word),
    InternalBalanceOverflow(Word),
    ContractMaxSize(Word),
    ExpectedUnallocatedStack,
    MaxStaticContractsReached(Word),
    TransferAmountCannotBeZero,
    ExpectedOutputVariable(Word),
    ExpectedParentInternalContext,
}

impl From<ScriptResult> for Word {
    fn from(r: ScriptResult) -> Word {
        match r {
            ScriptResult::Success => (ScriptResultRepr::Success as Word) << 56,
            ScriptResult::Revert => (ScriptResultRepr::Revert as Word) << 56,
            ScriptResult::OutOfGas => (ScriptResultRepr::OutOfGas as Word) << 56,
            ScriptResult::TransactionValidity => {
                (ScriptResultRepr::TransactionValidity as Word) << 56
            }
            ScriptResult::MemoryOverflow(val) => {
                ((ScriptResultRepr::MemoryOverflow as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::ArithmeticOverflow => {
                (ScriptResultRepr::ArithmeticOverflow as Word) << 56
            }
            ScriptResult::ContractNotFound => ((ScriptResultRepr::ContractNotFound as Word) << 56),
            ScriptResult::MemoryOwnership(val) => {
                ((ScriptResultRepr::MemoryOwnership as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::NotEnoughBalance => ((ScriptResultRepr::NotEnoughBalance as Word) << 56),
            ScriptResult::ExpectedInternalContext => {
                (ScriptResultRepr::ExpectedInternalContext as Word) << 56
            }
            ScriptResult::ColorNotFound => ((ScriptResultRepr::ColorNotFound as Word) << 56),
            ScriptResult::InputNotFound(val) => {
                ((ScriptResultRepr::InputNotFound as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::OutputNotFound(val) => {
                ((ScriptResultRepr::OutputNotFound as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::WitnessNotFound(val) => {
                ((ScriptResultRepr::WitnessNotFound as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::TransactionMaturity(val) => {
                ((ScriptResultRepr::TransactionMaturity as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::InvalidMetadataIdentifier => {
                (ScriptResultRepr::InvalidMetadataIdentifier as Word) << 56
            }
            ScriptResult::MalformedCallStructure => {
                (ScriptResultRepr::MalformedCallStructure as Word) << 56
            }
            ScriptResult::ReservedRegisterNotWritable(val) => {
                ((ScriptResultRepr::ReservedRegisterNotWritable as Word) << 56)
                    | (val & 0xffffffffffffff)
            }
            ScriptResult::ErrorFlag => ((ScriptResultRepr::ErrorFlag as Word) << 56),
            ScriptResult::InvalidImmediateValue => {
                (ScriptResultRepr::InvalidImmediateValue as Word) << 56
            }
            ScriptResult::ExpectedCoinInput(val) => {
                ((ScriptResultRepr::ExpectedCoinInput as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::MaxMemoryAccess(val) => {
                ((ScriptResultRepr::MaxMemoryAccess as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::MemoryWriteOverlap(val) => {
                ((ScriptResultRepr::MemoryWriteOverlap as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::ContractNotInInputs(val) => {
                ((ScriptResultRepr::ContractNotInInputs as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::InternalBalanceOverflow(val) => {
                ((ScriptResultRepr::InternalBalanceOverflow as Word) << 56)
                    | (val & 0xffffffffffffff)
            }
            ScriptResult::ContractMaxSize(val) => {
                ((ScriptResultRepr::ContractMaxSize as Word) << 56) | (val & 0xffffffffffffff)
            }
            ScriptResult::ExpectedUnallocatedStack => {
                (ScriptResultRepr::ExpectedUnallocatedStack as Word) << 56
            }
            ScriptResult::MaxStaticContractsReached(val) => {
                ((ScriptResultRepr::MaxStaticContractsReached as Word) << 56)
                    | (val & 0xffffffffffffff)
            }
            ScriptResult::TransferAmountCannotBeZero => {
                (ScriptResultRepr::TransferAmountCannotBeZero as Word) << 56
            }
            ScriptResult::ExpectedOutputVariable(val) => {
                ((ScriptResultRepr::ExpectedOutputVariable as Word) << 56)
                    | (val & 0xffffffffffffff)
            }
            ScriptResult::ExpectedParentInternalContext => {
                (ScriptResultRepr::ExpectedParentInternalContext as Word) << 56
            }
        }
    }
}

impl TryFrom<Word> for ScriptResult {
    type Error = io::Error;

    fn try_from(val: Word) -> Result<Self, Self::Error> {
        let repr = val >> 56;
        let repr = ScriptResultRepr::try_from(repr)?;
        let val = val & 0xffffffffffffff;

        match repr {
            ScriptResultRepr::Success => Ok(Self::Success),
            ScriptResultRepr::Revert => Ok(Self::Revert),
            ScriptResultRepr::OutOfGas => Ok(Self::OutOfGas),
            ScriptResultRepr::TransactionValidity => Ok(Self::TransactionValidity),
            ScriptResultRepr::MemoryOverflow => Ok(Self::MemoryOverflow(val)),
            ScriptResultRepr::ArithmeticOverflow => Ok(Self::ArithmeticOverflow),
            ScriptResultRepr::ContractNotFound => Ok(Self::ContractNotFound),
            ScriptResultRepr::MemoryOwnership => Ok(Self::MemoryOwnership(val)),
            ScriptResultRepr::NotEnoughBalance => Ok(Self::NotEnoughBalance),
            ScriptResultRepr::ExpectedInternalContext => Ok(Self::ExpectedInternalContext),
            ScriptResultRepr::ColorNotFound => Ok(Self::ColorNotFound),
            ScriptResultRepr::InputNotFound => Ok(Self::InputNotFound(val)),
            ScriptResultRepr::OutputNotFound => Ok(Self::OutputNotFound(val)),
            ScriptResultRepr::WitnessNotFound => Ok(Self::WitnessNotFound(val)),
            ScriptResultRepr::TransactionMaturity => Ok(Self::TransactionMaturity(val)),
            ScriptResultRepr::InvalidMetadataIdentifier => Ok(Self::InvalidMetadataIdentifier),
            ScriptResultRepr::MalformedCallStructure => Ok(Self::MalformedCallStructure),
            ScriptResultRepr::ReservedRegisterNotWritable => {
                Ok(Self::ReservedRegisterNotWritable(val))
            }
            ScriptResultRepr::ErrorFlag => Ok(Self::ErrorFlag),
            ScriptResultRepr::InvalidImmediateValue => Ok(Self::InvalidImmediateValue),
            ScriptResultRepr::ExpectedCoinInput => Ok(Self::ExpectedCoinInput(val)),
            ScriptResultRepr::MaxMemoryAccess => Ok(Self::MaxMemoryAccess(val)),
            ScriptResultRepr::MemoryWriteOverlap => Ok(Self::MemoryWriteOverlap(val)),
            ScriptResultRepr::ContractNotInInputs => Ok(Self::ContractNotInInputs(val)),
            ScriptResultRepr::InternalBalanceOverflow => Ok(Self::InternalBalanceOverflow(val)),
            ScriptResultRepr::ContractMaxSize => Ok(Self::ContractMaxSize(val)),
            ScriptResultRepr::ExpectedUnallocatedStack => Ok(Self::ExpectedUnallocatedStack),
            ScriptResultRepr::MaxStaticContractsReached => Ok(Self::MaxStaticContractsReached(val)),
            ScriptResultRepr::TransferAmountCannotBeZero => Ok(Self::TransferAmountCannotBeZero),
            ScriptResultRepr::ExpectedOutputVariable => Ok(Self::ExpectedOutputVariable(val)),
            ScriptResultRepr::ExpectedParentInternalContext => {
                Ok(Self::ExpectedParentInternalContext)
            }
        }
    }
}
