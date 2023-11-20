use odra::types::{U128,U256};
use odra::OdraType;
use crate::contracts::PoolKey;


#[derive(OdraType)]
pub struct Position {
    pub pool_key: PoolKey,
    pub liquidity: U256,
    pub lower_tick_index: i32,
    pub upper_tick_index: i32,
    pub fee_growth_inside_x: U128,
    pub fee_growth_inside_y: U128,
    pub seconds_per_liquidity_inside: U128,
    pub last_block_number: u64,
    pub tokens_owed_x: U256,
    pub tokens_owed_y: U256,
}
