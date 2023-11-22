#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;

use crate::contracts::State;
use contracts::Tickmap;
use decimal::Decimal;
use decimal::Decimal;
use invariant_math::liquidity::Liquidity;
use invariant_math::uints::U256T;
use math::{liquidity::Liquidity, percentage::Percentage};
use odra::{
    contract_env,
    types::{U128, U256},
    Variable,
};
use odra::{types::U256, Variable};

#[odra::module]
pub struct Invariant {
    tickmap: Tickmap,
    state: Variable<State>,
    liquidity: Variable<Liquidity>,
}

#[odra::module]
impl Invariant {
    #[odra(init)]
    pub fn init(&mut self) {
        let caller = contract_env::caller();
        let liquidity = Liquidity::new(U256::from(100_000_000u128));
        self.liquidity.set(liquidity);
        self.state.set(State {
            admin: caller,
            protocol_fee: Percentage::new(U128::from(10000000000u128)),
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
