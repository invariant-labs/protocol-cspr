use super::PoolKey;
use crate::math::{liquidity::Liquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount};
use odra::Event;
use odra::{prelude::vec::Vec, types::Address};

#[derive(Event, PartialEq, Eq, Debug)]
pub struct CreatePositionEvent {
    pub timestamp: u64,
    pub address: Address,
    pub pool: PoolKey,
    pub liquidity: Liquidity,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub current_sqrt_price: SqrtPrice,
}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct RemovePositionEvent {
    pub timestamp: u64,
    pub address: Address,
    pub pool: PoolKey,
    pub liquidity: Liquidity,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub current_sqrt_price: SqrtPrice,
}
#[derive(Event, PartialEq, Eq, Debug)]

pub struct CrossTickEvent {
    pub timestamp: u64,
    pub address: Address,
    pub pool: PoolKey,
    pub indexes: Vec<i32>,
}

#[derive(Event, PartialEq, Eq, Debug)]
pub struct SwapEvent {
    pub timestamp: u64,
    pub address: Address,
    pub pool: PoolKey,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee: TokenAmount,
    pub start_sqrt_price: SqrtPrice,
    pub target_sqrt_price: SqrtPrice,
    pub x_to_y: bool,
}
