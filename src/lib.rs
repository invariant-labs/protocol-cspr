#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;

use crate::math::{liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice};
use contracts::{FeeTier, Pool, PoolKey, Position, State, Tick, Tickmap};
use decimal::Decimal;
use odra::{
    contract_env,
    types::{casper_types::ContractPackageHash, Address, U128, U256},
    Variable,
};

#[derive(Debug, PartialEq)]
pub enum ContractErrors {
    InsufficientSenderBalance,
    InsufficientLPLocked,
    PairNotFound,
    MintFailed,
    BurnFailed,
    SwapFailed,
    NotAnAdmin,
    PoolAlreadyExist,
    PoolNotFound,
    TickAlreadyExist,
    InvalidTickIndexOrTickSpacing,
    FeeTierAlreadyExist,
    InvalidFee,
    PoolKeyNotFound,
    PoolKeyAlreadyExist,
    PositionNotFound,
    TickNotFound,
    FeeTierNotFound,
    AmountIsZero,
    WrongLimit,
    PriceLimitReached,
    NoGainSwap,
    InvalidTickSpacing,
    FeeTierAlreadyAdded,
    NotAFeeReceiver,
    ZeroLiquidity,
    TransferError,
    TokensAreTheSame,
    AmountUnderMinimumAmountOut,
}

pub struct SwapResult {
    next_sqrt_price: SqrtPrice,
}

#[odra::module]
pub struct Invariant {
    tickmap: Tickmap,
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

        let token_0: Address = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_1: Address = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(U128::from(1)),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();
        self.tickmap.flip(true, 0, 1, pool_key);

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
