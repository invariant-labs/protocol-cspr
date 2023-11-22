#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;

use crate::contracts::State;
use crate::math::{liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice};
use contracts::{Pool, Position, Tick};
use decimal::Decimal;
use odra::{
    contract_env,
    types::{U128, U256},
    Variable,
};

pub struct SwapResult {
    next_sqrt_price: SqrtPrice,
}

#[odra::module]
pub struct Invariant {
    position: Variable<Position>,
    pool: Variable<Pool>,
    tick: Variable<Tick>,
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
        self.position.set(Position::default());
        self.tick.set(Tick::default());
        self.pool.set(Pool::default());
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
