use crate::Transaction;

use core::fmt;
use core::str::FromStr;

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // its not worthy to extend the API requirement of serde because the transaction
        // serialization into json is not a fallible operation. We could alternatively just unwrap
        // here, but as a library its better to avoid runtime panic from user input
        serde_json::to_string(self).unwrap_or_default().fmt(f)
    }
}

#[cfg(feature = "std")]
impl FromStr for Transaction {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::io::{Error, ErrorKind};

        serde_json::from_str(s).map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
