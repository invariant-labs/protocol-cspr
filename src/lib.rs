#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;

#[cfg(test)]
pub mod e2e;

// use crate::contracts::Entrypoints;
use crate::math::{percentage::Percentage, sqrt_price::SqrtPrice};
use contracts::{FeeTier, FeeTiers, PoolKeys, Pools, Positions, State, Tickmap, Ticks};
use odra::prelude::vec::Vec;
use odra::{contract_env, OdraType, Variable};

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
    pub fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        let caller = contract_env::caller();
        let state = self.state.get().unwrap();
        let mut fee_tiers = self.fee_tiers.get_or_default();

        if caller != state.admin {
            return Err(InvariantError::NotAdmin);
        }

        fee_tiers.add(fee_tier)?;

        self.fee_tiers.set(fee_tiers);
        Ok(())
    }

    pub fn fee_tier_exist(&self, fee_tier: FeeTier) -> bool {
        let fee_tiers = self.fee_tiers.get_or_default();
        fee_tiers.contains(fee_tier)
    }

    pub fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        let caller = contract_env::caller();
        let state = self.state.get().unwrap();
        let mut fee_tiers = self.fee_tiers.get_or_default();

        if caller != state.admin {
            return Err(InvariantError::NotAdmin);
        }

        fee_tiers.remove(fee_tier)?;

        self.fee_tiers.set(fee_tiers);

        Ok(())
    }

    pub fn get_fee_tiers(&self) -> Vec<FeeTier> {
        let fee_tiers = self.fee_tiers.get_or_default();
        fee_tiers.get_all()
    }
}

// impl Entrypoints for Invariant {}
