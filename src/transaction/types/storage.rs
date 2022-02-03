use fuel_types::{bytes, Bytes32, Bytes64};
#[cfg(feature = "random")]
use rand::{distributions::Standard, prelude::Distribution, Rng};
use std::cmp::Ordering;
use std::io;

pub const SLOT_SIZE: usize = Bytes64::LEN;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde-types-minimal",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StorageSlot(Bytes32, Bytes32);

impl StorageSlot {
    pub fn new(key: Bytes32, value: Bytes32) -> Self {
        StorageSlot(key, value)
    }
}

#[cfg(feature = "random")]
impl Distribution<StorageSlot> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> StorageSlot {
        StorageSlot(rng.gen(), rng.gen())
    }
}

impl io::Read for StorageSlot {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < SLOT_SIZE {
            return Err(bytes::eof());
        }
        buf = bytes::store_array_unchecked(buf, &self.0);
        bytes::store_array_unchecked(buf, &self.1);
        Ok(SLOT_SIZE)
    }
}

impl io::Write for StorageSlot {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() < SLOT_SIZE {
            return Err(bytes::eof());
        }

        // Safety: buf len is checked
        let (key, buf) = unsafe { bytes::restore_array_unchecked(buf) };
        let (value, _) = unsafe { bytes::restore_array_unchecked(buf) };

        self.0 = key.into();
        self.1 = value.into();
        Ok(SLOT_SIZE)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl bytes::SizedBytes for StorageSlot {
    fn serialized_size(&self) -> usize {
        SLOT_SIZE
    }
}

impl PartialOrd for StorageSlot {
    fn partial_cmp(&self, other: &StorageSlot) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for StorageSlot {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
