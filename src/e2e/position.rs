use crate::contracts::{CreatePositionEvent, InvariantError, PoolKey, RemovePositionEvent};
use crate::e2e::snippets::init;
use crate::math::fee_growth::FeeGrowth;
use crate::math::liquidity::Liquidity;
use crate::math::sqrt_price::{calculate_sqrt_price, SqrtPrice};
use crate::math::token_amount::TokenAmount;
use crate::math::MIN_SQRT_PRICE;
use crate::{contracts::FeeTier, math::percentage::Percentage};
use decimal::{Decimal, Factories};
use odra::assert_events;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
fn test_create_position() {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let mint_amount = U256::from(500);
    let fee = Percentage::new(U128::from(0));
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
    invariant.add_fee_tier(fee_tier).unwrap();

    invariant
        .create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            calculate_sqrt_price(10).unwrap(),
            10,
        )
        .unwrap();

    token_x.approve(invariant.address(), &U256::from(500));
    token_y.approve(invariant.address(), &U256::from(500));

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();

    let lower_tick = -10;
    let upper_tick = 10;
    let liquidity_delta = Liquidity::new(U256::from(10));
    invariant
        .create_position(
            pool_key,
            lower_tick,
            upper_tick,
            liquidity_delta,
            SqrtPrice::new(U128::from(0)),
            SqrtPrice::max_instance(),
        )
        .unwrap();

    let pool = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    assert_events!(
        invariant,
        CreatePositionEvent {
            timestamp: 0,
            address: alice,
            pool: pool_key,
            liquidity: liquidity_delta,
            lower_tick,
            upper_tick,
            current_sqrt_price: pool.sqrt_price,
        }
    );
}

#[test]
fn test_remove_position() {
    let alice = test_env::get_account(0);
    let bob = test_env::get_account(1);
    test_env::set_caller(alice);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

    let init_tick = 0;
    let remove_position_index = 0;

    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();

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

    let lower_tick_index = -20;
    let upper_tick_index = 10;
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    token_x.approve(invariant.address(), &mint_amount);
    token_y.approve(invariant.address(), &mint_amount);

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
            pool_state.sqrt_price,
        )
        .unwrap();

    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    assert_events!(
        invariant,
        CreatePositionEvent {
            timestamp: 0,
            address: alice,
            pool: pool_key,
            liquidity: liquidity_delta,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            current_sqrt_price: pool_state.sqrt_price
        }
    );

    assert_eq!(pool_state.liquidity, liquidity_delta);
    let liquidity_delta = Liquidity::new(liquidity_delta.get() * 1_000_000);
    {
        let incorrect_lower_tick_index = lower_tick_index - 50;
        let incorrect_upper_tick_index = upper_tick_index + 50;

        token_x.approve(invariant.address(), &liquidity_delta.get());
        token_y.approve(invariant.address(), &liquidity_delta.get());

        invariant
            .create_position(
                pool_key,
                incorrect_lower_tick_index,
                incorrect_upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
            )
            .unwrap();

        let position_state = invariant.get_position(1).unwrap();

        assert_events!(
            invariant,
            CreatePositionEvent {
                timestamp: 0,
                address: alice,
                pool: pool_key,
                liquidity: liquidity_delta,
                lower_tick: incorrect_lower_tick_index,
                upper_tick: incorrect_upper_tick_index,
                current_sqrt_price: pool_state.sqrt_price,
            }
        );

        // Check position
        assert!(position_state.lower_tick_index == incorrect_lower_tick_index);
        assert!(position_state.upper_tick_index == incorrect_upper_tick_index);
    }

    let amount = 1000;
    token_x.mint(&bob, &U256::from(amount));
    let amount_x = token_x.balance_of(&bob);
    assert_eq!(amount_x, U256::from(amount));

    test_env::set_caller(bob);
    token_x.approve(invariant.address(), &U256::from(amount));

    let pool_state_before = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    let swap_amount = TokenAmount::new(U256::from(amount));
    let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
    invariant
        .swap(pool_key, true, swap_amount, true, slippage)
        .unwrap();

    let pool_state_after = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    assert_eq!(
        pool_state_after.fee_growth_global_x,
        FeeGrowth::new(U128::from(49999950000049999u64))
    );
    assert_eq!(
        pool_state_after.fee_protocol_token_x,
        TokenAmount::new(U256::from(1))
    );
    assert_eq!(
        pool_state_after.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );

    assert!(pool_state_after
        .sqrt_price
        .lt(&pool_state_before.sqrt_price));

    assert_eq!(pool_state_after.liquidity, pool_state_before.liquidity);
    assert_eq!(pool_state_after.current_tick_index, -10);
    assert_ne!(pool_state_after.sqrt_price, pool_state_before.sqrt_price);

    let amount_x = token_x.balance_of(&bob);
    let amount_y = token_y.balance_of(&bob);
    assert_eq!(amount_x, U256::from(0));
    assert_eq!(amount_y, U256::from(993));

    // pre load dex balances
    let dex_x_before_remove = token_x.balance_of(invariant.address());
    let dex_y_before_remove = token_y.balance_of(invariant.address());

    // Remove position
    test_env::set_caller(alice);
    invariant.remove_position(remove_position_index).unwrap();

    // Load states
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    assert_events!(
        invariant,
        RemovePositionEvent {
            timestamp: 0,
            address: alice,
            pool: pool_key,
            liquidity: Liquidity::from_integer(1_000_000),
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            current_sqrt_price: pool_state.sqrt_price,
        }
    );

    let lower_tick = invariant.get_tick(pool_key, lower_tick_index);
    let upper_tick = invariant.get_tick(pool_key, upper_tick_index);
    let lower_tick_bit = invariant.is_tick_initialized(pool_key, lower_tick_index);
    let upper_tick_bit = invariant.is_tick_initialized(pool_key, upper_tick_index);
    let dex_x = token_x.balance_of(invariant.address());
    let dex_y = token_y.balance_of(invariant.address());
    let expected_withdrawn_x = 499;
    let expected_withdrawn_y = 999;
    let expected_fee_x = 0;

    assert_eq!(
        dex_x_before_remove - dex_x,
        U256::from(expected_withdrawn_x) + expected_fee_x
    );
    assert_eq!(
        dex_y_before_remove - dex_y,
        U256::from(expected_withdrawn_y)
    );

    // Check ticks
    assert_eq!(lower_tick, Err(InvariantError::TickNotFound));
    assert_eq!(upper_tick, Err(InvariantError::TickNotFound));

    // Check tickmap
    assert!(!lower_tick_bit);
    assert!(!upper_tick_bit);

    // Check pool
    assert!(pool_state.liquidity == liquidity_delta);
    assert!(pool_state.current_tick_index == -10);
}

#[test]
fn test_position_within_current_tick() {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let max_tick_test = 177_450; // for tickSpacing 4
    let min_tick_test = -max_tick_test;
    let init_tick = -23028;

    let initial_balance = U256::from(100_000_000);
    let fee = Percentage::new(U128::from(0));
    let (mut invariant, mut token_x, mut token_y) = init(fee, initial_balance);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

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

    token_x.approve(invariant.address(), &initial_balance);
    token_y.approve(invariant.address(), &initial_balance);

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let lower_tick_index = min_tick_test + 10;
    let upper_tick_index = max_tick_test - 10;

    let liquidity_delta = Liquidity::from_integer(100);

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

    // Load states
    let position_state = invariant.get_position(0).unwrap();
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    assert_events!(
        invariant,
        CreatePositionEvent {
            timestamp: 0,
            address: alice,
            pool: pool_key,
            liquidity: liquidity_delta,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            current_sqrt_price: pool_state.sqrt_price,
        }
    );

    let lower_tick = invariant.get_tick(pool_key, lower_tick_index).unwrap();
    let upper_tick = invariant.get_tick(pool_key, upper_tick_index).unwrap();
    let alice_x = token_x.balance_of(&alice);
    let alice_y = token_y.balance_of(&alice);
    let dex_x = token_x.balance_of(invariant.address());
    let dex_y = token_y.balance_of(invariant.address());

    let zero_fee = FeeGrowth::new(U128::from(0));
    let expected_x_increase = 317;
    let expected_y_increase = 32;

    // Check ticks
    assert!(lower_tick.index == lower_tick_index);
    assert!(upper_tick.index == upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check pool
    assert!(pool_state.liquidity == liquidity_delta);
    assert!(pool_state.current_tick_index == init_tick);

    // Check position
    assert!(position_state.pool_key == pool_key);
    assert!(position_state.liquidity == liquidity_delta);
    assert!(position_state.lower_tick_index == lower_tick_index);
    assert!(position_state.upper_tick_index == upper_tick_index);
    assert!(position_state.fee_growth_inside_x == zero_fee);
    assert!(position_state.fee_growth_inside_y == zero_fee);

    // Check balances
    assert_eq!(alice_x, initial_balance.checked_sub(dex_x).unwrap());
    assert_eq!(alice_y, initial_balance.checked_sub(dex_y).unwrap());
    assert_eq!(dex_x, U256::from(expected_x_increase));
    assert_eq!(dex_y, U256::from(expected_y_increase));
}

#[test]
fn test_position_below_current_tick() {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let init_tick = -23028;

    let initial_balance = U256::from(10_000_000_000u64);
    let fee = Percentage::new(U128::from(0));
    let (mut invariant, mut token_x, mut token_y) = init(fee, initial_balance);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

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

    token_x.approve(invariant.address(), &initial_balance);
    token_y.approve(invariant.address(), &initial_balance);

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let lower_tick_index = -46080;
    let upper_tick_index = -23040;

    let liquidity_delta = Liquidity::from_integer(10000);

    let pool_state_before = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    invariant
        .create_position(
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            pool_state_before.sqrt_price,
            SqrtPrice::max_instance(),
        )
        .unwrap();

    // Load states
    let position_state = invariant.get_position(0).unwrap();
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    assert_events!(
        invariant,
        CreatePositionEvent {
            timestamp: 0,
            address: alice,
            pool: pool_key,
            liquidity: liquidity_delta,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            current_sqrt_price: pool_state.sqrt_price,
        }
    );

    let lower_tick = invariant.get_tick(pool_key, lower_tick_index).unwrap();
    let upper_tick = invariant.get_tick(pool_key, upper_tick_index).unwrap();
    let alice_x = token_x.balance_of(&alice);
    let alice_y = token_y.balance_of(&alice);
    let dex_x = token_x.balance_of(invariant.address());
    let dex_y = token_y.balance_of(invariant.address());

    let zero_fee = FeeGrowth::new(U128::from(0));
    let expected_x_increase = 0;
    let expected_y_increase = 2162;

    // Check ticks
    assert!(lower_tick.index == lower_tick_index);
    assert!(upper_tick.index == upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check pool
    assert!(pool_state.liquidity == pool_state_before.liquidity);
    assert!(pool_state.current_tick_index == init_tick);

    // Check position
    assert!(position_state.pool_key == pool_key);
    assert!(position_state.liquidity == liquidity_delta);
    assert!(position_state.lower_tick_index == lower_tick_index);
    assert!(position_state.upper_tick_index == upper_tick_index);
    assert!(position_state.fee_growth_inside_x == zero_fee);
    assert!(position_state.fee_growth_inside_y == zero_fee);

    // Check balances
    assert_eq!(alice_x, initial_balance.checked_sub(dex_x).unwrap());
    assert_eq!(alice_y, initial_balance.checked_sub(dex_y).unwrap());
    assert_eq!(dex_x, U256::from(expected_x_increase));
    assert_eq!(dex_y, U256::from(expected_y_increase));
}

#[test]
fn test_position_above_current_tick() {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let init_tick = -23028;

    let initial_balance = U256::from(10_000_000_000i64);
    let fee = Percentage::new(U128::from(0));
    let (mut invariant, mut token_x, mut token_y) = init(fee, initial_balance);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

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

    token_x.approve(invariant.address(), &initial_balance);
    token_y.approve(invariant.address(), &initial_balance);

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let lower_tick_index = -22980;
    let upper_tick_index = 0;
    let liquidity_delta = Liquidity::from_integer(1000);

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

    // Load states
    let position_state = invariant.get_position(0).unwrap();
    let pool_state = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();
    let lower_tick = invariant.get_tick(pool_key, lower_tick_index).unwrap();
    let upper_tick = invariant.get_tick(pool_key, upper_tick_index).unwrap();
    let alice_x = token_x.balance_of(&alice);
    let alice_y = token_y.balance_of(&alice);
    let dex_x = token_x.balance_of(invariant.address());
    let dex_y = token_y.balance_of(invariant.address());

    let zero_fee = FeeGrowth::new(U128::from(0));
    let expected_x_increase = 2155;
    let expected_y_increase = 0;

    // Check ticks
    assert!(lower_tick.index == lower_tick_index);
    assert!(upper_tick.index == upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check pool
    assert!(pool_state.liquidity == Liquidity::new(U256::from(0)));
    assert!(pool_state.current_tick_index == init_tick);

    // Check position
    assert!(position_state.pool_key == pool_key);
    assert!(position_state.liquidity == liquidity_delta);
    assert!(position_state.lower_tick_index == lower_tick_index);
    assert!(position_state.upper_tick_index == upper_tick_index);
    assert!(position_state.fee_growth_inside_x == zero_fee);
    assert!(position_state.fee_growth_inside_y == zero_fee);

    // Check balances
    assert_eq!(alice_x, initial_balance.checked_sub(dex_x).unwrap());
    assert_eq!(alice_y, initial_balance.checked_sub(dex_y).unwrap());
    assert_eq!(dex_x, U256::from(expected_x_increase));
    assert_eq!(dex_y, U256::from(expected_y_increase));
}
