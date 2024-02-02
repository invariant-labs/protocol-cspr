use crate::InvariantDeployer;
use odra::test_env;
use odra::types::U128;

#[test]

fn init_invariant() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    let _invariant = InvariantDeployer::init(U128::from(10));
}
