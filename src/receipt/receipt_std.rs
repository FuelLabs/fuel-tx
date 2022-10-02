use super::ReceiptRepr;

use fuel_types::Word;

use std::io;

impl TryFrom<Word> for ReceiptRepr {
    type Error = io::Error;

    fn try_from(b: Word) -> Result<Self, Self::Error> {
        match b {
            0x00 => Ok(Self::Call),
            0x01 => Ok(Self::Return),
            0x02 => Ok(Self::ReturnData),
            0x03 => Ok(Self::Panic),
            0x04 => Ok(Self::Revert),
            0x05 => Ok(Self::Log),
            0x06 => Ok(Self::LogData),
            0x07 => Ok(Self::Transfer),
            0x08 => Ok(Self::TransferOut),
            0x09 => Ok(Self::ScriptResult),
            0x0A => Ok(Self::MessageOut),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The provided identifier is invalid!",
            )),
        }
    }
}
