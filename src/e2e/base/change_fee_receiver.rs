use crate::contracts::errors::InvariantError;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::InvariantDeployer;
use crate::{FeeTier, PoolKey};
use decimal::*;
use odra::test_env;
use odra::types::casper_types::ContractPackageHash;
use odra::types::Address;
use odra::types::U128;

#[test]
fn test_change_fee_reciever() {
    let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
    let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(U128::from(0));

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
    invariant
        .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
        .unwrap();

    let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
    assert!(exist);

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    invariant
        .create_pool(
            token_0,
            token_1,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            init_sqrt_price.get(),
            init_tick,
        )
        .unwrap();

    let new_receiver = test_env::get_account(1);
    let _pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();

    invariant
        .change_fee_receiver(
            token_0,
            token_1,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            new_receiver,
        )
        .unwrap();

    let pool = invariant
        .get_pool(token_0, token_1, fee_tier.fee.get(), fee_tier.tick_spacing)
        .unwrap();
    assert_eq!(pool.fee_receiver, new_receiver);
}

#[test]
#[should_panic]
fn test_not_admin_change_fee_reciever() {
    let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
    let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(U128::from(0));

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
    invariant
        .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
        .unwrap();

    let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
    assert!(exist);

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    invariant
        .create_pool(
            token_0,
            token_1,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            init_sqrt_price.get(),
            init_tick,
        )
        .unwrap();

    let new_receiver = test_env::get_account(1);
    let _pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();
    test_env::set_caller(new_receiver);
    let result = invariant.change_fee_receiver(
        token_0,
        token_1,
        fee_tier.fee.get(),
        fee_tier.tick_spacing,
        new_receiver,
    );
    assert_eq!(result, Err(InvariantError::NotAdmin));
}
