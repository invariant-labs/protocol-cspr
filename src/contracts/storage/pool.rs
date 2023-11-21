use super::{FeeTier, PoolKey, Tick, Tickmap}; // Oracle,
use crate::{U128T, U256T};
use alloc::string::ToString;
use decimal::num_traits::WrappingAdd;
use decimal::*;
use invariant_math::{
    calculate_amount_delta,
    fee_growth::FeeGrowth,
    is_enough_amount_to_change_price,
    liquidity::Liquidity,
    percentage::Percentage,
    seconds_per_liquidity::{calculate_seconds_per_liquidity_inside, SecondsPerLiquidity},
    sqrt_price::SqrtPrice,
    sqrt_price::{calculate_sqrt_price, get_tick_at_sqrt_price},
    token_amount::TokenAmount,
};
use odra::types::{casper_types::account::AccountHash, Address, U128, U256};
use odra::OdraType;
use traceable_result::*;

#[derive(OdraType)]
pub struct Pool {
    pub liquidity: U256,
    pub sqrt_price: U128,
    pub current_tick_index: i32, // nearest tick below the current sqrt_price
    pub fee_growth_global_x: U128,
    pub fee_growth_global_y: U128,
    pub fee_protocol_token_x: U256,
    pub fee_protocol_token_y: U256,
    pub seconds_per_liquidity_global: U128,
    pub start_timestamp: u64,
    pub last_timestamp: u64,
    pub fee_receiver: Address,
    // pub oracle_address: Oracle,
    pub oracle_initialized: bool,
}
