use super::PoolKey;
use odra::Mapping;

#[odra::module]
pub struct Tickmap {
    pub bitmap: Mapping<PoolKey, Mapping<u16, u64>>,
}