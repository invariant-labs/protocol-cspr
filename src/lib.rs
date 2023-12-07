#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;

// #[cfg(test)]
// pub mod e2e;

use crate::contracts::InvariantTrait;
use crate::math::{percentage::Percentage, sqrt_price::SqrtPrice};
use contracts::{FeeTier, FeeTiers, PoolKeys, Pools, Positions, State, Tickmap, Ticks};
use odra::{contract_env, Variable};

#[derive(OdraType, Debug, PartialEq)]
pub enum InvariantError {
    NotAdmin,
    NotFeeReceiver,
    PoolAlreadyExist,
    PoolNotFound,
    TickAlreadyExist,
    InvalidTickIndexOrTickSpacing,
    PositionNotFound,
    TickNotFound,
    FeeTierNotFound,
    PoolKeyNotFound,
    AmountIsZero,
    WrongLimit,
    PriceLimitReached,
    NoGainSwap,
    InvalidTickSpacing,
    FeeTierAlreadyExist,
    PoolKeyAlreadyExist,
    UnauthorizedFeeReceiver,
    ZeroLiquidity,
    TransferError,
    TokensAreTheSame,
    AmountUnderMinimumAmountOut,
    InvalidFee,
    NotEmptyTickDeinitialization,
}

pub struct SwapResult {
    next_sqrt_price: SqrtPrice,
}

#[odra::module]
pub struct Invariant {
    positions: Positions,
    pools: Pools,
    tickmap: Tickmap,
    ticks: Ticks,
    fee_tiers: Variable<FeeTiers>,
    pool_keys: Variable<PoolKeys>,
    state: Variable<State>,
}

#[odra::module]
impl Invariant {
    #[odra(init)]
    pub fn init(&mut self, protocol_fee: Percentage) {
        let caller = contract_env::caller();

        self.state.set(State {
            admin: caller,
            protocol_fee,
        })
    }

    // impl InvariantTrait for Invariant {

    pub fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        let caller = contract_env::caller();
        let state = self.state.get();

        if caller != state.get().admin {
            // contract_env::revert(InvariantError::NotAdmin);
            return Err(InvariantError::NotAdmin);
        }

        self.fee_tiers.add(fee_tier)?;

        Ok(())
    }
    // }
}
