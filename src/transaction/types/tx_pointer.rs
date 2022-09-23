use crate::io::{Deserialize, Serialize};
use fuel_types::bytes::{SizedBytes, WORD_SIZE};

use core::{fmt, str};

#[cfg(feature = "random")]
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

/// Identification of unspend transaction output.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Serialize, Deserialize)]
pub struct TxPointer {
    /// Block height
    block_height: u32,
    /// Transaction index
    tx_index: u16,
}

impl TxPointer {
    pub const LEN: usize = 2 * WORD_SIZE;

    pub const fn new(block_height: u32, tx_index: u16) -> Self {
        Self {
            block_height,
            tx_index,
        }
    }

    pub const fn block_height(&self) -> u32 {
        self.block_height
    }

    pub const fn tx_index(&self) -> u16 {
        self.tx_index
    }
}

#[cfg(feature = "random")]
impl Distribution<TxPointer> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TxPointer {
        TxPointer::new(rng.gen(), rng.gen())
    }
}

impl fmt::LowerHex for TxPointer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:08x}{:04x}", self.block_height, self.tx_index)
    }
}

impl fmt::UpperHex for TxPointer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:08X}{:04X}", self.block_height, self.tx_index)
    }
}

impl str::FromStr for TxPointer {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const ERR: &str = "Invalid encoded byte";

        if s.len() != 12 {
            return Err(ERR);
        }

        let block_height = u32::from_str_radix(&s[..8], 16).map_err(|_| ERR)?;
        let tx_index = u16::from_str_radix(&s[8..12], 16).map_err(|_| ERR)?;

        Ok(Self::new(block_height, tx_index))
    }
}

impl SizedBytes for TxPointer {
    fn serialized_size(&self) -> usize {
        Self::LEN
    }
}

#[test]
fn fmt_encode_decode() {
    use core::str::FromStr;

    let cases = vec![(83473, 3829)];

    for (block_height, tx_index) in cases {
        let tx_pointer = TxPointer::new(block_height, tx_index);

        let lower = format!("{:x}", tx_pointer);
        let upper = format!("{:X}", tx_pointer);

        assert_eq!(lower, format!("{:08x}{:04x}", block_height, tx_index));
        assert_eq!(upper, format!("{:08X}{:04X}", block_height, tx_index));

        let x = TxPointer::from_str(&lower).expect("failed to decode from str");
        assert_eq!(tx_pointer, x);

        let x = TxPointer::from_str(&upper).expect("failed to decode from str");
        assert_eq!(tx_pointer, x);

        #[cfg(feature = "std")]
        {
            let bytes = tx_pointer.to_bytes();
            let tx_pointer_p =
                TxPointer::decode(&mut bytes.as_slice()).expect("failed to deserialize");

            assert_eq!(tx_pointer, tx_pointer_p);
        }
    }
}
