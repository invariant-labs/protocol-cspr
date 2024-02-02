use crate::contracts::errors::InvariantError;
use crate::math::percentage::Percentage;
use crate::InvariantDeployer;
use decimal::*;
use odra::test_env;
use odra::types::U128;

#[test]
fn test_change_protocol_fee() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(U128::from(0));

    let protocol_fee = invariant.get_protocol_fee();
    assert_eq!(protocol_fee, Percentage::new(U128::from(0)));

    let new_fee = Percentage::new(U128::from(1));
    invariant.change_protocol_fee(new_fee).unwrap();

    let protocol_fee = invariant.get_protocol_fee();
    assert_eq!(protocol_fee, new_fee);
}

#[test]
fn test_change_protocol_fee_not_admin() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let mut invariant = InvariantDeployer::init(U128::from(0));

    let protocol_fee = invariant.get_protocol_fee();
    assert_eq!(protocol_fee, Percentage::new(U128::from(0)));

    let new_fee = Percentage::new(U128::from(1));
    test_env::set_caller(test_env::get_account(1));
    let result = invariant.change_protocol_fee(new_fee);

    assert_eq!(result, Err(InvariantError::NotAdmin));
}
