use decimal::Decimal;
use odra::{types::U256, Variable};
use types::liquidity::Liquidity;
use uints::U256T;

extern crate alloc;

pub mod consts;
pub mod math;
pub mod types;
pub mod uints;

#[odra::module]
pub struct Invariant {
    liquidity: Variable<U256>,
}

#[odra::module]
impl Invariant {
    #[odra(init)]
    pub fn init(&mut self) {
        let liquidity = Liquidity::new(U256T::from(100_000_000u128));
        let liquidity_u256 = U256::from_dec_str(liquidity.get().to_string().as_str()).unwrap();
        self.liquidity.set(liquidity_u256);
    }
}

#[cfg(test)]
mod tests {
    use odra::test_env;

    use super::*;
    #[test]

    fn init_invariant() {
        let deployer = test_env::get_account(0);
        test_env::set_caller(deployer);
        let _invariant = InvariantDeployer::init();
    }
}
