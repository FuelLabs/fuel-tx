use super::Transaction;

/// Entity support metadata computation to cache results.
pub trait Cacheable {
    /// The cache is already computed.
    ///
    /// # Note: `true` doesn't mean that the cache is actual.
    fn is_computed(&self) -> bool;

    /// Computes the cache for the entity.
    fn precompute(&mut self);
}

impl Cacheable for Transaction {
    fn is_computed(&self) -> bool {
        match self {
            Self::Script(script) => script.is_computed(),
            Self::Create(create) => create.is_computed(),
        }
    }

    fn precompute(&mut self) {
        match self {
            Self::Script(script) => script.precompute(),
            Self::Create(create) => create.precompute(),
        }
    }
}
