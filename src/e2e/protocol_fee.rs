use crate::contracts::InvariantError;
use crate::contracts::PoolKey;
use crate::e2e::snippets::init;
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
fn test_protocol_fee() {
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
        // let mint_amount = U256::from(10u128.pow(10));
        // token_x.mint(&deployer, &mint_amount);
        // token_y.mint(&deployer, &mint_amount);
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

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
        let amount = U256::from(1000);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(500));
        assert_eq!(amount_y, U256::from(1000));

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(pool_key, true, swap_amount, true, slippage)
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
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
    // Withdraw fee
    test_env::set_caller(test_env::get_account(0));
    invariant.withdraw_protocol_fee(pool_key).unwrap();
    let amount_x = token_x.balance_of(&deployer);
    let amount_y = token_y.balance_of(&deployer);

    assert_eq!(amount_x, U256::from(9999999501u128));
    assert_eq!(amount_y, U256::from(9999999000u128));

    let amount_x = token_x.balance_of(invariant.address());
    let amount_y = token_y.balance_of(invariant.address());
    assert_eq!(amount_x, U256::from(1499));
    assert_eq!(amount_y, U256::from(7));

    let pool_after_withdraw = invariant
        .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
        .unwrap();

    assert_eq!(
        pool_after_withdraw.fee_protocol_token_x,
        TokenAmount::new(U256::from(0))
    );
    assert_eq!(
        pool_after_withdraw.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}

#[test]
fn test_protocol_fee_not_admin() {
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
        // let mint_amount = U256::from(10u128.pow(10));
        // token_x.mint(&deployer, &mint_amount);
        // token_y.mint(&deployer, &mint_amount);
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

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
        let amount = U256::from(1000);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(500));
        assert_eq!(amount_y, U256::from(1000));

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(pool_key, true, swap_amount, true, slippage)
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
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
    // Withdraw fee not admin
    let result = invariant.withdraw_protocol_fee(pool_key);
    assert_eq!(result, Err(InvariantError::NotFeeReceiver));
}
