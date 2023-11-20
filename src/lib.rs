#![no_std]
mod contracts;
use decimal::Decimal;
use invariant_math::liquidity::Liquidity;
use invariant_math::uints::U256T;
use odra::{types::U256, Variable};
extern crate alloc;
use alloc::string::ToString;
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
