use std::str::FromStr;

use fuel_types::Bytes32;

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

impl FromStr for UtxoId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const ERR: &str = "Invalid encoded byte";
        let alternate = s.starts_with("0x");
        let s = if alternate { &s[2..] } else { s };
        let utxo_id = if s.len() == 0 {
            UtxoId {
                tx_id: Bytes32::default(),
                output_index: 0,
            }
        } else {
            UtxoId {
                tx_id: Bytes32::from_str(&s[..s.len() - 1])?,
                output_index: s
                    .as_bytes()
                    .last()
                    .cloned()
                    .map(hex_val)
                    .flatten()
                    .ok_or(ERR)?,
            }
        };
        Ok(utxo_id)
    }
}

const fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'0'..=b'9' => Some(c - b'0'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fuel_types::Bytes32;

    #[test]
    fn fmt_utxo_id() {
        let mut tx_id = Bytes32::zeroed();
        *tx_id.get_mut(0).unwrap() = 12;
        *tx_id.get_mut(31).unwrap() = 11;

        let utxo_id = UtxoId {
            tx_id,
            output_index: 10,
        };
        assert_eq!(
            format!("{:x}", utxo_id),
            "0x0c0000000000000000000000000000000000000000000000000000000000000b0a"
        );
    }

    #[test]
    fn from_str_utxo_id() -> Result<(), &'static str> {
        let utxo_id = UtxoId::from_str(
            "0x0c0000000000000000000000000000000000000000000000000000000000000b0a",
        )?;

        assert_eq!(utxo_id.output_index, 10);
        assert_eq!(utxo_id.tx_id[31], 11);
        assert_eq!(utxo_id.tx_id[0], 12);
        Ok(())
    }
}
