use crate::contracts::Position;
use crate::math::percentage::Percentage;
use crate::{
    token::{TokenDeployer, TokenRef},
    InvariantDeployer, InvariantRef,
};
use alloc::string::String;
use decimal::*;
use odra::types::U256;

pub fn init(fee: Percentage, supply: U256) -> (InvariantRef, TokenRef, TokenRef) {
    let invariant = InvariantDeployer::init(fee.get());
    let token_0 = TokenDeployer::init(String::from(""), String::from(""), 0, &supply);
    let token_1 = TokenDeployer::init(String::from(""), String::from(""), 0, &supply);
    if token_0.address() < token_1.address() {
        (invariant, token_0, token_1)
    } else {
        (invariant, token_1, token_0)
    }
}

pub fn positions_equals(position_a: Position, position_b: Position) -> bool {
    let mut equal = true;

    if position_a.fee_growth_inside_x != position_b.fee_growth_inside_x {
        equal = false;
    };

    if position_a.fee_growth_inside_y != position_b.fee_growth_inside_y {
        equal = false;
    };

    if position_a.liquidity != position_b.liquidity {
        equal = false;
    };

    if position_a.lower_tick_index != position_b.lower_tick_index {
        equal = false;
    };

    if position_a.upper_tick_index != position_b.upper_tick_index {
        equal = false;
    };

    if position_a.pool_key != position_b.pool_key {
        equal = false;
    };

    if position_a.tokens_owed_x != position_b.tokens_owed_x {
        equal = false;
    };

    if position_a.tokens_owed_y != position_b.tokens_owed_y {
        equal = false;
    };

    equal
}
