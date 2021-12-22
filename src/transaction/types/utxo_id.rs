use crate::TxId;

/// Identification of unspend transaction output.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde-types-minimal",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct UtxoId {
    /// transaction id
    pub tx_id: TxId,
    /// output index
    pub output_index: u8,
}
