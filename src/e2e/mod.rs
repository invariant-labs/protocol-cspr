pub mod add_fee_tier;
pub mod change_fee_receiver;
pub mod change_protocol_fee;
pub mod constructor;
pub mod create_pool;
pub mod position;
pub mod position_list;
pub mod protocol_fee;
pub mod remove_fee_tier;

use crate::contracts::Position;

fn positions_equals(position_a: Position, position_b: Position) -> bool {
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
