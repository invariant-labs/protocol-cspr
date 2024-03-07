use super::PoolKey;
use crate::{liquidity::Liquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount};
use odra::prelude::vec::Vec;
use odra::Event;

use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(Event, PartialEq, Eq, Debug, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CreatePositionEvent {
    #[tsify(type = "bigint")]
    pub timestamp: u64,
    pub address: String,
    pub pool: PoolKey,
    pub liquidity: Liquidity,
    #[tsify(type = "bigint")]
    pub lower_tick: i32,
    #[tsify(type = "bigint")]
    pub upper_tick: i32,
    pub current_sqrt_price: SqrtPrice,
}

#[derive(Event, PartialEq, Eq, Debug, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RemovePositionEvent {
    #[tsify(type = "bigint")]
    pub timestamp: u64,
    pub address: String,
    pub pool: PoolKey,
    pub liquidity: Liquidity,
    #[tsify(type = "bigint")]
    pub lower_tick: i32,
    #[tsify(type = "bigint")]
    pub upper_tick: i32,
    pub current_sqrt_price: SqrtPrice,
}

#[derive(Event, PartialEq, Eq, Debug, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrossTickEvent {
    #[tsify(type = "bigint")]
    pub timestamp: u64,
    pub address: String,
    pub pool: PoolKey,
    #[tsify(type = "bigint[]")]
    pub indexes: Vec<i32>,
}

#[derive(Event, PartialEq, Eq, Debug, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SwapEvent {
    #[tsify(type = "bigint")]
    pub timestamp: u64,
    pub address: String,
    pub pool: PoolKey,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee: TokenAmount,
    pub start_sqrt_price: SqrtPrice,
    pub target_sqrt_price: SqrtPrice,
    pub x_to_y: bool,
}
