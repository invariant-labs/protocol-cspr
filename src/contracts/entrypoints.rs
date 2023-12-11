use super::{FeeTier, Pool, PoolKey};
use crate::{math::percentage::Percentage, InvariantError};
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

    fn get_protocol_fee(&self) -> Percentage;
    fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError>;
    fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Result<(), InvariantError>;
    fn change_fee_receiver(
        &mut self,
        pool_key: PoolKey,
        fee_receiver: Address,
    ) -> Result<(), InvariantError>;
}
