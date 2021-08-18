use crate::bytes;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::array::TryFromSliceError;
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

mod input;
mod output;
mod witness;

macro_rules! key {
    ($i:ident, $s:expr) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde-types", derive(serde::Serialize, serde::Deserialize))]
        pub struct $i([u8; $s]);

        impl $i {
            pub const fn new(bytes: [u8; $s]) -> Self {
                Self(bytes)
            }

            pub const fn size_of() -> usize {
                $s
            }

            /// Add a conversion between arbitrary slices to the type
            ///
            /// # Panics
            ///
            /// Will panic if the provided slice length is different than `Self::size_of()`
            pub fn from_slice_unchecked(bytes: &[u8]) -> Self {
                $i(bytes::from_slice_unchecked(bytes))
            }
        }

        impl rand::Fill for $i {
            fn try_fill<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), rand::Error> {
                rng.fill_bytes(self.as_mut());

                Ok(())
            }
        }

        impl Distribution<$i> for Standard {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $i {
                $i(rng.gen())
            }
        }

        impl Deref for $i {
            type Target = [u8; $s];

            fn deref(&self) -> &[u8; $s] {
                &self.0
            }
        }

        impl DerefMut for $i {
            fn deref_mut(&mut self) -> &mut [u8; $s] {
                &mut self.0
            }
        }

        impl AsRef<[u8]> for $i {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl AsMut<[u8]> for $i {
            fn as_mut(&mut self) -> &mut [u8] {
                &mut self.0
            }
        }

        impl From<[u8; $s]> for $i {
            fn from(bytes: [u8; $s]) -> Self {
                Self(bytes)
            }
        }

        impl From<$i> for [u8; $s] {
            fn from(salt: $i) -> [u8; $s] {
                salt.0
            }
        }

        impl TryFrom<&[u8]> for $i {
            type Error = TryFromSliceError;

            fn try_from(bytes: &[u8]) -> Result<$i, TryFromSliceError> {
                <[u8; $s]>::try_from(bytes).map(|b| b.into())
            }
        }
    };
}

pub use input::Input;
pub use output::Output;
pub use witness::Witness;

key!(Address, 32);
key!(Color, 32);
key!(ContractId, 32);
key!(Bytes8, 8);
key!(Bytes32, 32);
key!(Salt, 32);

impl ContractId {
    pub const SEED: [u8; 4] = 0x4655454C_u32.to_be_bytes();
}

#[cfg(test)]
mod tests {
    use crate::*;
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};
    use std::convert::TryFrom;

    #[test]
    fn from_slice_unchecked() {
        let rng = &mut StdRng::seed_from_u64(8586);

        let mut bytes = [0u8; 257];
        rng.fill_bytes(&mut bytes);

        let i = &bytes[..Address::size_of()];
        let a = Address::from_slice_unchecked(i);
        let b = Address::try_from(i).expect("Memory conversion");
        assert_eq!(a, b);

        let i = &bytes[..Color::size_of()];
        let a = Color::from_slice_unchecked(i);
        let b = Color::try_from(i).expect("Memory conversion");
        assert_eq!(a, b);

        let i = &bytes[..ContractId::size_of()];
        let a = ContractId::from_slice_unchecked(i);
        let b = ContractId::try_from(i).expect("Memory conversion");
        assert_eq!(a, b);

        let i = &bytes[..Bytes8::size_of()];
        let a = Bytes8::from_slice_unchecked(i);
        let b = Bytes8::try_from(i).expect("Memory conversion");
        assert_eq!(a, b);

        let i = &bytes[..Bytes32::size_of()];
        let a = Bytes32::from_slice_unchecked(i);
        let b = Bytes32::try_from(i).expect("Memory conversion");
        assert_eq!(a, b);

        let i = &bytes[..Salt::size_of()];
        let a = Salt::from_slice_unchecked(i);
        let b = Salt::try_from(i).expect("Memory conversion");
        assert_eq!(a, b);
    }
}
