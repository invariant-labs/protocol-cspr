use crate::fee_growth::FeeGrowth;
use crate::liquidity::Liquidity;
use crate::sqrt_price::SqrtPrice;
use decimal::*;
use odra::types::{U128, U256};
use odra::OdraType;

use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(OdraType, PartialEq, Eq, Debug, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Tick {
    #[tsify(type = "bigint")]
    pub index: i32,
    pub sign: bool,
    pub liquidity_change: Liquidity,
    pub liquidity_gross: Liquidity,
    pub sqrt_price: SqrtPrice,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    #[tsify(type = "bigint")]
    pub seconds_outside: u64,
}

impl Default for Tick {
    fn default() -> Self {
        Tick {
            index: 0i32,
            sign: false,
            liquidity_change: Liquidity::new(U256::from(0)),
            liquidity_gross: Liquidity::new(U256::from(0)),
            sqrt_price: SqrtPrice::from_integer(1),
            fee_growth_outside_x: FeeGrowth::new(U128::from(0)),
            fee_growth_outside_y: FeeGrowth::new(U128::from(0)),
            seconds_outside: 0u64,
        }
    }
}
