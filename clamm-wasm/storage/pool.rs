use crate::{
    fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use odra::OdraType;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(OdraType, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    pub liquidity: Liquidity,
    pub sqrt_price: SqrtPrice,
    #[tsify(type = "bigint")]
    pub current_tick_index: i32,
    pub fee_growth_global_x: FeeGrowth,
    pub fee_growth_global_y: FeeGrowth,
    pub fee_protocol_token_x: TokenAmount,
    pub fee_protocol_token_y: TokenAmount,
    #[tsify(type = "bigint")]
    pub start_timestamp: u64,
    #[tsify(type = "bigint")]
    pub last_timestamp: u64,
    pub fee_receiver: String,
    pub oracle_initialized: bool,
}

// impl Default for Pool {
//     fn default() -> Self {
//         Self {
//             fee_receiver: Address::Account(AccountHash::new([0x0; 32])),
//             liquidity: Liquidity::default(),
//             sqrt_price: SqrtPrice::default(),
//             current_tick_index: i32::default(),
//             fee_growth_global_x: FeeGrowth::default(),
//             fee_growth_global_y: FeeGrowth::default(),
//             fee_protocol_token_x: TokenAmount::default(),
//             fee_protocol_token_y: TokenAmount::default(),
//             start_timestamp: u64::default(),
//             last_timestamp: u64::default(),
//             oracle_initialized: bool::default(),
//         }
//     }
// }
