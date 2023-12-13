use crate::contracts::InvariantError;
use crate::contracts::PoolKey;
use crate::math::fee_growth::FeeGrowth;
use crate::math::liquidity::Liquidity;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::math::sqrt_price::SqrtPrice;
use crate::math::token_amount::TokenAmount;
use crate::math::MIN_SQRT_PRICE;
use crate::token::TokenDeployer;
use crate::FeeTier;
use crate::InvariantDeployer;
use alloc::string::String;
use decimal::*;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
fn liquidity_gap() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let mut token_x = TokenDeployer::init(String::from(""), String::from(""), 0, &mint_amount);
    let mut token_y = TokenDeployer::init(String::from(""), String::from(""), 0, &mint_amount);
    let mut invariant = InvariantDeployer::init(Percentage::from_scale(1, 2));
    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant.add_fee_tier(fee_tier).unwrap();
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
        let mint_amount = U256::from(10u128.pow(10));
        token_x.mint(&deployer, &mint_amount);
        token_y.mint(&deployer, &mint_amount);
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -10;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(20_006_000);

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
    // Init basic swap
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(10067);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let dex_x_before = token_x.balance_of(invariant.address());
        let dex_y_before = token_y.balance_of(invariant.address());

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        let target_sqrt_price = invariant
            .quote(pool_key, true, swap_amount, true, slippage)
            .unwrap()
            .target_sqrt_price;
        invariant
            .swap(pool_key, true, swap_amount, true, target_sqrt_price)
            .unwrap();

        let pool = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let expected_price = calculate_sqrt_price(-10).unwrap();
        let expected_y_amount_out = U256::from(9999);
        let liquidity = Liquidity::from_integer(20_006_000);
        let lower_tick_index = -10;

        assert_eq!(pool.liquidity, liquidity);
        assert_eq!(pool.current_tick_index, lower_tick_index);
        assert_eq!(pool.sqrt_price, expected_price);

        let user_x = token_x.balance_of(&caller);
        let user_y = token_y.balance_of(&caller);

        let dex_x_after = token_x.balance_of(invariant.address());
        let dex_y_after = token_y.balance_of(invariant.address());

        let delta_dex_x = dex_x_after - dex_x_before;
        let delta_dex_y = dex_y_before - dex_y_after;

        assert_eq!(user_x, U256::from(0));
        assert_eq!(user_y, expected_y_amount_out);
        assert_eq!(delta_dex_x, swap_amount.get());
        assert_eq!(delta_dex_y, expected_y_amount_out);
        assert_eq!(
            pool.fee_growth_global_x,
            FeeGrowth::new(U128::from(29991002699190242927121u128))
        );
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(U128::from(0)));
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(1)));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(0)));
    }
    // No gain swap
    {
        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(U256::from(1));
        let result = invariant.swap(pool_key, true, swap_amount, true, slippage);
        assert_eq!(result, Err(InvariantError::NoGainSwap));
    }
    // Should skip gap and then swap
    {
        test_env::set_caller(deployer);
        let lower_tick_after_swap = -90;
        let upper_tick_after_swap = -50;
        let liquidity_delta = Liquidity::from_integer(20008000);

        token_x.mint(&deployer, &liquidity_delta.get());
        token_y.mint(&deployer, &liquidity_delta.get());

        token_x.approve(invariant.address(), &liquidity_delta.get());
        token_y.approve(invariant.address(), &liquidity_delta.get());

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key,
                lower_tick_after_swap,
                upper_tick_after_swap,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        let caller = test_env::get_account(1);
        let amount = U256::from(10067);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        let target_sqrt_price = invariant
            .quote(pool_key, true, swap_amount, true, slippage)
            .unwrap()
            .target_sqrt_price;
        invariant
            .swap(pool_key, true, swap_amount, true, target_sqrt_price)
            .unwrap();

        invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();
    }
}
