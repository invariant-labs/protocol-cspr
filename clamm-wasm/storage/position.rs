use crate::PoolKey;
use crate::{fee_growth::FeeGrowth, liquidity::Liquidity, token_amount::TokenAmount};
use odra::OdraType;

use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(OdraType, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub pool_key: PoolKey,
    pub liquidity: Liquidity,
    #[tsify(type = "bigint")]
    pub lower_tick_index: i32,
    #[tsify(type = "bigint")]
    pub upper_tick_index: i32,
    pub fee_growth_inside_x: FeeGrowth,
    pub fee_growth_inside_y: FeeGrowth,
    #[tsify(type = "bigint")]
    pub last_block_number: u64,
    pub tokens_owed_x: TokenAmount,
    pub tokens_owed_y: TokenAmount,
}
