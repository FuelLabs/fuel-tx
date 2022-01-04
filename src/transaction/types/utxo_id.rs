use crate::TxId;

/// Identification of unspend transaction output.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
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

impl core::fmt::LowerHex for UtxoId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:x}{:02x}", self.tx_id, self.output_index)
        } else {
            write!(f, "{:#x}{:02x}", self.tx_id, self.output_index)
        }
    }
}

impl core::fmt::UpperHex for UtxoId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:X}{:02X}", self.tx_id, self.output_index)
        } else {
            write!(f, "{:#X}{:02X}", self.tx_id, self.output_index)
        }
    }
}

#[cfg(test)]
mod tests {
    use fuel_types::Bytes32;
    use rand::{prelude::StdRng, Rng, SeedableRng};

    use super::*;

    #[test]
    fn fmt_utxo_id() {
        let mut tx_id = Bytes32::zeroed();
        *tx_id.get_mut(0).unwrap() = 9;
        *tx_id.get_mut(31).unwrap() = 8;

        let utxo_id = UtxoId {
            tx_id,
            output_index: 7,
        };
        assert_eq!(
            format!("{:x}", utxo_id),
            "0x090000000000000000000000000000000000000000000000000000000000000807"
        );
    }
}
