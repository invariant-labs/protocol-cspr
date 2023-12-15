use crate::contracts::errors::InvariantError;
use crate::contracts::PoolKey;
use crate::e2e::snippets::init;
use crate::math::liquidity::Liquidity;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::FeeTier;
use crate::InvariantDeployer;
use decimal::*;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
fn test_add_multiple_fee_tiers() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    let first_fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();
    let second_fee_tier = FeeTier::new(Percentage::new(U128::from(20)), 2).unwrap();
    let third_fee_tier = FeeTier::new(Percentage::new(U128::from(30)), 3).unwrap();

    invariant.add_fee_tier(first_fee_tier).unwrap();
    invariant.add_fee_tier(second_fee_tier).unwrap();
    invariant.add_fee_tier(third_fee_tier).unwrap();

    let exist = invariant.fee_tier_exist(first_fee_tier);
    assert!(exist);
    let exist = invariant.fee_tier_exist(second_fee_tier);
    assert!(exist);
    let exist = invariant.fee_tier_exist(third_fee_tier);
    assert!(exist);

    let fee_tiers = invariant.get_fee_tiers();
    assert_eq!(fee_tiers.len(), 3);
    assert_eq!(fee_tiers[0], first_fee_tier);
    assert_eq!(fee_tiers[1], second_fee_tier);
    assert_eq!(fee_tiers[2], third_fee_tier);
}

#[test]
fn test_add_existing_fee_tier() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    let first_fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();

    invariant.add_fee_tier(first_fee_tier).unwrap();

    let result = invariant.add_fee_tier(first_fee_tier);
    assert_eq!(result, Err(InvariantError::FeeTierAlreadyExist));
}

#[test]
fn test_add_fee_tier_not_admin() {
    let deployer = test_env::get_account(0);
    let not_admin = test_env::get_account(1);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));
    let fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();

    test_env::set_caller(not_admin);
    let result = invariant.add_fee_tier(fee_tier);

    assert_eq!(result, Err(InvariantError::NotAdmin));
}

#[test]
fn test_add_fee_tier_zero_fee() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));
    let first_fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();

    invariant.add_fee_tier(first_fee_tier).unwrap();
}

#[test]
fn test_interaction_with_pool_on_removed_fee_tier() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);

    let mint_amount = U256::from(500);
    let fee = Percentage::new(U128::from(0));
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
        let mint_amount = U256::from(10u128.pow(10));
        token_x.mint(&deployer, &mint_amount);
        token_y.mint(&deployer, &mint_amount);
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
}
