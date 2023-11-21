use decimal::*;
use invariant_math::liquidity::Liquidity;
use invariant_math::sqrt_price::SqrtPrice;
use odra::types::{U128, U256};
use odra::OdraType;
use traceable_result::*;


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

impl Default for Tick {
    fn default() -> Self {
        Tick {
            index: 0i32,
            sign: false,
            liquidity_change: U256::from(0),
            liquidity_gross: U256::from(0),
            sqrt_price: U128::default(),
            fee_growth_outside_x: U128::from(0),
            fee_growth_outside_y: U128::from(0),
            seconds_per_liquidity_outside: U128::from(0),
            seconds_outside: 0u64,
        }
    }
}