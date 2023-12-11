#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;

#[cfg(test)]
pub mod e2e;

use crate::math::{check_tick, percentage::Percentage, sqrt_price::SqrtPrice};
use contracts::{
    pool_key, FeeTier, FeeTiers, Pool, PoolKey, PoolKeys, Pools, Positions, State, Tickmap, Ticks,
};
use odra::contract_env;
use odra::prelude::vec::Vec;
use odra::types::Address;
use odra::{OdraType, UnwrapOrRevert, Variable};

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
    TokensAreSame,
    AmountUnderMinimumAmountOut,
    InvalidFee,
    NotEmptyTickDeinitialization,
    InvalidInitTick,
}

pub struct SwapResult {
    next_sqrt_price: SqrtPrice,
}

#[odra::module]
pub struct Invariant {
    _positions: Positions,
    pools: Pools,
    _tickmap: Tickmap,
    _ticks: Ticks,
    fee_tiers: Variable<FeeTiers>,
    pool_keys: Variable<PoolKeys>,
    state: Variable<State>,
}

#[odra::module]
impl Entrypoints for Invariant {
    #[odra(init)]
    pub fn init(&mut self, protocol_fee: Percentage) {
        let caller = contract_env::caller();

        self.pool_keys.set(PoolKeys::default());
        self.fee_tiers.set(FeeTiers::default());
        self.state.set(State {
            admin: caller,
            protocol_fee,
        })
    }
    pub fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        let caller = contract_env::caller();
        let state = self.state.get().unwrap_or_revert();
        let mut fee_tiers = self.fee_tiers.get().unwrap_or_revert();

        if caller != state.admin {
            return Err(InvariantError::NotAdmin);
        }

        fee_tiers.add(fee_tier)?;

        self.fee_tiers.set(fee_tiers);
        Ok(())
    }

    pub fn fee_tier_exist(&self, fee_tier: FeeTier) -> bool {
        let fee_tiers = self.fee_tiers.get().unwrap_or_revert();
        fee_tiers.contains(fee_tier)
    }

    pub fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        let caller = contract_env::caller();
        let state = self.state.get().unwrap_or_revert();
        let mut fee_tiers = self.fee_tiers.get().unwrap_or_revert();

        if caller != state.admin {
            return Err(InvariantError::NotAdmin);
        }

        fee_tiers.remove(fee_tier)?;

        self.fee_tiers.set(fee_tiers);

        Ok(())
    }

    pub fn get_fee_tiers(&self) -> Vec<FeeTier> {
        let fee_tiers = self.fee_tiers.get().unwrap_or_revert();
        fee_tiers.get_all()
    }

    pub fn create_pool(
        &mut self,
        token_0: Address,
        token_1: Address,
        fee_tier: FeeTier,
        init_tick: i32,
    ) -> Result<(), InvariantError> {
        let current_timestamp = odra::contract_env::get_block_time();
        let mut pool_keys = self.pool_keys.get().unwrap_or_revert();
        let fee_tiers = self.fee_tiers.get().unwrap_or_revert();
        let state = self.state.get().unwrap_or_revert();

        if !fee_tiers.contains(fee_tier) {
            return Err(InvariantError::FeeTierNotFound);
        };

        check_tick(init_tick, fee_tier.tick_spacing)
            .map_err(|_| InvariantError::InvalidInitTick)?;

        let pool_key = PoolKey::new(token_0, token_1, fee_tier)?;

        if self.pools.get(pool_key).is_ok() {
            return Err(InvariantError::PoolAlreadyExist);
        };

        let pool = Pool::create(init_tick, current_timestamp, state.admin);

        self.pools.add(pool_key, &pool)?;
        pool_keys.add(pool_key)?;

        self.pool_keys.set(pool_keys);
        Ok(())
    }

    pub fn get_pool(
        &self,
        token_0: Address,
        token_1: Address,
        fee_tier: FeeTier,
    ) -> Result<Pool, InvariantError> {
        let key: PoolKey = PoolKey::new(token_0, token_1, fee_tier)?;
        let pool = self.pools.get(key)?;

        Ok(pool)
    }

    pub fn get_pools(&self) -> Vec<PoolKey> {
        self.pool_keys.get().unwrap_or_revert().get_all()
    }

    pub fn get_protocol_fee(&self) -> Percentage {
        let state = self.state.get().unwrap_or_revert();
        state.protocol_fee
    }

    pub fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
        let caller = contract_env::caller();
        let mut pool = self.pools.get(pool_key)?;

        if caller != pool.fee_receiver {
            return Err(InvariantError::NotFeeReceiver);
        }

        let (_fee_protocol_token_x, _fee_protocol_token_y) = pool.withdraw_protocol_fee(pool_key);

        self.pools.update(pool_key, &pool)?;

        // TODO
        // Transfer tokens

        Ok(())
    }

    pub fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Result<(), InvariantError> {
        let caller = contract_env::caller();
        let mut state = self.state.get().unwrap_or_revert();

        if caller != state.admin {
            return Err(InvariantError::NotAdmin);
        }

        state.protocol_fee = protocol_fee;

        self.state.set(state);

        Ok(())
    }

    pub fn change_fee_receiver(
        &mut self,
        pool_key: PoolKey,
        fee_receiver: Address,
    ) -> Result<(), InvariantError> {
        let caller = contract_env::caller();
        let state = self.state.get().unwrap_or_revert();
        let mut pool = self.pools.get(pool_key)?;

        if caller != state.admin {
            return Err(InvariantError::NotAdmin);
        }

        pool.fee_receiver = fee_receiver;
        self.pools.update(pool_key, &pool)?;

        Ok(())
    }
}
