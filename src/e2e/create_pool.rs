use crate::math::percentage::Percentage;
use crate::FeeTier;
use crate::InvariantDeployer;
use crate::InvariantError;
use decimal::*;
use odra::test_env;
use odra::types::casper_types::ContractPackageHash;
use odra::types::Address;
use odra::types::U128;

#[test]
fn test_create_pool() {
    // TODO - change to real token addresses
    let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
    let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    let fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();
    invariant.add_fee_tier(fee_tier).unwrap();

    let exist = invariant.fee_tier_exist(fee_tier);
    assert!(exist);

    let init_tick = 0;
    invariant
        .create_pool(token_0, token_1, fee_tier, init_tick)
        .unwrap();

    invariant.get_pool(token_0, token_1, fee_tier).unwrap();
}

#[test]
fn test_create_pool_with_same_tokens() {
    // TODO - change to real token addresses
    let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));

    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    let fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();
    invariant.add_fee_tier(fee_tier).unwrap();

    let exist = invariant.fee_tier_exist(fee_tier);
    assert!(exist);

    let init_tick = 0;
    let result = invariant.create_pool(token_0, token_0, fee_tier, init_tick);

    assert_eq!(result, Err(InvariantError::TokensAreSame));
}

#[test]
fn test_create_pool_x_to_y_and_y_to_x() {
    // TODO - change to real token addresses
    let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
    let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    let fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();
    invariant.add_fee_tier(fee_tier).unwrap();

    let exist = invariant.fee_tier_exist(fee_tier);
    assert!(exist);

    let init_tick = 0;
    invariant
        .create_pool(token_0, token_1, fee_tier, init_tick)
        .unwrap();
    let result = invariant.create_pool(token_1, token_0, fee_tier, init_tick);
    assert_eq!(result, Err(InvariantError::PoolAlreadyExist));
}

#[test]
fn test_create_pool_fee_tier_not_added() {
    // TODO - change to real token addresses
    let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
    let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    let fee_tier = FeeTier::new(Percentage::new(U128::from(10)), 1).unwrap();

    let init_tick = 0;
    let result = invariant.create_pool(token_0, token_1, fee_tier, init_tick);
    assert_eq!(result, Err(InvariantError::FeeTierNotFound));
}
