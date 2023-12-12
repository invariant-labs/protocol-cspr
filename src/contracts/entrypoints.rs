use super::{FeeTier, InvariantError, Pool, PoolKey, Position, Tick};
use crate::{
    math::{
        liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice,
        token_amount::TokenAmount,
    },
    CalculateSwapResult, QuoteResult, SwapHop,
};

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

    fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool;
    fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError>;

    fn claim_fee(&mut self, index: u32) -> Result<(TokenAmount, TokenAmount), InvariantError>;

    fn create_position(
        &mut self,
        pool_key: PoolKey,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: Liquidity,
        slippage_limit_lower: SqrtPrice,
        slippage_limit_upper: SqrtPrice,
    ) -> Result<Position, InvariantError>;

    fn transfer_position(&mut self, index: u32, receiver: Address) -> Result<(), InvariantError>;

    fn remove_position(&mut self, index: u32)
        -> Result<(TokenAmount, TokenAmount), InvariantError>;

    fn get_position(&mut self, index: u32) -> Result<Position, InvariantError>;

    fn get_all_positions(&mut self) -> Vec<Position>;

    fn quote(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<QuoteResult, InvariantError>;
    fn swap(
        &mut self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<CalculateSwapResult, InvariantError>;

    fn quote_route(
        &mut self,
        amount_in: TokenAmount,
        swaps: Vec<SwapHop>,
    ) -> Result<TokenAmount, InvariantError>;
    fn swap_route(
        &mut self,
        amount_in: TokenAmount,
        expected_amount_out: TokenAmount,
        slippage: Percentage,
        swaps: Vec<SwapHop>,
    ) -> Result<(), InvariantError>;
}
