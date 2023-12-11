#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;

#[cfg(test)]
pub mod e2e;

use crate::math::{check_tick, percentage::Percentage, sqrt_price::SqrtPrice};
use contracts::{
    FeeTier, FeeTiers, Pool, PoolKey, PoolKeys, Pools, Position, Positions, State, Tick, Tickmap,
    Ticks,
};
use decimal::Decimal;
use math::liquidity::Liquidity;
use odra::contract_env;
use odra::prelude::vec::Vec;
use odra::types::{Address, U256};
use odra::{OdraType, UnwrapOrRevert, Variable};
use odra_modules::erc20::Erc20Ref;

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

impl Invariant {
    fn create_tick(&mut self, pool_key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        let current_timestamp = contract_env::get_block_time();

        check_tick(index, pool_key.fee_tier.tick_spacing)
            .map_err(|_| InvariantError::InvalidTickIndexOrTickSpacing)?;

        let pool = self.pools.get(pool_key)?;

        let tick = Tick::create(index, &pool, current_timestamp);
        self._ticks.add(pool_key, index, &tick)?;

        self._tickmap
            .flip(true, index, pool_key.fee_tier.tick_spacing, pool_key);

        Ok(tick)
    }
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

    pub fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool {
        self._tickmap.get(index, key.fee_tier.tick_spacing, key)
    }

    pub fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        self._ticks.get(key, index)
    }

    pub fn create_position(
        &mut self,
        pool_key: PoolKey,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: Liquidity,
        slippage_limit_lower: SqrtPrice,
        slippage_limit_upper: SqrtPrice,
    ) -> Result<Position, InvariantError> {
        let caller = contract_env::caller();
        let contract = contract_env::self_address();
        let current_timestamp = contract_env::get_block_time();
        let current_block_number = contract_env::get_block_time();

        // liquidity delta = 0 => return
        if liquidity_delta == Liquidity::new(U256::from(0)) {
            return Err(InvariantError::ZeroLiquidity);
        }

        let mut pool = self.pools.get(pool_key)?;

        let mut lower_tick = self
            ._ticks
            .get(pool_key, lower_tick)
            .unwrap_or_else(|_| Self::create_tick(self, pool_key, lower_tick).unwrap());

        let mut upper_tick = self
            ._ticks
            .get(pool_key, upper_tick)
            .unwrap_or_else(|_| Self::create_tick(self, pool_key, upper_tick).unwrap());

        let (position, x, y) = Position::create(
            &mut pool,
            pool_key,
            &mut lower_tick,
            &mut upper_tick,
            current_timestamp,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            current_block_number,
            pool_key.fee_tier.tick_spacing,
        )?;

        self.pools.update(pool_key, &pool)?;

        self._positions.add(caller, &position);

        self._ticks
            .update(pool_key, lower_tick.index, &lower_tick)?;
        self._ticks
            .update(pool_key, upper_tick.index, &upper_tick)?;

        Erc20Ref::at(&pool_key.token_x).transfer_from(&caller, &contract, &x.get());
        Erc20Ref::at(&pool_key.token_y).transfer_from(&caller, &contract, &y.get());

        Ok(position)
    }
}
