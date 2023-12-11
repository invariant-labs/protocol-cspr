use super::{FeeTier, Pool, PoolKey, Tick};
use crate::InvariantError;
use odra::{prelude::vec::Vec, types::Address};

pub trait Entrypoints {
    fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;
    fn fee_tier_exist(&self, fee_tier: FeeTier) -> bool;
    fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;
    fn get_fee_tiers(&self) -> Vec<FeeTier>;

    fn create_pool(
        &mut self,
        token_0: Address,
        token_1: Address,
        fee_tier: FeeTier,
        init_tick: i32,
    ) -> Result<(), InvariantError>;
    fn get_pool(
        &self,
        token_0: Address,
        token_1: Address,
        fee_tier: FeeTier,
    ) -> Result<Pool, InvariantError>;
    fn get_pools(&self) -> Vec<PoolKey>;

    fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool;

    fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError>;
}
