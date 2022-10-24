mod input;
mod output;
mod storage;
mod tx_pointer;
mod witness;

pub use fuel_types::UtxoId;
pub use input::{Input, InputRepr};
pub use output::{Output, OutputRepr};
pub use storage::StorageSlot;
pub use tx_pointer::TxPointer;
pub use witness::Witness;
