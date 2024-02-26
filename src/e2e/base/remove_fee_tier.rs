use crate::contracts::errors::InvariantError;
use crate::math::percentage::Percentage;
use crate::FeeTier;
use crate::InvariantDeployer;
use decimal::*;
use odra::test_env;
use odra::types::U128;

#[test]

fn test_remove_fee_tier() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(U128::from(0));
    let fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();

    invariant
        .add_fee_tier(fee_tier.fee.v, fee_tier.tick_spacing)
        .unwrap();
    let exist = invariant.fee_tier_exist(fee_tier);
    assert!(exist);

    invariant.remove_fee_tier(fee_tier).unwrap();
    let exist = invariant.fee_tier_exist(fee_tier);
    assert!(!exist);
}

#[test]
fn test_remove_not_existing_fee_tier() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(U128::from(0));
    let fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();

    let result = invariant.remove_fee_tier(fee_tier);
    assert_eq!(result, Err(InvariantError::FeeTierNotFound));
}

#[test]
fn test_remove_fee_tier_not_admin() {
    let deployer = test_env::get_account(0);
    let not_admin = test_env::get_account(1);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(U128::from(0));
    let fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();

    invariant
        .add_fee_tier(fee_tier.fee.v, fee_tier.tick_spacing)
        .unwrap();
    let exist = invariant.fee_tier_exist(fee_tier);
    assert!(exist);

    test_env::set_caller(not_admin);
    let result = invariant.remove_fee_tier(fee_tier);
    assert_eq!(result, Err(InvariantError::NotAdmin));
}
