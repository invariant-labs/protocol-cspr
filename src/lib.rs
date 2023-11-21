#![no_std]

extern crate alloc;

pub mod contracts;

use crate::contracts::State;
use alloc::string::ToString;
use decimal::Decimal;
use invariant_math::liquidity::Liquidity;
use invariant_math::uints::{U128T,U256T};
use odra::{types::{U256, U128}, Variable, contract_env};

#[odra::module]
pub struct Invariant {
    state: Variable<State>,
    liquidity: Variable<U256>,
}

#[odra::module]
impl Invariant {
    #[odra(init)]
    pub fn init(&mut self) {
        let caller = contract_env::caller();
        let liquidity = Liquidity::new(U256T::from(100_000_000u128));
        let liquidity_u256 = U256::from_dec_str(liquidity.get().to_string().as_str()).unwrap();
        self.liquidity.set(liquidity_u256);
        self.state.set(State {
            admin: caller,
            protocol_fee: U128::from(10000000000u128),
        })
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
