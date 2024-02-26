use crate::contracts::InvariantError;
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
use crate::Erc20Deployer;
use crate::FeeTier;
use crate::InvariantDeployer;
use alloc::string::String;
use decimal::*;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
fn test_cross_both_side() {
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
            .add_fee_tier(fee_tier.fee.v, fee_tier.tick_spacing)
            .unwrap();
        let exist = invariant.fee_tier_exist(fee_tier);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
            )
            .unwrap();
    }
    // Init basic position
    {
        let mint_amount = U256::from(10u128.pow(5));

        token_x.mint(&deployer, &mint_amount);
        token_y.mint(&deployer, &mint_amount);

        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let middle_tick = -10;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(20006000);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key,
                middle_tick,
                upper_tick,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        invariant
            .create_position(
                pool_key,
                lower_tick,
                middle_tick,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        assert_eq!(pool_after.liquidity, liquidity)
    }

    let limit_without_cross_tick_amount = TokenAmount::new(U256::from(10_068));
    let not_cross_amount = TokenAmount::new(U256::from(1));
    let min_amount_to_cross_from_tick_price = TokenAmount::new(U256::from(3));
    let crossing_amount_by_amount_out = TokenAmount::new(U256::from(20136101434u128));

    let mint_amount = limit_without_cross_tick_amount.get()
        + not_cross_amount.get()
        + min_amount_to_cross_from_tick_price.get()
        + crossing_amount_by_amount_out.get();

    token_x.mint(&deployer, &mint_amount);
    token_y.mint(&deployer, &mint_amount);

    token_x.approve(invariant.address(), &mint_amount);
    token_y.approve(invariant.address(), &mint_amount);

    {
        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let sqrt_price_limit = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        invariant
            .swap(
                pool_key,
                true,
                limit_without_cross_tick_amount,
                true,
                sqrt_price_limit,
            )
            .unwrap();

        let pool = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let expected_tick = -10;
        let expected_sqrt_price = calculate_sqrt_price(expected_tick).unwrap();

        assert_eq!(pool.current_tick_index, expected_tick);
        assert_eq!(pool.sqrt_price, expected_sqrt_price);
        assert_eq!(pool.liquidity, pool_before.liquidity);
    }

    {
        let sqrt_price_limit = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        invariant
            .swap(
                pool_key,
                true,
                min_amount_to_cross_from_tick_price,
                true,
                sqrt_price_limit,
            )
            .unwrap();

        let sqrt_price_limit = SqrtPrice::new(U128::from(MAX_SQRT_PRICE));

        invariant
            .swap(
                pool_key,
                false,
                min_amount_to_cross_from_tick_price,
                true,
                sqrt_price_limit,
            )
            .unwrap();
    }

    {
        let massive_x = U256::from(10u128.pow(19));
        let massive_y = U256::from(10u128.pow(19));
        token_x.mint(&deployer, &massive_x);
        token_y.mint(&deployer, &massive_y);

        token_x.approve(invariant.address(), &massive_x);
        token_y.approve(invariant.address(), &massive_y);

        let massive_liquidity_delta = Liquidity::from_integer(19996000399699881985603u128);

        invariant
            .create_position(
                pool_key,
                -20,
                0,
                massive_liquidity_delta,
                SqrtPrice::new(U128::from(MIN_SQRT_PRICE)),
                SqrtPrice::new(U128::from(MAX_SQRT_PRICE)),
            )
            .unwrap();
    }

    {
        token_x.approve(invariant.address(), &U256::from(3));
        token_y.approve(invariant.address(), &U256::from(2));

        invariant
            .swap(
                pool_key,
                true,
                TokenAmount::new(U256::from(1)),
                false,
                SqrtPrice::new(U128::from(MIN_SQRT_PRICE)),
            )
            .unwrap();
        invariant
            .swap(
                pool_key,
                false,
                TokenAmount::new(U256::from(2)),
                true,
                SqrtPrice::new(U128::from(MAX_SQRT_PRICE)),
            )
            .unwrap();

        let pool = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let expected_liquidity = Liquidity::from_integer(19996000399699901991603u128);
        let expected_liquidity_change_on_last_tick =
            Liquidity::from_integer(19996000399699901991603u128);
        let expected_liquidity_change_on_upper_tick = Liquidity::from_integer(20006000);

        assert_eq!(pool.current_tick_index, -20);
        assert_eq!(
            pool.fee_growth_global_x,
            FeeGrowth::new(U128::from(29991002699190242927121u128))
        );
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(U128::from(0)));
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(4)),);
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(2)),);

        assert_eq!(pool.liquidity, expected_liquidity);
        assert_eq!(
            pool.sqrt_price,
            SqrtPrice::new(U128::from(999500149964999999999999u128))
        );

        let final_last_tick = invariant.get_tick(pool_key, -20).unwrap();
        assert_eq!(
            final_last_tick.fee_growth_outside_x,
            FeeGrowth::new(U128::from(0))
        );
        assert_eq!(
            final_last_tick.fee_growth_outside_y,
            FeeGrowth::new(U128::from(0))
        );

        assert_eq!(
            final_last_tick.liquidity_change,
            expected_liquidity_change_on_last_tick
        );

        let final_lower_tick = invariant.get_tick(pool_key, -10).unwrap();
        assert_eq!(
            final_lower_tick.fee_growth_outside_x,
            FeeGrowth::new(U128::from(29991002699190242927121u128))
        );
        assert_eq!(
            final_lower_tick.fee_growth_outside_y,
            FeeGrowth::new(U128::from(0))
        );
        assert_eq!(
            final_lower_tick.liquidity_change,
            Liquidity::new(U256::from(0))
        );

        let final_upper_tick = invariant.get_tick(pool_key, 10).unwrap();
        assert_eq!(
            final_upper_tick.fee_growth_outside_x,
            FeeGrowth::new(U128::from(0))
        );
        assert_eq!(
            final_upper_tick.fee_growth_outside_y,
            FeeGrowth::new(U128::from(0))
        );

        assert_eq!(
            final_upper_tick.liquidity_change,
            expected_liquidity_change_on_upper_tick
        );
    }
}

#[test]
fn test_cross_both_side_not_cross_case() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = &Some(U256::from(10u128.pow(10)));
    let mut token_x = Erc20Deployer::init(String::from(""), String::from(""), 0, &mint_amount);
    let mut token_y = Erc20Deployer::init(String::from(""), String::from(""), 0, &mint_amount);
    let mut invariant = InvariantDeployer::init(Percentage::from_scale(1, 2).get());
    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant
            .add_fee_tier(fee_tier.fee.v, fee_tier.tick_spacing)
            .unwrap();
        let exist = invariant.fee_tier_exist(fee_tier);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
            )
            .unwrap();
    }
    // Init basic position
    {
        let mint_amount = U256::from(10u128.pow(5));

        token_x.mint(&deployer, &mint_amount);
        token_y.mint(&deployer, &mint_amount);

        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let middle_tick = -10;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(20006000);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key,
                middle_tick,
                upper_tick,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        invariant
            .create_position(
                pool_key,
                lower_tick,
                middle_tick,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        assert_eq!(pool_after.liquidity, liquidity)
    }

    let limit_without_cross_tick_amount = TokenAmount::new(U256::from(10_068));
    let not_cross_amount = TokenAmount::new(U256::from(1));
    let min_amount_to_cross_from_tick_price = TokenAmount::new(U256::from(3));
    let crossing_amount_by_amount_out = TokenAmount::new(U256::from(20136101434u128));

    let mint_amount = limit_without_cross_tick_amount.get()
        + not_cross_amount.get()
        + min_amount_to_cross_from_tick_price.get()
        + crossing_amount_by_amount_out.get();

    token_x.mint(&deployer, &mint_amount);
    token_y.mint(&deployer, &mint_amount);

    token_x.approve(invariant.address(), &mint_amount);
    token_y.approve(invariant.address(), &mint_amount);

    {
        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let sqrt_price_limit = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        invariant
            .swap(
                pool_key,
                true,
                limit_without_cross_tick_amount,
                true,
                sqrt_price_limit,
            )
            .unwrap();

        let pool = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let expected_tick = -10;
        let expected_sqrt_price = calculate_sqrt_price(expected_tick).unwrap();

        assert_eq!(pool.current_tick_index, expected_tick);
        assert_eq!(pool.sqrt_price, expected_sqrt_price);
        assert_eq!(pool.liquidity, pool_before.liquidity);
    }

    let sqrt_price_limit = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
    let result = invariant.swap(pool_key, true, not_cross_amount, true, sqrt_price_limit);
    assert_eq!(result, Err(InvariantError::NoGainSwap));
}
