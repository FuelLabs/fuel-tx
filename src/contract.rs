use crate::{Transaction, ValidationError};

#[cfg(feature = "std")]
use crate::StorageSlot;

use fuel_crypto::Hasher;
use fuel_types::{Bytes32, ContractId, Salt};

#[cfg(feature = "std")]
use fuel_merkle::{binary, common::StorageMap, sparse};

#[cfg(feature = "std")]
use fuel_types::Bytes8;

use alloc::vec::Vec;

#[cfg(feature = "std")]
use core::iter;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Deployable representation of a contract code.
pub struct Contract(Vec<u8>);

impl Contract {
    #[cfg(feature = "std")]
    /// Calculate the code root of the contract, using [`Self::root_from_code`].
    pub fn root(&self) -> Bytes32 {
        Self::root_from_code(self)
    }

    #[cfg(feature = "std")]
    /// Calculate the code root from a contract.
    ///
    /// <https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/identifiers.md#contract-id>
    pub fn root_from_code<B>(bytes: B) -> Bytes32
    where
        B: AsRef<[u8]>,
    {
        let mut storage = StorageMap::new();
        let mut tree = binary::MerkleTree::new(&mut storage);

        bytes
            .as_ref()
            .chunks(Bytes8::LEN)
            .map(|c| {
                if c.len() == Bytes8::LEN {
                    // Safety: checked len chunk
                    unsafe { Bytes8::from_slice_unchecked(c) }
                } else {
                    // Potential collision with non-padded input. Consider adding an extra leaf
                    // for padding?
                    let mut b = [0u8; 8];

                    let l = c.len();
                    (&mut b[..l]).copy_from_slice(c);

                    b.into()
                }
            })
            .try_for_each(|l| tree.push(l.as_ref()))
            .and_then(|_| tree.root())
            .expect("In-memory impl should be infallible")
            .into()
    }

    #[cfg(feature = "std")]
    /// Calculate the root of the initial storage slots for this contract
    pub fn initial_state_root<'a, I>(mut storage_slots: I) -> Bytes32
    where
        I: Iterator<Item = &'a StorageSlot>,
    {
        let mut storage = StorageMap::new();
        let mut tree = sparse::MerkleTree::new(&mut storage);

        storage_slots
            .try_for_each(|s| tree.update(s.key(), s.value().as_ref()))
            .expect("In-memory impl should be infallible");

        tree.root().into()
    }

    #[cfg(feature = "std")]
    /// The default state root value without any entries
    pub fn default_state_root() -> Bytes32 {
        Self::initial_state_root(iter::empty())
    }

    /// Calculate and return the contract id, provided a salt, code root and state root.
    ///
    /// <https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/identifiers.md#contract-id>
    pub fn id(&self, salt: &Salt, root: &Bytes32, state_root: &Bytes32) -> ContractId {
        let mut hasher = Hasher::default();

        hasher.input(ContractId::SEED);
        hasher.input(salt);
        hasher.input(root);
        hasher.input(state_root);

        ContractId::from(*hasher.digest())
    }
}

impl From<Vec<u8>> for Contract {
    fn from(c: Vec<u8>) -> Self {
        Self(c)
    }
}

impl From<&[u8]> for Contract {
    fn from(c: &[u8]) -> Self {
        Self(c.into())
    }
}

impl From<&mut [u8]> for Contract {
    fn from(c: &mut [u8]) -> Self {
        Self(c.into())
    }
}

impl From<Contract> for Vec<u8> {
    fn from(c: Contract) -> Vec<u8> {
        c.0
    }
}

impl AsRef<[u8]> for Contract {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for Contract {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl TryFrom<&Transaction> for Contract {
    type Error = ValidationError;

    fn try_from(tx: &Transaction) -> Result<Self, Self::Error> {
        match tx {
            Transaction::Create {
                bytecode_witness_index,
                witnesses,
                ..
            } => witnesses
                .get(*bytecode_witness_index as usize)
                .map(|c| c.as_ref().into())
                .ok_or(ValidationError::TransactionCreateBytecodeWitnessIndex),

            _ => Err(ValidationError::TransactionScriptOutputContractCreated { index: 0 }),
        }
    }
}
