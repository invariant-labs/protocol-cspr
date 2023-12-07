use crate::math::percentage::Percentage;
use crate::InvariantDeployer;
use decimal::*;
use odra::test_env;
use odra::types::U128;

#[test]

fn add_multiple_fee_tiers() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    let first_fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
    let second_fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 2).unwrap();
    let third_fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 3).unwrap();

    invariant.add_fee_tier(first_fee_tier);
    invariant.add_fee_tier(second_fee_tier);
    invariant.add_fee_tier(third_fee_tier);
}
