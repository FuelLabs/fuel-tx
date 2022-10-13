use super::TransactionRepr;
use crate::{Create, Script, Transaction};

use fuel_types::bytes::{self, SizedBytes, WORD_SIZE};
use fuel_types::Word;

use std::io::{self, Write};

impl Transaction {
    pub fn try_from_bytes(bytes: &[u8]) -> io::Result<(usize, Self)> {
        let mut tx = Self::default();

        let n = tx.write(bytes)?;

        Ok((n, tx))
    }
}

impl io::Read for Transaction {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.serialized_size();
        if buf.len() < n {
            return Err(bytes::eof());
        }

        match self {
            Self::Script(script) => {
                let buf = bytes::store_number_unchecked(buf, TransactionRepr::Script as Word);
                script.read(buf)
            }

            Self::Create(create) => {
                let buf = bytes::store_number_unchecked(buf, TransactionRepr::Create as Word);
                create.read(buf)
            }
        }
    }
}

impl Write for Transaction {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() < WORD_SIZE {
            return Err(bytes::eof());
        }

        // Safety: buffer size is checked
        let (identifier, buf): (Word, _) = unsafe { bytes::restore_number_unchecked(buf) };
        let identifier = TransactionRepr::try_from(identifier)?;

        match identifier {
            TransactionRepr::Script => {
                let mut script = Script::default();
                let n = script.write(buf)?;

                *self = Transaction::Script(script);

                Ok(n)
            }

            TransactionRepr::Create => {
                let mut create = Create::default();
                let n = create.write(buf)?;

                *self = Transaction::Create(create);

                Ok(n)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Transaction::Script(script) => script.flush(),
            Transaction::Create(create) => create.flush(),
        }
    }
}
