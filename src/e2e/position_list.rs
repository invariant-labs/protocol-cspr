use crate::contracts::{FeeTier, InvariantError, PoolKey};
use crate::math::liquidity::Liquidity;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::SqrtPrice;
use crate::token::TokenDeployer;
use crate::InvariantDeployer;
use decimal::{Decimal, Factories};
use odra::prelude::string::String;
use odra::test_env;
use odra::types::{U128, U256};

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

#[test]
fn test_add_multiple_positions() {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let init_tick = -23028;

    let initial_balance = 10u128.pow(10);
    let mut token_x = TokenDeployer::init(
        String::from(""),
        String::from(""),
        0,
        &U256::from(initial_balance),
    );
    let mut token_y = TokenDeployer::init(
        String::from(""),
        String::from(""),
        0,
        &U256::from(initial_balance),
    );
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

    invariant.add_fee_tier(fee_tier).unwrap();

    invariant
        .create_pool(*token_x.address(), *token_y.address(), fee_tier, init_tick)
        .unwrap();

    token_x.approve(invariant.address(), &U256::from(initial_balance));
    token_y.approve(invariant.address(), &U256::from(initial_balance));

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    // Open three positions
    {
        invariant
            .create_position(
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        invariant
            .create_position(
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        invariant
            .create_position(
                pool_key,
                tick_indexes[0],
                tick_indexes[2],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        invariant
            .create_position(
                pool_key,
                tick_indexes[1],
                tick_indexes[4],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();
    }

    // Remove middle position
    {
        let position_index_to_remove = 2;
        let positions_list_before = invariant.get_all_positions();
        let last_position = positions_list_before[positions_list_before.len() - 1];

        invariant.remove_position(position_index_to_remove).unwrap();

        let positions_list_after = invariant.get_all_positions();
        let tested_position = positions_list_after[position_index_to_remove as usize];

        // Last position should be at removed index
        assert_eq!(last_position.pool_key, tested_position.pool_key);
        assert_eq!(last_position.liquidity, tested_position.liquidity);
        assert_eq!(
            last_position.lower_tick_index,
            tested_position.lower_tick_index
        );
        assert_eq!(
            last_position.upper_tick_index,
            tested_position.upper_tick_index
        );
        assert_eq!(
            last_position.fee_growth_inside_x,
            tested_position.fee_growth_inside_x
        );
        assert_eq!(
            last_position.fee_growth_inside_y,
            tested_position.fee_growth_inside_y
        );
        assert_eq!(last_position.tokens_owed_x, tested_position.tokens_owed_x);
        assert_eq!(last_position.tokens_owed_y, tested_position.tokens_owed_y);
    }
    // Add position in place of the removed one
    {
        let positions_list_before = invariant.get_all_positions();

        invariant
            .create_position(
                pool_key,
                tick_indexes[1],
                tick_indexes[2],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        let positions_list_after = invariant.get_all_positions();
        assert_eq!(positions_list_before.len() + 1, positions_list_after.len());
    }
    // Remove last position
    {
        let last_position_index_before = invariant.get_all_positions().len() - 1;

        invariant
            .remove_position(last_position_index_before as u32)
            .unwrap();

        let last_position_index_after = invariant.get_all_positions().len() - 1;

        assert_eq!(last_position_index_before - 1, last_position_index_after)
    }
    // Remove all positions
    {
        let last_position_index = invariant.get_all_positions().len();

        for i in (0..last_position_index).rev() {
            invariant.remove_position(i as u32).unwrap();
        }

        let list_length = invariant.get_all_positions().len();
        assert_eq!(list_length, 0);
    }
    // Add position to cleared list
    {
        let list_length_before = invariant.get_all_positions().len();

        invariant
            .create_position(
                pool_key,
                tick_indexes[0],
                tick_indexes[1],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();
        let list_length_after = invariant.get_all_positions().len();
        assert_eq!(list_length_after, list_length_before + 1);
    };
}
