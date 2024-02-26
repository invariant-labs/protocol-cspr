use crate::contracts::InvariantError;
use crate::contracts::PoolKey;
use crate::e2e::snippets::init;
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
fn test_basic_slippage() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init dex and tokens
    let mint_amount = U256::from(10u128.pow(23));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init pool
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
    // Init position
    {
        let amount = U256::from(10u128.pow(10));
        token_x.approve(invariant.address(), &amount);
        token_y.approve(invariant.address(), &amount);

        let lower_tick = -1000;
        let upper_tick = 1000;
        let liquidity = Liquidity::from_integer(10_000_000_000u128);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key,
                lower_tick,
                upper_tick,
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
    // Init swap
    {
        let amount = U256::from(10u128.pow(8));
        token_x.approve(invariant.address(), &amount);

        let target_sqrt_price = SqrtPrice::new(U128::from(1009940000000000000000001u128));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(pool_key, false, swap_amount, true, target_sqrt_price)
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let expected_sqrt_price = SqrtPrice::new(U128::from(1009940000000000000000000u128));

        assert_eq!(pool_after.sqrt_price, expected_sqrt_price);
    }
}

#[test]
fn test_swap_close_to_limit() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init dex and tokens
    let mint_amount = U256::from(10u128.pow(23));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);
    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init pool
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
    // Init position
    {
        let amount = U256::from(10u128.pow(10));
        token_x.approve(invariant.address(), &amount);
        token_y.approve(invariant.address(), &amount);

        let lower_tick = -1000;
        let upper_tick = 1000;
        let liquidity = Liquidity::from_integer(10_000_000_000u128);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key,
                lower_tick,
                upper_tick,
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
    // Init swap
    {
        let amount = U256::from(10u128.pow(8));
        token_x.approve(invariant.address(), &amount);

        let swap_amount = TokenAmount::new(amount);
        let target_sqrt_price = SqrtPrice::new(U128::from(MAX_SQRT_PRICE));
        let quoted_target_sqrt_price = invariant
            .quote(pool_key, false, swap_amount, true, target_sqrt_price)
            .unwrap()
            .target_sqrt_price;

        let target_sqrt_price = quoted_target_sqrt_price - SqrtPrice::new(U128::from(1));
        let result = invariant.swap(pool_key, false, swap_amount, true, target_sqrt_price);

        assert_eq!(result, Err(InvariantError::PriceLimitReached));
    }
}

#[test]
fn test_swap_exact_limit() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init dex and tokens
    let mint_amount = U256::from(10u128.pow(23));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);
    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init pool
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
    // Init position
    {
        let amount = U256::from(10u128.pow(10));
        token_x.approve(invariant.address(), &amount);
        token_y.approve(invariant.address(), &amount);

        let lower_tick = -1000;
        let upper_tick = 1000;
        let liquidity = Liquidity::from_integer(10_000_000_000u128);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key,
                lower_tick,
                upper_tick,
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
    // Init swap
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(1000);
        token_x.mint(&caller, &amount);

        let amount_x = token_x.balance_of(&caller);
        assert_eq!(amount_x, amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let swap_amount = TokenAmount::new(amount);

        let sqrt_price_limit = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));

        let quoted_target_sqrt_price = invariant
            .quote(pool_key, true, swap_amount, true, sqrt_price_limit)
            .unwrap()
            .target_sqrt_price;

        invariant
            .swap(pool_key, true, swap_amount, true, quoted_target_sqrt_price)
            .unwrap();
    }
}
