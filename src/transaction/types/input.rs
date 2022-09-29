use crate::canonical::{Deserialize, Error, Output, Serialize};
use crate::{TxPointer, UtxoId};

use alloc::{vec, vec::Vec};
use consts::*;
use fuel_crypto::{Hasher, PublicKey};
use fuel_types::bytes;
use fuel_types::{Address, AssetId, Bytes32, ContractId, MessageId, Word};

use core::mem;

mod consts;
mod repr;

pub use repr::{InputRepr, InputSpec};

/// User-friendly interpretation of the [`InputSpec`](InputSpec).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Input {
    CoinSigned {
        utxo_id: UtxoId,
        owner: Address,
        amount: Word,
        asset_id: AssetId,
        tx_pointer: TxPointer,
        witness_index: u8,
        maturity: Word,
    },

    CoinPredicate {
        utxo_id: UtxoId,
        owner: Address,
        amount: Word,
        asset_id: AssetId,
        tx_pointer: TxPointer,
        maturity: Word,
        predicate: Vec<u8>,
        predicate_data: Vec<u8>,
    },

    Contract {
        utxo_id: UtxoId,
        balance_root: Bytes32,
        state_root: Bytes32,
        tx_pointer: TxPointer,
        contract_id: ContractId,
    },

    MessageSigned {
        message_id: MessageId,
        sender: Address,
        recipient: Address,
        amount: Word,
        nonce: Word,
        witness_index: u8,
        data: Vec<u8>,
    },

    MessagePredicate {
        message_id: MessageId,
        sender: Address,
        recipient: Address,
        amount: Word,
        nonce: Word,
        data: Vec<u8>,
        predicate: Vec<u8>,
        predicate_data: Vec<u8>,
    },
}

impl Default for Input {
    fn default() -> Self {
        Self::Contract {
            utxo_id: Default::default(),
            balance_root: Default::default(),
            state_root: Default::default(),
            tx_pointer: Default::default(),
            contract_id: Default::default(),
        }
    }
}

impl bytes::SizedBytes for Input {
    fn serialized_size(&self) -> usize {
        match self {
            Self::CoinSigned { .. } => INPUT_COIN_FIXED_SIZE,

            Self::CoinPredicate {
                predicate,
                predicate_data,
                ..
            } => {
                INPUT_COIN_FIXED_SIZE
                    + bytes::padded_len(predicate.as_slice())
                    + bytes::padded_len(predicate_data.as_slice())
            }

            Self::Contract { .. } => INPUT_CONTRACT_SIZE,

            Self::MessageSigned { data, .. } => {
                INPUT_MESSAGE_FIXED_SIZE + bytes::padded_len(data.as_slice())
            }

            Self::MessagePredicate {
                data,
                predicate,
                predicate_data,
                ..
            } => {
                INPUT_MESSAGE_FIXED_SIZE
                    + bytes::padded_len(data.as_slice())
                    + bytes::padded_len(predicate.as_slice())
                    + bytes::padded_len(predicate_data.as_slice())
            }
        }
    }
}

impl Input {
    pub const fn repr(&self) -> InputRepr {
        InputRepr::from_input(self)
    }

    pub fn owner(pk: &PublicKey) -> Address {
        let owner: [u8; Address::LEN] = pk.hash().into();

        owner.into()
    }

    pub const fn coin_predicate(
        utxo_id: UtxoId,
        owner: Address,
        amount: Word,
        asset_id: AssetId,
        tx_pointer: TxPointer,
        maturity: Word,
        predicate: Vec<u8>,
        predicate_data: Vec<u8>,
    ) -> Self {
        Self::CoinPredicate {
            utxo_id,
            owner,
            amount,
            asset_id,
            tx_pointer,
            maturity,
            predicate,
            predicate_data,
        }
    }

    pub const fn coin_signed(
        utxo_id: UtxoId,
        owner: Address,
        amount: Word,
        asset_id: AssetId,
        tx_pointer: TxPointer,
        witness_index: u8,
        maturity: Word,
    ) -> Self {
        Self::CoinSigned {
            utxo_id,
            owner,
            amount,
            asset_id,
            tx_pointer,
            witness_index,
            maturity,
        }
    }

    pub const fn contract(
        utxo_id: UtxoId,
        balance_root: Bytes32,
        state_root: Bytes32,
        tx_pointer: TxPointer,
        contract_id: ContractId,
    ) -> Self {
        Self::Contract {
            utxo_id,
            balance_root,
            state_root,
            tx_pointer,
            contract_id,
        }
    }

    pub const fn message_signed(
        message_id: MessageId,
        sender: Address,
        recipient: Address,
        amount: Word,
        nonce: Word,
        witness_index: u8,
        data: Vec<u8>,
    ) -> Self {
        Self::MessageSigned {
            message_id,
            sender,
            recipient,
            amount,
            nonce,
            witness_index,
            data,
        }
    }

    pub const fn message_predicate(
        message_id: MessageId,
        sender: Address,
        recipient: Address,
        amount: Word,
        nonce: Word,
        data: Vec<u8>,
        predicate: Vec<u8>,
        predicate_data: Vec<u8>,
    ) -> Self {
        Self::MessagePredicate {
            message_id,
            sender,
            recipient,
            amount,
            nonce,
            data,
            predicate,
            predicate_data,
        }
    }

    pub const fn utxo_id(&self) -> Option<&UtxoId> {
        match self {
            Self::CoinSigned { utxo_id, .. }
            | Self::CoinPredicate { utxo_id, .. }
            | Self::Contract { utxo_id, .. } => Some(utxo_id),
            Self::MessageSigned { .. } => None,
            Self::MessagePredicate { .. } => None,
        }
    }

    pub const fn input_owner(&self) -> Option<&Address> {
        match self {
            Self::CoinSigned { owner, .. } | Self::CoinPredicate { owner, .. } => Some(owner),
            Self::MessageSigned { .. } | Self::MessagePredicate { .. } | Self::Contract { .. } => {
                None
            }
        }
    }

    pub const fn asset_id(&self) -> Option<&AssetId> {
        match self {
            Input::CoinSigned { asset_id, .. } | Input::CoinPredicate { asset_id, .. } => {
                Some(asset_id)
            }
            Input::MessageSigned { .. } | Input::MessagePredicate { .. } => Some(&AssetId::BASE),
            Input::Contract { .. } => None,
        }
    }

    pub const fn contract_id(&self) -> Option<&ContractId> {
        match self {
            Self::Contract { contract_id, .. } => Some(contract_id),
            _ => None,
        }
    }

    pub const fn amount(&self) -> Option<Word> {
        match self {
            Input::CoinSigned { amount, .. }
            | Input::CoinPredicate { amount, .. }
            | Input::MessageSigned { amount, .. }
            | Input::MessagePredicate { amount, .. } => Some(*amount),
            Input::Contract { .. } => None,
        }
    }

    pub const fn witness_index(&self) -> Option<u8> {
        match self {
            Input::CoinSigned { witness_index, .. }
            | Input::MessageSigned { witness_index, .. } => Some(*witness_index),
            Input::CoinPredicate { .. }
            | Input::Contract { .. }
            | Input::MessagePredicate { .. } => None,
        }
    }

    pub const fn maturity(&self) -> Option<Word> {
        match self {
            Input::CoinSigned { maturity, .. } | Input::CoinPredicate { maturity, .. } => {
                Some(*maturity)
            }
            Input::Contract { .. }
            | Input::MessageSigned { .. }
            | Input::MessagePredicate { .. } => None,
        }
    }

    pub fn predicate_offset(&self) -> Option<usize> {
        match self {
            Input::CoinPredicate { .. } => InputRepr::Coin.coin_predicate_offset(),
            Input::MessagePredicate { data, .. } => InputRepr::Message
                .data_offset()
                .map(|o| o + bytes::padded_len(data)),
            Input::CoinSigned { .. } | Input::Contract { .. } | Input::MessageSigned { .. } => None,
        }
    }

    pub fn predicate_data_offset(&self) -> Option<usize> {
        match self {
            Input::CoinPredicate { predicate, .. } | Input::MessagePredicate { predicate, .. } => {
                self.predicate_offset()
                    .map(|o| o + bytes::padded_len(predicate))
            }
            Input::CoinSigned { .. } | Input::Contract { .. } | Input::MessageSigned { .. } => None,
        }
    }

    pub fn predicate_len(&self) -> Option<usize> {
        match self {
            Input::CoinPredicate { predicate, .. } | Input::MessagePredicate { predicate, .. } => {
                Some(predicate.len())
            }
            Input::CoinSigned { .. } | Input::MessageSigned { .. } => Some(0),
            Input::Contract { .. } => None,
        }
    }

    pub fn predicate_data_len(&self) -> Option<usize> {
        match self {
            Input::CoinPredicate { predicate_data, .. }
            | Input::MessagePredicate { predicate_data, .. } => Some(predicate_data.len()),
            Input::CoinSigned { .. } | Input::MessageSigned { .. } => Some(0),
            Input::Contract { .. } => None,
        }
    }

    pub const fn message_id(&self) -> Option<&MessageId> {
        match self {
            Self::MessagePredicate { message_id, .. } | Self::MessageSigned { message_id, .. } => {
                Some(message_id)
            }
            _ => None,
        }
    }

    pub const fn tx_pointer(&self) -> Option<&TxPointer> {
        match self {
            Input::CoinSigned { tx_pointer, .. }
            | Input::CoinPredicate { tx_pointer, .. }
            | Input::Contract { tx_pointer, .. } => Some(tx_pointer),
            _ => None,
        }
    }

    pub fn input_data(&self) -> Option<&[u8]> {
        match self {
            Input::MessageSigned { data, .. } | Input::MessagePredicate { data, .. } => Some(data),
            _ => None,
        }
    }

    pub fn input_predicate(&self) -> Option<&[u8]> {
        match self {
            Input::CoinPredicate { predicate, .. } | Input::MessagePredicate { predicate, .. } => {
                Some(predicate)
            }

            _ => None,
        }
    }

    pub fn input_predicate_data(&self) -> Option<&[u8]> {
        match self {
            Input::CoinPredicate { predicate_data, .. }
            | Input::MessagePredicate { predicate_data, .. } => Some(predicate_data),

            _ => None,
        }
    }

    /// Return a tuple containing the predicate and its data if the input is of
    /// type `CoinPredicate` or `MessagePredicate`
    pub fn predicate(&self) -> Option<(&[u8], &[u8])> {
        match self {
            Input::CoinPredicate {
                predicate,
                predicate_data,
                ..
            }
            | Input::MessagePredicate {
                predicate,
                predicate_data,
                ..
            } => Some((predicate.as_slice(), predicate_data.as_slice())),

            _ => None,
        }
    }

    pub const fn is_coin(&self) -> bool {
        self.is_coin_signed() | self.is_coin_predicate()
    }

    pub const fn is_coin_signed(&self) -> bool {
        matches!(self, Input::CoinSigned { .. })
    }

    pub const fn is_coin_predicate(&self) -> bool {
        matches!(self, Input::CoinPredicate { .. })
    }

    pub const fn is_message(&self) -> bool {
        self.is_message_signed() | self.is_message_predicate()
    }

    pub const fn is_message_signed(&self) -> bool {
        matches!(self, Input::MessageSigned { .. })
    }

    pub const fn is_message_predicate(&self) -> bool {
        matches!(self, Input::MessagePredicate { .. })
    }

    pub const fn is_contract(&self) -> bool {
        matches!(self, Input::Contract { .. })
    }

    pub const fn coin_predicate_offset() -> usize {
        INPUT_COIN_FIXED_SIZE
    }

    pub const fn message_data_offset() -> usize {
        INPUT_MESSAGE_FIXED_SIZE
    }

    pub const fn balance_root(&self) -> Option<&Bytes32> {
        match self {
            Input::Contract { balance_root, .. } => Some(balance_root),
            _ => None,
        }
    }

    pub const fn state_root(&self) -> Option<&Bytes32> {
        match self {
            Input::Contract { state_root, .. } => Some(state_root),
            _ => None,
        }
    }

    pub const fn sender(&self) -> Option<&Address> {
        match self {
            Input::MessageSigned { sender, .. } | Input::MessagePredicate { sender, .. } => {
                Some(sender)
            }
            _ => None,
        }
    }

    pub const fn recipient(&self) -> Option<&Address> {
        match self {
            Input::MessageSigned { recipient, .. } | Input::MessagePredicate { recipient, .. } => {
                Some(recipient)
            }
            _ => None,
        }
    }

    pub const fn nonce(&self) -> Option<Word> {
        match self {
            Input::MessageSigned { nonce, .. } | Input::MessagePredicate { nonce, .. } => {
                Some(*nonce)
            }
            _ => None,
        }
    }

    pub fn compute_message_id(
        sender: &Address,
        recipient: &Address,
        nonce: Word,
        amount: Word,
        data: &[u8],
    ) -> MessageId {
        let message_id = *Hasher::default()
            .chain(sender)
            .chain(recipient)
            .chain(nonce.to_be_bytes())
            .chain(amount.to_be_bytes())
            // TODO: Seems it is a bug and we need to pad `data` with zeros
            .chain(data)
            .finalize();

        message_id.into()
    }

    pub fn predicate_owner<P>(predicate: P) -> Address
    where
        P: AsRef<[u8]>,
    {
        use crate::Contract;

        let root = Contract::root_from_code(predicate);

        (*root).into()
    }

    #[cfg(feature = "std")]
    pub fn is_predicate_owner_valid<P>(owner: &Address, predicate: P) -> bool
    where
        P: AsRef<[u8]>,
    {
        owner == &Self::predicate_owner(predicate)
    }

    /// Prepare the output for VM predicate execution
    pub fn prepare_init_predicate(&mut self) {
        match self {
            Input::CoinSigned { tx_pointer, .. } | Input::CoinPredicate { tx_pointer, .. } => {
                mem::take(tx_pointer);
            }

            Input::Contract {
                utxo_id,
                balance_root,
                state_root,
                tx_pointer,
                ..
            } => {
                mem::take(tx_pointer);
                mem::take(balance_root);
                mem::take(state_root);
                mem::take(utxo_id);
            }

            _ => (),
        }
    }
}

impl From<Input> for InputSpec {
    fn from(input: Input) -> Self {
        match input {
            Input::CoinSigned {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer,
                witness_index,
                maturity,
            } => InputSpec::Coin {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer,
                witness_index,
                maturity,
                predicate: vec![],
                predicate_data: vec![],
            },
            Input::CoinPredicate {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer,
                maturity,
                predicate,
                predicate_data,
            } => InputSpec::Coin {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer,
                witness_index: 0,
                maturity,
                predicate,
                predicate_data,
            },
            Input::Contract {
                utxo_id,
                balance_root,
                state_root,
                tx_pointer,
                contract_id,
            } => InputSpec::Contract {
                utxo_id,
                balance_root,
                state_root,
                tx_pointer,
                contract_id,
            },
            Input::MessageSigned {
                message_id,
                sender,
                recipient,
                amount,
                nonce,
                witness_index,
                data,
            } => InputSpec::Message {
                message_id,
                sender,
                recipient,
                amount,
                nonce,
                witness_index,
                data,
                predicate: vec![],
                predicate_data: vec![],
            },
            Input::MessagePredicate {
                message_id,
                sender,
                recipient,
                amount,
                nonce,
                data,
                predicate,
                predicate_data,
            } => InputSpec::Message {
                message_id,
                sender,
                recipient,
                amount,
                nonce,
                witness_index: 0,
                data,
                predicate,
                predicate_data,
            },
        }
    }
}

impl From<InputSpec> for Input {
    fn from(input: InputSpec) -> Self {
        match input {
            InputSpec::Coin {
                utxo_id,
                owner,
                amount,
                asset_id,
                tx_pointer,
                witness_index,
                maturity,
                predicate,
                predicate_data,
            } => {
                if predicate.is_empty() {
                    Self::CoinSigned {
                        utxo_id,
                        owner,
                        amount,
                        asset_id,
                        tx_pointer,
                        witness_index,
                        maturity,
                    }
                } else {
                    Self::CoinPredicate {
                        utxo_id,
                        owner,
                        amount,
                        asset_id,
                        tx_pointer,
                        maturity,
                        predicate,
                        predicate_data,
                    }
                }
            }
            InputSpec::Contract {
                utxo_id,
                balance_root,
                state_root,
                tx_pointer,
                contract_id,
            } => Self::Contract {
                utxo_id,
                balance_root,
                state_root,
                tx_pointer,
                contract_id,
            },
            InputSpec::Message {
                message_id,
                sender,
                recipient,
                amount,
                nonce,
                witness_index,
                data,
                predicate,
                predicate_data,
            } => {
                if predicate.is_empty() {
                    Self::MessageSigned {
                        message_id,
                        sender,
                        recipient,
                        amount,
                        nonce,
                        witness_index,
                        data,
                    }
                } else {
                    Self::MessagePredicate {
                        message_id,
                        sender,
                        recipient,
                        amount,
                        nonce,
                        data,
                        predicate,
                        predicate_data,
                    }
                }
            }
        }
    }
}

impl Serialize for Input {
    #[inline(always)]
    fn encode_static<O: Output + ?Sized>(&self, buffer: &mut O) -> Result<(), Error> {
        let input_spec: InputSpec = self.clone().into();
        input_spec.encode(buffer)
    }
}

impl Deserialize for Input {
    fn decode_static<I: crate::canonical::Input + ?Sized>(buffer: &mut I) -> Result<Self, Error> {
        let input_spec = InputSpec::decode(buffer)?;
        Ok(input_spec.into())
    }
}
