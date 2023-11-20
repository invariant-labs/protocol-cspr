use odra::types::{U128, U256};
use odra::OdraType;

#[derive(OdraType)]
pub struct Tick {
    pub index: i32,
    pub sign: bool,
    pub liquidity_change: U256,
    pub liquidity_gross: U256,
    pub sqrt_price: U128,
    pub fee_growth_outside_x: U128,
    pub fee_growth_outside_y: U128,
    pub seconds_per_liquidity_outside: U128,
    pub seconds_outside: u64,
}
