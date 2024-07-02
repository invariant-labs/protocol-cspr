use crate::contracts::PoolKey;
use crate::e2e::snippets::init;
use crate::math::fee_growth::FeeGrowth;
use crate::math::liquidity::Liquidity;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::math::sqrt_price::SqrtPrice;
use crate::math::token_amount::TokenAmount;
use crate::math::MAX_SQRT_PRICE;
use crate::math::MIN_SQRT_PRICE;
use crate::FeeTier;
use decimal::*;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
pub fn test_swap() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant
            .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
            .unwrap();

        let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Init basic position
    {
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick,
                upper_tick,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(pool_after.liquidity, liquidity)
    }
    // Init basic swap
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(1000);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(500));
        assert_eq!(amount_y, U256::from(1000));

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                true,
                swap_amount.get(),
                true,
                slippage.get(),
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let expected_tick = -20;

        assert_eq!(pool_after.liquidity, pool_before.liquidity);
        assert_eq!(pool_after.current_tick_index, expected_tick);
        assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

        let amount_x = token_x.balance_of(&caller);
        let amount_y = token_y.balance_of(&caller);
        assert_eq!(amount_x, U256::from(0));
        assert_eq!(amount_y, U256::from(993));

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(1500));
        assert_eq!(amount_y, U256::from(7));

        assert_eq!(
            pool_after.fee_growth_global_x,
            FeeGrowth::new(U128::from(50000000000000000000000u128))
        );
        assert_eq!(
            pool_after.fee_growth_global_y,
            FeeGrowth::new(U128::from(0))
        );

        assert_eq!(
            pool_after.fee_protocol_token_x,
            TokenAmount::new(U256::from(1))
        );
        assert_eq!(
            pool_after.fee_protocol_token_y,
            TokenAmount::new(U256::from(0))
        );
    }
}

#[test]
fn test_swap_x_to_y() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(6, 3);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant
            .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
            .unwrap();

        let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Init basic position
    {
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let middle_tick = -10;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick,
                upper_tick,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick - 20,
                middle_tick,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(pool_after.liquidity, liquidity)
    }
    // Init basic swap
    {
        let swapper = test_env::get_account(1);
        let amount = U256::from(1000);
        token_x.mint(&swapper, &amount);

        test_env::set_caller(swapper);
        token_x.approve(invariant.address(), &amount);

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(500));
        assert_eq!(amount_y, U256::from(2499));

        let dex_x_before = token_x.balance_of(invariant.address());
        let dex_y_before = token_y.balance_of(invariant.address());
        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                true,
                swap_amount.get(),
                true,
                slippage.get(),
            )
            .unwrap();

        // Load states
        let pool = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();
        let lower_tick = invariant
            .get_tick(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                -20,
            )
            .unwrap();
        let middle_tick = invariant
            .get_tick(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                -10,
            )
            .unwrap();
        let upper_tick = invariant
            .get_tick(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                10,
            )
            .unwrap();
        let lower_tick_bit = invariant.is_tick_initialized(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            -20,
        );
        let middle_tick_bit = invariant.is_tick_initialized(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            -10,
        );
        let upper_tick_bit = invariant.is_tick_initialized(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            10,
        );
        let swapper_x = token_x.balance_of(&swapper);
        let swapper_y = token_y.balance_of(&swapper);
        let dex_x = token_x.balance_of(invariant.address());
        let dex_y = token_y.balance_of(invariant.address());
        let delta_dex_x = dex_x - dex_x_before;
        let delta_dex_y = dex_y_before - dex_y;
        let expected_y = amount - U256::from(10);
        let expected_x = U256::from(0);
        let liquidity_delta = Liquidity::from_integer(1000000);

        // Check balances
        assert_eq!(swapper_x, expected_x);
        assert_eq!(swapper_y, expected_y);
        assert_eq!(delta_dex_x, amount);
        assert_eq!(delta_dex_y, expected_y);

        // Check Pool
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(U128::from(0)));
        assert_eq!(
            pool.fee_growth_global_x,
            FeeGrowth::new(U128::from(40000000000000000000000u128))
        );
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(0)));
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(2)));

        // Check Ticks
        assert_eq!(lower_tick.liquidity_change, liquidity_delta);
        assert_eq!(middle_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.liquidity_change, liquidity_delta);
        assert_eq!(
            upper_tick.fee_growth_outside_x,
            FeeGrowth::new(U128::from(0))
        );
        assert_eq!(
            middle_tick.fee_growth_outside_x,
            FeeGrowth::new(U128::from(30000000000000000000000u128))
        );
        assert_eq!(
            lower_tick.fee_growth_outside_x,
            FeeGrowth::new(U128::from(0))
        );
        assert!(lower_tick_bit);
        assert!(middle_tick_bit);
        assert!(upper_tick_bit);
    }
}

#[test]
fn test_swap_y_to_x() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(6, 3);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant
            .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
            .unwrap();

        let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Init basic position
    {
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick_index = -10;
        let middle_tick_index = 10;
        let upper_tick_index = 20;

        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick_index,
                upper_tick_index,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                middle_tick_index,
                upper_tick_index + 20,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(pool_after.liquidity, liquidity)
    }
    // Init basic swap
    {
        let swapper = test_env::get_account(1);
        let amount = U256::from(1000);
        token_y.mint(&swapper, &amount);

        test_env::set_caller(swapper);
        token_y.approve(invariant.address(), &amount);

        let dex_x_before = token_x.balance_of(invariant.address());
        let dex_y_before = token_y.balance_of(invariant.address());
        let slippage = SqrtPrice::new(U128::from(MAX_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                false,
                swap_amount.get(),
                true,
                slippage.get(),
            )
            .unwrap();

        // Load states
        let pool = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();
        let lower_tick = invariant
            .get_tick(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                -10,
            )
            .unwrap();
        let middle_tick = invariant
            .get_tick(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                10,
            )
            .unwrap();
        let upper_tick = invariant
            .get_tick(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                20,
            )
            .unwrap();
        let lower_tick_bit = invariant.is_tick_initialized(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            -10,
        );
        let middle_tick_bit = invariant.is_tick_initialized(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            10,
        );
        let upper_tick_bit = invariant.is_tick_initialized(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            20,
        );
        let swapper_x = token_x.balance_of(&swapper);
        let swapper_y = token_y.balance_of(&swapper);
        let dex_x = token_x.balance_of(invariant.address());
        let dex_y = token_y.balance_of(invariant.address());
        let delta_dex_x = dex_x_before - dex_x;
        let delta_dex_y = dex_y - dex_y_before;
        let expected_x = amount - U256::from(10);
        let expected_y = U256::from(0);
        let liquidity_delta = Liquidity::from_integer(1000000);

        // Check balances
        assert_eq!(swapper_x, expected_x);
        assert_eq!(swapper_y, expected_y);
        assert_eq!(delta_dex_x, expected_x);
        assert_eq!(delta_dex_y, amount);

        // Check Pool
        assert_eq!(pool.fee_growth_global_x, FeeGrowth::new(U128::from(0)));
        assert_eq!(
            pool.fee_growth_global_y,
            FeeGrowth::new(U128::from(40000000000000000000000u128))
        );
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(0)));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(2)));

        // Check Ticks
        assert_eq!(lower_tick.liquidity_change, liquidity_delta);
        assert_eq!(middle_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.liquidity_change, liquidity_delta);
        assert_eq!(
            upper_tick.fee_growth_outside_y,
            FeeGrowth::new(U128::from(0))
        );
        assert_eq!(
            middle_tick.fee_growth_outside_y,
            FeeGrowth::new(U128::from(30000000000000000000000u128))
        );
        assert_eq!(
            lower_tick.fee_growth_outside_y,
            FeeGrowth::new(U128::from(0))
        );
        assert!(lower_tick_bit);
        assert!(middle_tick_bit);
        assert!(upper_tick_bit);
    }
}

#[test]
#[should_panic]
fn test_swap_not_enough_liquidity_token_y() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(6, 3);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant
            .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
            .unwrap();

        let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Init basic position
    {
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let middle_tick = -10;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick,
                upper_tick,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick - 20,
                middle_tick,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(pool_after.liquidity, liquidity)
    }
    // Init basic swap
    {
        let swapper = test_env::get_account(1);
        let amount = U256::from(1000);
        token_y.mint(&swapper, &amount);

        test_env::set_caller(swapper);
        token_y.approve(invariant.address(), &amount);

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(500));
        assert_eq!(amount_y, U256::from(2499));

        let slippage = SqrtPrice::new(U128::from(MAX_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                false,
                swap_amount.get(),
                true,
                slippage.get(),
            )
            .unwrap();
    }
}

#[test]
#[should_panic]
fn test_swap_not_enough_liquidity_token_x() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(6, 3);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant
            .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
            .unwrap();

        let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Init basic position
    {
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick_index = -10;
        let middle_tick_index = 10;
        let upper_tick_index = 20;

        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick_index,
                upper_tick_index,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                middle_tick_index,
                upper_tick_index + 20,
                liquidity.get(),
                slippage_limit_lower.get(),
                slippage_limit_upper.get(),
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(pool_after.liquidity, liquidity)
    }
    // Init basic swap
    {
        let swapper = test_env::get_account(1);
        let amount = U256::from(1000);
        token_x.mint(&swapper, &amount);

        test_env::set_caller(swapper);
        token_x.approve(invariant.address(), &amount);

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                true,
                swap_amount.get(),
                true,
                slippage.get(),
            )
            .unwrap();
    }
}
