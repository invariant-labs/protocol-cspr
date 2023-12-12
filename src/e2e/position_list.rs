use crate::contracts::{FeeTier, InvariantError};
use crate::math::percentage::Percentage;
use crate::token::TokenDeployer;
use crate::InvariantDeployer;
use decimal::Factories;
use odra::prelude::string::String;
use odra::test_env;
use odra::types::U256;

#[test]
fn test_remove_position_from_empty_list() {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let initial_amount = 10u128.pow(10);
    let token_x = TokenDeployer::init(
        String::from(""),
        String::from(""),
        0,
        &U256::from(initial_amount),
    );
    let token_y = TokenDeployer::init(
        String::from(""),
        String::from(""),
        0,
        &U256::from(initial_amount),
    );
    let mut invariant = InvariantDeployer::init(Percentage::from_scale(6, 3));

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 3).unwrap();

    invariant.add_fee_tier(fee_tier).unwrap();

    let init_tick = -23028;

    invariant
        .create_pool(*token_x.address(), *token_y.address(), fee_tier, init_tick)
        .unwrap();

    let result = invariant.remove_position(0);
    assert_eq!(result, Err(InvariantError::PositionNotFound));
}
