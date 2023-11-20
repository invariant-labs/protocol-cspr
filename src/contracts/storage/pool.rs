use odra::types::{U128,U256, Address};
use odra::OdraType;
use crate::contracts::Oracle;


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
    pub oracle_address: Oracle,
    pub oracle_initialized: bool,
}