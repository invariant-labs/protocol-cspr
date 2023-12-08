use crate::math::percentage::Percentage;
use crate::FeeTier;
use crate::InvariantDeployer;
use crate::InvariantError;
use decimal::*;
use odra::test_env;
use odra::types::U128;

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
