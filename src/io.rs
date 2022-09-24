use alloc::vec::Vec;
use fuel_asm::InstructionResult;
pub use fuel_tx_derive::{Deserialize, Serialize};
use fuel_types::{
    Address, AssetId, Bytes20, Bytes32, Bytes4, Bytes8, ContractId, MessageId, Salt, Word,
};

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// The data of each field should be 64 bits aligned.
    IsNotAligned,
    /// The buffer is to short for writing or reading.
    BufferItTooShort,
    /// Got unknown enum's discriminant.
    UnknownDiscriminant,
    /// Wrong align.
    WrongAlign,
    /// Unknown error.
    Unknown(&'static str),
}

/// Allows writing of data.
pub trait Output {
    /// Write bytes to the output buffer.
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error>;

    /// Write a single byte to the output buffer.
    fn push_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.write(&[byte])
    }
}

/// !INTERNAL USAGE ONLY!
/// This enum provides type information required for specialization and deserialization.
pub enum Type {
    U8,
    Unknown,
}

/// Allows serialize the type into the `Output`.
pub trait Serialize {
    // !INTERNAL USAGE ONLY!
    #[doc(hidden)]
    const TYPE: Type = Type::Unknown;

    /// Returns the size required for serialization inner data.
    fn size(&self) -> usize {
        let mut calculator = SizeCalculator(0);
        self.encode(&mut calculator)
            .expect("Can't encode to get a size");
        self.encode_extra(&mut calculator)
            .expect("Can't encode extras to get a size");
        calculator.size()
    }

    /// Encodes into bytes vector.
    fn to_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(self.size());
        self.encode(&mut vec).expect("Unable to encode self");
        self.encode_extra(&mut vec)
            .expect("Unable to encode extras for self");
        vec
    }

    /// Encodes all data required for `Self` deserialization into the `buffer`.
    fn encode<O: Output + ?Sized>(&self, buffer: &mut O) -> Result<(), Error>;

    /// Encodes extra information required to fill `Self` during deserialization.
    ///
    /// # Note: It is empty for primitives. But it can be helpful for containers because this
    /// method is called at the end of struct/enum serialization.
    fn encode_extra<O: Output + ?Sized>(&self, _buffer: &mut O) -> Result<(), Error> {
        Ok(())
    }
}

/// Allows reading of data into a slice.
pub trait Input {
    /// Returns the remaining length of the input data.
    fn remaining(&mut self) -> usize;

    /// Read the exact number of bytes required to fill the given buffer.
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    /// Read a single byte from the input.
    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut buf = [0u8];
        self.read(&mut buf[..])?;
        Ok(buf[0])
    }

    /// Skips next `n` bytes.
    fn skip(&mut self, n: usize) -> Result<(), Error>;
}

/// Allows deserialize the type from the `Input`.
pub trait Deserialize: Sized {
    // !INTERNAL USAGE ONLY!
    #[doc(hidden)]
    const TYPE: Type = Type::Unknown;

    /// Decode `Self` from the `buffer`.
    fn decode<I: Input + ?Sized>(buffer: &mut I) -> Result<Self, Error>;

    /// Decodes extra information from the `buffer` to fill `Self`.
    ///
    /// # Note: It is empty for primitives. But it can be helpful for containers to fill elements.
    fn decode_extra<I: Input + ?Sized>(&mut self, _buffer: &mut I) -> Result<(), Error> {
        Ok(())
    }
}

/// The data of each field should be 64 bits aligned.
pub const ALIGN: usize = 8;

/// Returns the number of bytes to fill aligned
const fn fill_bytes(len: usize) -> usize {
    (ALIGN - (len % ALIGN)) % ALIGN
}

/// Returns the number of bytes to fill aligned
macro_rules! fill_bytes {
    ($t:ident) => {{
        fill_bytes(::core::mem::size_of::<$t>())
    }};
}

macro_rules! impl_for_type_aligned {
    ($t:ident) => {
        impl Serialize for $t {
            fn encode<O: Output + ?Sized>(&self, buffer: &mut O) -> Result<(), Error> {
                // It will be removed by the compiler because it is a const expression.
                // It is a check for future potential changes.
                if ::core::mem::size_of::<$t>() % ALIGN > 0 {
                    return Err(Error::IsNotAligned);
                }
                buffer.write(self.as_ref())
            }
        }

        impl Deserialize for $t {
            fn decode<I: Input + ?Sized>(buffer: &mut I) -> Result<Self, Error> {
                let mut asset = $t::default();
                buffer.read(asset.as_mut())?;
                Ok(asset)
            }
        }
    };
}

impl_for_type_aligned!(Address);
impl_for_type_aligned!(AssetId);
impl_for_type_aligned!(ContractId);
impl_for_type_aligned!(Bytes8);
impl_for_type_aligned!(Bytes32);
impl_for_type_aligned!(MessageId);
impl_for_type_aligned!(Salt);

macro_rules! impl_for_type_not_aligned {
    ($t:ident) => {
        impl Serialize for $t {
            fn encode<O: Output + ?Sized>(&self, buffer: &mut O) -> Result<(), Error> {
                const FILL_SIZE: usize = fill_bytes!($t);
                let zeroed: [u8; FILL_SIZE] = [0; FILL_SIZE];
                buffer.write(self.as_ref())?;
                buffer.write(zeroed.as_ref())
            }
        }

        impl Deserialize for $t {
            fn decode<I: Input + ?Sized>(buffer: &mut I) -> Result<Self, Error> {
                let mut asset = $t::default();
                buffer.read(asset.as_mut())?;
                buffer.skip(fill_bytes!($t))?;
                Ok(asset)
            }
        }
    };
}

impl_for_type_not_aligned!(Bytes4);
impl_for_type_not_aligned!(Bytes20);

macro_rules! impl_for_primitives {
    ($t:ident, $ty:path) => {
        impl Serialize for $t {
            const TYPE: Type = $ty;

            fn encode<O: Output + ?Sized>(&self, buffer: &mut O) -> Result<(), Error> {
                const FILL_SIZE: usize = fill_bytes!($t);
                let zeroed: [u8; FILL_SIZE] = [0; FILL_SIZE];

                let bytes = <$t>::to_be_bytes(*self);
                buffer.write(bytes.as_ref())?;
                buffer.write(zeroed.as_ref())
            }
        }

        impl Deserialize for $t {
            const TYPE: Type = $ty;

            fn decode<I: Input + ?Sized>(buffer: &mut I) -> Result<Self, Error> {
                let mut asset = [0u8; ::core::mem::size_of::<$t>()];
                buffer.read(asset.as_mut())?;
                buffer.skip(fill_bytes!($t))?;
                Ok(<$t>::from_be_bytes(asset))
            }
        }
    };
}

impl_for_primitives!(u8, Type::U8);
impl_for_primitives!(u16, Type::Unknown);
impl_for_primitives!(u32, Type::Unknown);
impl_for_primitives!(usize, Type::Unknown);
impl_for_primitives!(u64, Type::Unknown);
impl_for_primitives!(u128, Type::Unknown);

// `Option` is not supported by the specification. So ignore them.
impl<T> Serialize for Option<T> {
    fn size(&self) -> usize {
        0
    }

    fn encode<O: Output + ?Sized>(&self, _buffer: &mut O) -> Result<(), Error> {
        Ok(())
    }
}

// `Option` is not supported by the specification. So ignore them.
impl<T> Deserialize for Option<T> {
    fn decode<I: Input + ?Sized>(_buffer: &mut I) -> Result<Self, Error> {
        Ok(None)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    // Encode only the size of the vector. Elements will be encoded in the `encode_extra` method.
    fn encode<O: Output + ?Sized>(&self, buffer: &mut O) -> Result<(), Error> {
        self.len().encode(buffer)
    }

    fn encode_extra<O: Output + ?Sized>(&self, buffer: &mut O) -> Result<(), Error> {
        for e in self.iter() {
            // Bytes - Vec<u8> it a separate case without padding for each element.
            // It should padded at the end if is not % ALIGN
            match T::TYPE {
                Type::U8 => {
                    // Safety: `Type::U8` implemented only for `u8`.
                    let byte = unsafe { ::core::mem::transmute::<&T, &u8>(e) };
                    buffer.push_byte(*byte)?;
                }
                Type::Unknown => {
                    e.encode(buffer)?;
                    e.encode_extra(buffer)?;
                }
            };
        }

        if let Type::U8 = T::TYPE {
            for _ in 0..fill_bytes(self.capacity()) {
                buffer.push_byte(0)?;
            }
        }

        Ok(())
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    // Decode only the capacity of the vector. Elements will be decoded in the `decode_extra` method.
    // The capacity is needed for iteration there.
    fn decode<I: Input + ?Sized>(buffer: &mut I) -> Result<Self, Error> {
        let cap: usize = usize::decode(buffer)?;

        Ok(Vec::with_capacity(cap))
    }

    fn decode_extra<I: Input + ?Sized>(&mut self, buffer: &mut I) -> Result<(), Error> {
        for _ in 0..self.capacity() {
            // Bytes - Vec<u8> it a separate case without unpadding for each element.
            // It should unpadded at the end if is not % ALIGN
            match T::TYPE {
                Type::U8 => {
                    let byte = buffer.read_byte()?;
                    // Safety: `Type::U8` implemented only for `u8`, so it is `Vec<u8>`.
                    let _self =
                        unsafe { ::core::mem::transmute::<&mut Vec<T>, &mut Vec<u8>>(self) };
                    _self.push(byte);
                }
                Type::Unknown => {
                    let mut e = T::decode(buffer)?;
                    e.decode_extra(buffer)?;
                    self.push(e);
                }
            };
        }

        if let Type::U8 = T::TYPE {
            buffer.skip(fill_bytes(self.capacity()))?;
        }

        Ok(())
    }
}

impl Output for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.extend_from_slice(bytes);
        Ok(())
    }
}

impl<'a> Output for &'a mut [u8] {
    fn write(&mut self, from: &[u8]) -> Result<(), Error> {
        if from.len() > self.len() {
            return Err(Error::BufferItTooShort);
        }
        let len = from.len();
        self[..len].copy_from_slice(from);
        // We need to reduce the inner slice by `len`, because we already filled them.
        let reduced = &mut self[len..];

        // Compiler is not clever enough to allow it.
        // https://stackoverflow.com/questions/25730586/how-can-i-create-my-own-data-structure-with-an-iterator-that-returns-mutable-ref
        *self = unsafe { &mut *(reduced as *mut [u8]) };
        Ok(())
    }
}

pub struct SizeCalculator(usize);

impl SizeCalculator {
    pub fn size(self) -> usize {
        self.0
    }
}

impl Output for SizeCalculator {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or(Error::BufferItTooShort)?;
        Ok(())
    }
}

impl<'a> Input for &'a [u8] {
    fn remaining(&mut self) -> usize {
        self.len()
    }

    fn read(&mut self, into: &mut [u8]) -> Result<(), Error> {
        if into.len() > self.len() {
            return Err(Error::BufferItTooShort);
        }

        let len = into.len();
        into.copy_from_slice(&self[..len]);
        *self = &self[len..];
        Ok(())
    }

    fn skip(&mut self, n: usize) -> Result<(), Error> {
        if n > self.len() {
            return Err(Error::BufferItTooShort);
        }

        *self = &self[n..];
        Ok(())
    }
}

// TODO: Move trait definition to `fuel-types` and derive this implementation for `fuel-asm`.
impl Serialize for InstructionResult {
    fn encode<O: Output + ?Sized>(&self, buffer: &mut O) -> Result<(), Error> {
        let word: Word = (*self).into();
        word.encode(buffer)
    }
}

impl Deserialize for InstructionResult {
    fn decode<I: Input + ?Sized>(buffer: &mut I) -> Result<Self, Error> {
        let word: Word = Word::decode(buffer)?;
        Ok(word.into())
    }
}

// TODO: Add tests for primitives, vectors, structs, enums
