use crate::contracts::{FeeTier, InvariantError, PoolKey, Position};
use crate::math::fee_growth::FeeGrowth;
use crate::math::liquidity::Liquidity;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::{calculate_sqrt_price, SqrtPrice};
use crate::token::TokenDeployer;
use crate::InvariantDeployer;
use decimal::{Decimal, Factories};
use odra::prelude::string::String;
use odra::test_env;
use odra::types::{U128, U256};

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

#[test]
fn test_remove_position_from_empty_list() {
    let user_without_any_positions: odra::types::Address = test_env::get_account(0);
    test_env::set_caller(user_without_any_positions);

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
        .create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            calculate_sqrt_price(init_tick).unwrap(),
            init_tick,
        )
        .unwrap();

    let result = invariant.remove_position(0);
    assert_eq!(result, Err(InvariantError::PositionNotFound));
}

#[test]
fn test_add_multiple_positions() {
    let positions_owner = test_env::get_account(0);
    test_env::set_caller(positions_owner);

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
        .create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            calculate_sqrt_price(init_tick).unwrap(),
            init_tick,
        )
        .unwrap();

    token_x.approve(invariant.address(), &U256::from(initial_balance));
    token_y.approve(invariant.address(), &U256::from(initial_balance));

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    // Open four positions
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

#[test]
fn test_only_owner_can_modify_position_list() {
    let positions_owner = test_env::get_account(0);
    test_env::set_caller(positions_owner);

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
        .create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            calculate_sqrt_price(init_tick).unwrap(),
            init_tick,
        )
        .unwrap();

    token_x.approve(invariant.address(), &U256::from(initial_balance));
    token_y.approve(invariant.address(), &U256::from(initial_balance));

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    // Open four positions
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
        assert!(positions_equals(last_position, tested_position));
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

        let unauthorized_user = test_env::get_account(1);
        test_env::set_caller(unauthorized_user);
        let result = invariant.remove_position(last_position_index_before as u32);
        assert_eq!(result, Err(InvariantError::PositionNotFound));
    }
}

#[test]
fn test_transfer_position_ownership() {
    let positions_owner = test_env::get_account(0);
    test_env::set_caller(positions_owner);

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
        .create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            calculate_sqrt_price(init_tick).unwrap(),
            init_tick,
        )
        .unwrap();

    token_x.approve(invariant.address(), &U256::from(initial_balance));
    token_y.approve(invariant.address(), &U256::from(initial_balance));

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();
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
        let list_length = invariant.get_all_positions().len();

        assert_eq!(list_length, 1)
    }

    let recipient = test_env::get_account(1);
    // Open  additional positions
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
                tick_indexes[1],
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
                tick_indexes[3],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();
    }
    // Transfer first position
    {
        let transferred_index = 0;
        let owner_list_before = invariant.get_all_positions();
        test_env::set_caller(recipient);
        let recipient_list_before = invariant.get_all_positions();
        test_env::set_caller(positions_owner);
        let removed_position = invariant.get_position(transferred_index).unwrap();
        let last_position_before = owner_list_before[owner_list_before.len() - 1];

        invariant
            .transfer_position(transferred_index, recipient)
            .unwrap();

        test_env::set_caller(recipient);
        let recipient_position = invariant.get_position(transferred_index).unwrap();
        let recipient_list_after = invariant.get_all_positions();
        test_env::set_caller(positions_owner);
        let owner_first_position_after = invariant.get_position(transferred_index).unwrap();
        let owner_list_after = invariant.get_all_positions();

        assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
        assert_eq!(owner_list_before.len() - 1, owner_list_after.len());

        // move last position
        assert!(positions_equals(
            owner_first_position_after,
            last_position_before
        ));

        // Equals fields od transferred position
        assert!(positions_equals(recipient_position, removed_position));
    }

    // Transfer middle position
    {
        let transferred_index = 1;
        let owner_list_before = invariant.get_all_positions();
        test_env::set_caller(recipient);
        let recipient_list_before = invariant.get_all_positions();
        let last_position_before = owner_list_before[owner_list_before.len() - 1];

        test_env::set_caller(positions_owner);
        invariant
            .transfer_position(transferred_index, recipient)
            .unwrap();

        let owner_list_after = invariant.get_all_positions();
        test_env::set_caller(recipient);
        let recipient_list_after = invariant.get_all_positions();
        test_env::set_caller(positions_owner);
        let owner_first_position_after = invariant.get_position(transferred_index).unwrap();

        assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
        assert_eq!(owner_list_before.len() - 1, owner_list_after.len());

        // move last position
        assert!(positions_equals(
            owner_first_position_after,
            last_position_before
        ));
    }
    // Transfer last position
    {
        let owner_list_before = invariant.get_all_positions();
        let transferred_index = (owner_list_before.len() - 1) as u32;
        let removed_position = invariant.get_position(transferred_index).unwrap();

        invariant
            .transfer_position(transferred_index, recipient)
            .unwrap();

        test_env::set_caller(recipient);
        let recipient_list_after = invariant.get_all_positions();
        let recipient_position_index = (recipient_list_after.len() - 1) as u32;
        let recipient_position = invariant.get_position(recipient_position_index).unwrap();

        assert!(positions_equals(removed_position, recipient_position));
    }

    // Clear position
    {
        let transferred_index = 0;
        let recipient_list_before = invariant.get_all_positions();
        test_env::set_caller(positions_owner);
        let removed_position = invariant.get_position(transferred_index).unwrap();

        invariant
            .transfer_position(transferred_index, recipient)
            .unwrap();

        test_env::set_caller(recipient);
        let recipient_list_after = invariant.get_all_positions();
        let recipient_position_index = (recipient_list_after.len() - 1) as u32;
        let recipient_position = invariant.get_position(recipient_position_index).unwrap();
        test_env::set_caller(positions_owner);
        let owner_list_after = invariant.get_all_positions();

        assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
        assert_eq!(0, owner_list_after.len());

        // Equals fields od transferred position
        assert!(positions_equals(recipient_position, removed_position));
    }

    // Get back position
    {
        let transferred_index = 0;
        let owner_list_before = invariant.get_all_positions();
        test_env::set_caller(recipient);
        let recipient_list_before = invariant.get_all_positions();
        let removed_position = invariant.get_position(transferred_index).unwrap();
        let last_position_before = recipient_list_before[recipient_list_before.len() - 1];

        invariant
            .transfer_position(transferred_index, positions_owner)
            .unwrap();

        test_env::set_caller(positions_owner);
        let owner_list_after = invariant.get_all_positions();
        test_env::set_caller(recipient);
        let recipient_list_after = invariant.get_all_positions();
        let recipient_first_position_after = invariant.get_position(transferred_index).unwrap();

        test_env::set_caller(positions_owner);
        let owner_new_position = invariant.get_position(transferred_index).unwrap();

        assert_eq!(recipient_list_after.len(), recipient_list_before.len() - 1);
        assert_eq!(owner_list_before.len() + 1, owner_list_after.len());

        // move last position
        assert!(positions_equals(
            last_position_before,
            recipient_first_position_after
        ));

        // Equals fields od transferred position
        assert!(positions_equals(owner_new_position, removed_position));
    }
}

#[test]
fn test_only_owner_can_transfer_position() {
    let position_owner = test_env::get_account(0);
    test_env::set_caller(position_owner);

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
        .create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            calculate_sqrt_price(init_tick).unwrap(),
            init_tick,
        )
        .unwrap();

    token_x.approve(invariant.address(), &U256::from(initial_balance));
    token_y.approve(invariant.address(), &U256::from(initial_balance));

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();
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
        let list_length = invariant.get_all_positions().len();

        assert_eq!(list_length, 1)
    }

    // Open  additional positions
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
                tick_indexes[1],
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
                tick_indexes[3],
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();
    }
    // Transfer first position
    {
        let transferred_index = 0;
        let unauthorized_user = test_env::get_account(1);
        test_env::set_caller(unauthorized_user);
        let result = invariant.transfer_position(transferred_index, position_owner);
        assert_eq!(result, Err(InvariantError::PositionNotFound));
    }
}

#[test]
fn test_multiple_positions_on_same_tick() {
    let positions_owner = test_env::get_account(0);
    test_env::set_caller(positions_owner);

    let init_tick = 0;

    let initial_balance = 100_000_000;
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

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 10).unwrap();

    invariant.add_fee_tier(fee_tier).unwrap();

    invariant
        .create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            calculate_sqrt_price(init_tick).unwrap(),
            init_tick,
        )
        .unwrap();

    token_x.approve(invariant.address(), &U256::from(initial_balance));
    token_y.approve(invariant.address(), &U256::from(initial_balance));

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Three position on same lower and upper tick
    {
        let lower_tick_index = -10;
        let upper_tick_index = 10;

        let liquidity_delta = Liquidity::new(U256::from(100));

        let pool_state = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();

        invariant
            .create_position(
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        let first_position = invariant.get_position(0).unwrap();

        invariant
            .create_position(
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        let second_position = invariant.get_position(1).unwrap();

        invariant
            .create_position(
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        let third_position = invariant.get_position(2).unwrap();

        assert!(first_position.lower_tick_index == second_position.lower_tick_index);
        assert!(first_position.upper_tick_index == second_position.upper_tick_index);
        assert!(first_position.lower_tick_index == third_position.lower_tick_index);
        assert!(first_position.upper_tick_index == third_position.upper_tick_index);

        // Load states
        let pool_state = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();
        let lower_tick = invariant.get_tick(pool_key, lower_tick_index).unwrap();
        let upper_tick = invariant.get_tick(pool_key, upper_tick_index).unwrap();
        let expected_liquidity = Liquidity::new(liquidity_delta.get() * 3);
        let zero_fee = FeeGrowth::new(U128::from(0));

        // Check ticks
        assert!(lower_tick.index == lower_tick_index);
        assert!(upper_tick.index == upper_tick_index);
        assert_eq!(lower_tick.liquidity_gross, expected_liquidity);
        assert_eq!(upper_tick.liquidity_gross, expected_liquidity);
        assert_eq!(lower_tick.liquidity_change, expected_liquidity);
        assert_eq!(upper_tick.liquidity_change, expected_liquidity);
        assert!(lower_tick.sign);
        assert!(!upper_tick.sign);

        // Check pool
        assert_eq!(pool_state.liquidity, expected_liquidity);
        assert!(pool_state.current_tick_index == init_tick);

        // Check first position
        assert!(first_position.pool_key == pool_key);
        assert!(first_position.liquidity == liquidity_delta);
        assert!(first_position.lower_tick_index == lower_tick_index);
        assert!(first_position.upper_tick_index == upper_tick_index);
        assert!(first_position.fee_growth_inside_x == zero_fee);
        assert!(first_position.fee_growth_inside_y == zero_fee);

        // Check second position
        assert!(second_position.pool_key == pool_key);
        assert!(second_position.liquidity == liquidity_delta);
        assert!(second_position.lower_tick_index == lower_tick_index);
        assert!(second_position.upper_tick_index == upper_tick_index);
        assert!(second_position.fee_growth_inside_x == zero_fee);
        assert!(second_position.fee_growth_inside_y == zero_fee);

        // Check third position
        assert!(third_position.pool_key == pool_key);
        assert!(third_position.liquidity == liquidity_delta);
        assert!(third_position.lower_tick_index == lower_tick_index);
        assert!(third_position.upper_tick_index == upper_tick_index);
        assert!(third_position.fee_growth_inside_x == zero_fee);
        assert!(third_position.fee_growth_inside_y == zero_fee);
    }
    {
        let lower_tick_index = -10;
        let upper_tick_index = 10;
        let zero_fee = FeeGrowth::new(U128::from(0));

        let liquidity_delta = Liquidity::new(U256::from(100));

        let pool_state = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();

        invariant
            .create_position(
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        let first_position = invariant.get_position(3).unwrap();

        // Check first position
        assert!(first_position.pool_key == pool_key);
        assert!(first_position.liquidity == liquidity_delta);
        assert!(first_position.lower_tick_index == lower_tick_index);
        assert!(first_position.upper_tick_index == upper_tick_index);
        assert!(first_position.fee_growth_inside_x == zero_fee);
        assert!(first_position.fee_growth_inside_y == zero_fee);

        let lower_tick_index = -20;
        let upper_tick_index = -10;

        invariant
            .create_position(
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        let second_position = invariant.get_position(4).unwrap();

        // Check second position
        assert!(second_position.pool_key == pool_key);
        assert!(second_position.liquidity == liquidity_delta);
        assert!(second_position.lower_tick_index == lower_tick_index);
        assert!(second_position.upper_tick_index == upper_tick_index);
        assert!(second_position.fee_growth_inside_x == zero_fee);
        assert!(second_position.fee_growth_inside_y == zero_fee);

        let lower_tick_index = 10;
        let upper_tick_index = 20;
        invariant
            .create_position(
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        let third_position = invariant.get_position(5).unwrap();

        // Check third position
        assert!(third_position.pool_key == pool_key);
        assert!(third_position.liquidity == liquidity_delta);
        assert!(third_position.lower_tick_index == lower_tick_index);
        assert!(third_position.upper_tick_index == upper_tick_index);
        assert!(third_position.fee_growth_inside_x == zero_fee);
        assert!(third_position.fee_growth_inside_y == zero_fee);

        // Load states
        let pool_state = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();
        let tick_n20 = invariant.get_tick(pool_key, -20).unwrap();
        let tick_n10 = invariant.get_tick(pool_key, -10).unwrap();
        let tick_10 = invariant.get_tick(pool_key, 10).unwrap();
        let tick_20 = invariant.get_tick(pool_key, 20).unwrap();
        let tick_n20_bit = invariant.is_tick_initialized(pool_key, -20);
        let tick_n10_bit = invariant.is_tick_initialized(pool_key, -10);
        let tick_20_bit = invariant.is_tick_initialized(pool_key, 20);

        let expected_active_liquidity = Liquidity::new(U256::from(400));

        // Check tick -20
        assert_eq!(tick_n20.index, -20);
        assert_eq!(tick_n20.liquidity_gross, Liquidity::new(U256::from(100)));
        assert_eq!(tick_n20.liquidity_change, Liquidity::new(U256::from(100)));
        assert!(tick_n20.sign);
        assert!(tick_n20_bit);

        // Check tick -10
        assert_eq!(tick_n10.index, -10);
        assert_eq!(tick_n10.liquidity_gross, Liquidity::new(U256::from(500)));
        assert_eq!(tick_n10.liquidity_change, Liquidity::new(U256::from(300)));
        assert!(tick_n10.sign);
        assert!(tick_n10_bit);

        // Check tick 10
        assert_eq!(tick_10.index, 10);
        assert_eq!(tick_10.liquidity_gross, Liquidity::new(U256::from(500)));
        assert_eq!(tick_10.liquidity_change, Liquidity::new(U256::from(300)));
        assert!(!tick_10.sign);
        assert!(tick_20_bit);

        // Check tick 20
        assert_eq!(tick_20.index, 20);
        assert_eq!(tick_20.liquidity_gross, Liquidity::new(U256::from(100)));
        assert_eq!(tick_20.liquidity_change, Liquidity::new(U256::from(100)));
        assert!(!tick_20.sign);
        assert!(tick_20_bit);

        // Check pool
        assert_eq!(pool_state.liquidity, expected_active_liquidity);
        assert!(pool_state.current_tick_index == init_tick);
    }
}
