#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;

use math::sqrt_price::{get_max_tick, get_min_tick};
pub use odra_modules::erc20::{Erc20, Erc20Deployer, Erc20Ref};

#[cfg(test)]
pub mod e2e;

use crate::contracts::errors::InvariantError;
use crate::math::{check_tick, percentage::Percentage, sqrt_price::SqrtPrice};
use contracts::{events::*, unwrap_invariant_result, InvariantConfig, InvariantErrorReturn};
use contracts::{
    FeeTier, FeeTiers, Pool, PoolKey, PoolKeys, Pools, Position, Positions, Tick, Tickmap, Ticks,
    UpdatePoolTick,
};
use decimal::*;
use math::clamm::{calculate_min_amount_out, compute_swap_step, SwapResult};
use math::liquidity::Liquidity;
use math::token_amount::TokenAmount;
use math::{MAX_SQRT_PRICE, MIN_SQRT_PRICE};
use odra::contract_env;
use odra::prelude::vec;
use odra::prelude::vec::Vec;
use odra::types::event::OdraEvent;
use odra::types::{Address, U128, U256};
use odra::{OdraType, UnwrapOrRevert, Variable};
use traceable_result::*;
#[derive(OdraType, Debug, PartialEq)]
pub struct QuoteResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub target_sqrt_price: SqrtPrice,
    pub ticks: Vec<Tick>,
}
#[derive(OdraType, Debug, PartialEq)]
pub struct CalculateSwapResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub start_sqrt_price: SqrtPrice,
    pub target_sqrt_price: SqrtPrice,
    pub fee: TokenAmount,
    pub pool: Pool,
    pub ticks: Vec<Tick>,
}
#[derive(OdraType, Debug)]
pub struct SwapHop {
    pub token_x: Address,
    pub token_y: Address,
    pub fee: U128,
    pub tick_spacing: u32,
    pub x_to_y: bool,
}

#[odra::module]
pub struct Invariant {
    positions: Positions,
    pools: Pools,
    tickmap: Tickmap,
    ticks: Ticks,
    fee_tiers: Variable<FeeTiers>,
    pool_keys: Variable<PoolKeys>,
    config: Variable<InvariantConfig>,
}

impl Invariant {
    fn create_tick(&mut self, pool_key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        let current_timestamp = contract_env::get_block_time();

        unwrap_invariant_result(
            check_tick(index, pool_key.fee_tier.tick_spacing)
                .map_err(|_| InvariantError::InvalidTickIndexOrTickSpacing),
        );

        let pool = unwrap_invariant_result(self.pools.get(pool_key));

        let tick = Tick::create(index, &pool, current_timestamp);
        unwrap_invariant_result(self.ticks.add(pool_key, index, &tick));

        self.tickmap
            .flip(true, index, pool_key.fee_tier.tick_spacing, pool_key);

        Ok(tick)
    }

    fn remove_tick(&mut self, key: PoolKey, tick: Tick) -> Result<(), InvariantError> {
        if !tick.liquidity_gross.is_zero() {
            contract_env::revert(InvariantErrorReturn::NotEmptyTickDeinitialization);
        }

        self.tickmap
            .flip(false, tick.index, key.fee_tier.tick_spacing, key);
        unwrap_invariant_result(self.ticks.remove(key, tick.index));
        Ok(())
    }

    fn calculate_swap(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<CalculateSwapResult, InvariantError> {
        let current_timestamp = contract_env::get_block_time();
        let config = self.config.get().unwrap_or_revert();
        if amount.is_zero() {
            contract_env::revert(InvariantErrorReturn::AmountIsZero);
        }

        let mut ticks: Vec<Tick> = vec![];

        let mut pool = unwrap_invariant_result(self.pools.get(pool_key));

        if x_to_y {
            if pool.sqrt_price <= sqrt_price_limit
                || sqrt_price_limit > SqrtPrice::new(U128::from(MAX_SQRT_PRICE))
            {
                contract_env::revert(InvariantErrorReturn::WrongLimit);
            }
        } else if pool.sqrt_price >= sqrt_price_limit
            || sqrt_price_limit < SqrtPrice::new(U128::from(MIN_SQRT_PRICE))
        {
            contract_env::revert(InvariantErrorReturn::WrongLimit);
        }

        let mut remaining_amount = amount;

        let mut total_amount_in = TokenAmount::new(U256::from(0));
        let mut total_amount_out = TokenAmount::new(U256::from(0));

        let event_start_sqrt_price = pool.sqrt_price;
        let mut event_fee_amount = TokenAmount::new(U256::from(0));

        let tick_limit = if x_to_y {
            get_min_tick(pool_key.fee_tier.tick_spacing)
        } else {
            get_max_tick(pool_key.fee_tier.tick_spacing)
        };

        while !remaining_amount.is_zero() {
            let (swap_limit, limiting_tick) = self.tickmap.get_closer_limit(
                sqrt_price_limit,
                x_to_y,
                pool.current_tick_index,
                pool_key.fee_tier.tick_spacing,
                pool_key,
            )?;

            let result = unwrap!(compute_swap_step(
                pool.sqrt_price,
                swap_limit,
                pool.liquidity,
                remaining_amount,
                by_amount_in,
                pool_key.fee_tier.fee,
            ));

            // make remaining amount smaller
            if by_amount_in {
                remaining_amount -= result.amount_in + result.fee_amount;
            } else {
                remaining_amount -= result.amount_out;
            }

            unwrap!(pool.add_fee(result.fee_amount, x_to_y, config.protocol_fee));
            event_fee_amount += result.fee_amount;

            pool.sqrt_price = result.next_sqrt_price;

            total_amount_in += result.amount_in + result.fee_amount;
            total_amount_out += result.amount_out;

            // Fail if price would go over swap limit
            if pool.sqrt_price == sqrt_price_limit && !remaining_amount.is_zero() {
                contract_env::revert(InvariantErrorReturn::PriceLimitReached);
            }

            let mut tick_update = {
                if let Some((tick_index, is_initialized)) = limiting_tick {
                    if is_initialized {
                        let tick = self.ticks.get(pool_key, tick_index)?;
                        UpdatePoolTick::TickInitialized(tick)
                    } else {
                        UpdatePoolTick::TickUninitialized(tick_index)
                    }
                } else {
                    UpdatePoolTick::NoTick
                }
            };

            let (amount_to_add, amount_after_tick_update, has_crossed) = pool.update_tick(
                result,
                swap_limit,
                &mut tick_update,
                remaining_amount,
                by_amount_in,
                x_to_y,
                current_timestamp,
                config.protocol_fee,
                pool_key.fee_tier,
            );

            remaining_amount = amount_after_tick_update;
            total_amount_in += amount_to_add;

            if let UpdatePoolTick::TickInitialized(tick) = tick_update {
                if has_crossed {
                    ticks.push(tick)
                }
            }

            let reached_tick_limit = match x_to_y {
                true => pool.current_tick_index <= tick_limit,
                false => pool.current_tick_index >= tick_limit,
            };

            if reached_tick_limit {
                return Err(InvariantError::TickLimitReached);
            }
        }

        if total_amount_out.get().is_zero() {
            contract_env::revert(InvariantErrorReturn::NoGainSwap);
        }

        Ok(CalculateSwapResult {
            amount_in: total_amount_in,
            amount_out: total_amount_out,
            start_sqrt_price: event_start_sqrt_price,
            target_sqrt_price: pool.sqrt_price,
            fee: event_fee_amount,
            pool,
            ticks,
        })
    }

    fn route(
        &mut self,
        is_swap: bool,
        amount_in: TokenAmount,
        swaps: Vec<SwapHop>,
    ) -> Result<TokenAmount, InvariantError> {
        let mut next_swap_amount = amount_in;

        for swap in swaps.iter() {
            let SwapHop {
                token_x,
                token_y,
                fee,
                tick_spacing,
                x_to_y,
            } = *swap;

            let pool_key = unwrap_invariant_result(PoolKey::new(
                token_x,
                token_y,
                unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing)),
            ));

            let sqrt_price_limit = if x_to_y {
                SqrtPrice::new(U128::from(MIN_SQRT_PRICE))
            } else {
                SqrtPrice::new(U128::from(MAX_SQRT_PRICE))
            };

            let result = unwrap_invariant_result(if is_swap {
                self.swap(
                    pool_key.token_x,
                    pool_key.token_y,
                    pool_key.fee_tier.fee.get(),
                    pool_key.fee_tier.tick_spacing,
                    x_to_y,
                    next_swap_amount.get(),
                    true,
                    sqrt_price_limit.get(),
                )
            } else {
                self.calculate_swap(pool_key, x_to_y, next_swap_amount, true, sqrt_price_limit)
            });

            next_swap_amount = result.amount_out;
        }

        Ok(next_swap_amount)
    }

    fn emit_create_position_event(
        &self,
        address: Address,
        pool: PoolKey,
        liquidity: Liquidity,
        lower_tick: i32,
        upper_tick: i32,
        current_sqrt_price: SqrtPrice,
    ) {
        let timestamp = contract_env::get_block_time();
        CreatePositionEvent {
            timestamp,
            address,
            pool,
            liquidity,
            lower_tick,
            upper_tick,
            current_sqrt_price,
        }
        .emit();
    }

    fn emit_remove_position_event(
        &self,
        address: Address,
        pool: PoolKey,
        liquidity: Liquidity,
        lower_tick: i32,
        upper_tick: i32,
        current_sqrt_price: SqrtPrice,
    ) {
        let timestamp = contract_env::get_block_time();
        RemovePositionEvent {
            timestamp,
            address,
            pool,
            liquidity,
            lower_tick,
            upper_tick,
            current_sqrt_price,
        }
        .emit();
    }

    fn emit_cross_tick_event(&self, address: Address, pool: PoolKey, indexes: Vec<i32>) {
        let timestamp = contract_env::get_block_time();
        CrossTickEvent {
            timestamp,
            address,
            pool,
            indexes,
        }
        .emit();
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_swap_event(
        &self,
        address: Address,
        pool: PoolKey,
        amount_in: TokenAmount,
        amount_out: TokenAmount,
        fee: TokenAmount,
        start_sqrt_price: SqrtPrice,
        target_sqrt_price: SqrtPrice,
        x_to_y: bool,
    ) {
        let timestamp = contract_env::get_block_time();
        SwapEvent {
            timestamp,
            address,
            pool,
            amount_in,
            amount_out,
            fee,
            start_sqrt_price,
            target_sqrt_price,
            x_to_y,
        }
        .emit()
    }
}

#[odra::module]
impl Entrypoints for Invariant {
    #[odra(init)]
    pub fn init(&mut self, fee: U128) {
        let protocol_fee = Percentage::new(fee);
        let caller = contract_env::caller();

        self.pool_keys.set(PoolKeys::default());
        self.fee_tiers.set(FeeTiers::default());

        self.config.set(InvariantConfig {
            admin: caller,
            protocol_fee,
        });
    }

    pub fn add_fee_tier(&mut self, fee: U128, tick_spacing: u32) -> Result<(), InvariantError> {
        let fee_tier = unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing));

        let caller = contract_env::caller();
        let config = self.config.get().unwrap_or_revert();
        let mut fee_tiers = self.fee_tiers.get().unwrap_or_revert();

        if caller != config.admin {
            contract_env::revert(InvariantErrorReturn::NotAdmin);
        }

        unwrap_invariant_result(fee_tiers.add(fee_tier));

        self.fee_tiers.set(fee_tiers);
        Ok(())
    }

    pub fn fee_tier_exist(&self, fee: U128, tick_spacing: u32) -> bool {
        let fee_tier = FeeTier::new(Percentage::new(fee), tick_spacing).unwrap();
        let fee_tiers = self.fee_tiers.get().unwrap_or_revert();
        fee_tiers.contains(fee_tier)
    }

    pub fn remove_fee_tier(&mut self, fee: U128, tick_spacing: u32) -> Result<(), InvariantError> {
        let fee_tier = unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing));
        let caller = contract_env::caller();
        let config = self.config.get().unwrap_or_revert();
        let mut fee_tiers = self.fee_tiers.get().unwrap_or_revert();

        if caller != config.admin {
            contract_env::revert(InvariantErrorReturn::NotAdmin);
        }

        unwrap_invariant_result(fee_tiers.remove(fee_tier));

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
        fee: U128,
        tick_spacing: u32,
        init_sqrt_price: U128,
        init_tick: i32,
    ) -> Result<(), InvariantError> {
        let fee_tier = unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing));
        let init_sqrt_price = SqrtPrice::new(init_sqrt_price);

        let current_timestamp = odra::contract_env::get_block_time();
        let mut pool_keys = self.pool_keys.get().unwrap_or_revert();
        let fee_tiers = self.fee_tiers.get().unwrap_or_revert();
        let config = self.config.get().unwrap_or_revert();

        if !fee_tiers.contains(fee_tier) {
            contract_env::revert(InvariantErrorReturn::FeeTierNotFound);
        };

        unwrap_invariant_result(
            check_tick(init_tick, fee_tier.tick_spacing)
                .map_err(|_| InvariantError::InvalidInitTick),
        );

        let pool_key = unwrap_invariant_result(PoolKey::new(token_0, token_1, fee_tier));

        if self.pools.get(pool_key).is_ok() {
            contract_env::revert(InvariantErrorReturn::PoolAlreadyExist);
        };

        let pool = unwrap_invariant_result(Pool::create(
            init_sqrt_price,
            init_tick,
            current_timestamp,
            fee_tier.tick_spacing,
            config.admin,
        ));

        unwrap_invariant_result(self.pools.add(pool_key, &pool));
        unwrap_invariant_result(pool_keys.add(pool_key));

        self.pool_keys.set(pool_keys);
        Ok(())
    }

    pub fn get_pool(
        &self,
        token_0: Address,
        token_1: Address,
        fee: U128,
        tick_spacing: u32,
    ) -> Result<Pool, InvariantError> {
        let fee_tier = unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing));
        let key: PoolKey = unwrap_invariant_result(PoolKey::new(token_0, token_1, fee_tier));
        let pool = unwrap_invariant_result(self.pools.get(key));

        Ok(pool)
    }

    pub fn get_pools(&self) -> Vec<PoolKey> {
        self.pool_keys.get().unwrap_or_revert().get_all()
    }

    pub fn get_protocol_fee(&self) -> Percentage {
        let config = self.config.get().unwrap_or_revert();
        config.protocol_fee
    }

    pub fn withdraw_protocol_fee(
        &mut self,
        token_0: Address,
        token_1: Address,
        fee: U128,
        tick_spacing: u32,
    ) -> Result<(), InvariantError> {
        let pool_key = unwrap_invariant_result(PoolKey::new(
            token_0,
            token_1,
            unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing)),
        ));

        let caller = contract_env::caller();
        let mut pool = unwrap_invariant_result(self.pools.get(pool_key));

        if caller != pool.fee_receiver {
            contract_env::revert(InvariantErrorReturn::NotFeeReceiver);
        }

        let (fee_protocol_token_x, fee_protocol_token_y) = pool.withdraw_protocol_fee(pool_key);

        Erc20Ref::at(&pool_key.token_x).transfer(&pool.fee_receiver, &fee_protocol_token_x.get());
        Erc20Ref::at(&pool_key.token_y).transfer(&pool.fee_receiver, &fee_protocol_token_y.get());

        unwrap_invariant_result(self.pools.update(pool_key, &pool));

        Ok(())
    }

    pub fn change_protocol_fee(&mut self, protocol_fee: U128) -> Result<(), InvariantError> {
        let protocol_fee = Percentage::new(protocol_fee);
        let caller = contract_env::caller();
        let mut config = self.config.get().unwrap_or_revert();

        if caller != config.admin {
            contract_env::revert(InvariantErrorReturn::NotAdmin);
        }

        config.protocol_fee = protocol_fee;

        self.config.set(config);

        Ok(())
    }

    pub fn change_fee_receiver(
        &mut self,
        token_0: Address,
        token_1: Address,
        fee: U128,
        tick_spacing: u32,
        fee_receiver: Address,
    ) -> Result<(), InvariantError> {
        let pool_key = unwrap_invariant_result(PoolKey::new(
            token_0,
            token_1,
            unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing)),
        ));
        let caller = contract_env::caller();
        let config = self.config.get().unwrap_or_revert();
        let mut pool = unwrap_invariant_result(self.pools.get(pool_key));

        if caller != config.admin {
            contract_env::revert(InvariantErrorReturn::NotAdmin);
        }

        pool.fee_receiver = fee_receiver;
        unwrap_invariant_result(self.pools.update(pool_key, &pool));

        Ok(())
    }

    pub fn is_tick_initialized(
        &self,
        token_0: Address,
        token_1: Address,
        fee: U128,
        tick_spacing: u32,
        index: i32,
    ) -> bool {
        let key = unwrap_invariant_result(PoolKey::new(
            token_0,
            token_1,
            unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing)),
        ));
        self.tickmap.get(index, key.fee_tier.tick_spacing, key)
    }

    pub fn get_tick(
        &self,
        token_0: Address,
        token_1: Address,
        fee: U128,
        tick_spacing: u32,
        index: i32,
    ) -> Result<Tick, InvariantError> {
        let key = unwrap_invariant_result(PoolKey::new(
            token_0,
            token_1,
            FeeTier::new(Percentage::new(fee), tick_spacing).unwrap(),
        ));
        self.ticks.get(key, index)
    }

    pub fn claim_fee(&mut self, index: u32) -> Result<(TokenAmount, TokenAmount), InvariantError> {
        let caller = odra::contract_env::caller();
        let current_timestamp = odra::contract_env::get_block_time();
        let mut position = unwrap_invariant_result(self.positions.get(caller, index));
        let mut lower_tick =
            unwrap_invariant_result(self.ticks.get(position.pool_key, position.lower_tick_index));
        let mut upper_tick =
            unwrap_invariant_result(self.ticks.get(position.pool_key, position.upper_tick_index));
        let mut pool = unwrap_invariant_result(self.pools.get(position.pool_key));

        let (x, y) = position.claim_fee(
            &mut pool,
            &mut upper_tick,
            &mut lower_tick,
            current_timestamp,
        );

        unwrap_invariant_result(self.positions.update(caller, index, &position));
        unwrap_invariant_result(self.pools.update(position.pool_key, &pool));
        unwrap_invariant_result(self.ticks.update(
            position.pool_key,
            position.lower_tick_index,
            &lower_tick,
        ));
        unwrap_invariant_result(self.ticks.update(
            position.pool_key,
            position.upper_tick_index,
            &upper_tick,
        ));

        if !x.get().is_zero() {
            Erc20Ref::at(&position.pool_key.token_x).transfer(&caller, &x.get());
        }

        if !y.get().is_zero() {
            Erc20Ref::at(&position.pool_key.token_y).transfer(&caller, &y.get());
        }

        Ok((x, y))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn create_position(
        &mut self,
        token_0: Address,
        token_1: Address,
        fee: U128,
        tick_spacing: u32,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: U256,
        slippage_limit_lower: U128,
        slippage_limit_upper: U128,
    ) -> Result<Position, InvariantError> {
        let pool_key = unwrap_invariant_result(PoolKey::new(
            token_0,
            token_1,
            unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing)),
        ));
        let liquidity_delta = Liquidity::new(liquidity_delta);
        let slippage_limit_lower = SqrtPrice::new(slippage_limit_lower);
        let slippage_limit_upper = SqrtPrice::new(slippage_limit_upper);

        let caller = contract_env::caller();
        let contract = contract_env::self_address();
        let current_timestamp = contract_env::get_block_time();
        let current_block_number = contract_env::get_block_time();

        // liquidity delta = 0 => return
        if liquidity_delta == Liquidity::new(U256::from(0)) {
            contract_env::revert(InvariantErrorReturn::ZeroLiquidity);
        }

        if lower_tick == upper_tick {
            contract_env::revert(InvariantErrorReturn::InvalidTickIndex);
        }

        let mut pool = unwrap_invariant_result(self.pools.get(pool_key));

        let mut lower_tick = self.ticks.get(pool_key, lower_tick).unwrap_or_else(|_| {
            unwrap_invariant_result(Self::create_tick(self, pool_key, lower_tick))
        });

        let mut upper_tick = self.ticks.get(pool_key, upper_tick).unwrap_or_else(|_| {
            unwrap_invariant_result(Self::create_tick(self, pool_key, upper_tick))
        });

        let (position, x, y) = unwrap_invariant_result(Position::create(
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
        ));

        unwrap_invariant_result(self.pools.update(pool_key, &pool));

        self.positions.add(caller, &position);

        unwrap_invariant_result(self.ticks.update(pool_key, lower_tick.index, &lower_tick));
        unwrap_invariant_result(self.ticks.update(pool_key, upper_tick.index, &upper_tick));

        Erc20Ref::at(&pool_key.token_x).transfer_from(&caller, &contract, &x.get());
        Erc20Ref::at(&pool_key.token_y).transfer_from(&caller, &contract, &y.get());

        self.emit_create_position_event(
            caller,
            pool_key,
            liquidity_delta,
            lower_tick.index,
            upper_tick.index,
            pool.sqrt_price,
        );

        Ok(position)
    }

    pub fn transfer_position(
        &mut self,
        index: u32,
        receiver: Address,
    ) -> Result<(), InvariantError> {
        let caller = contract_env::caller();

        unwrap_invariant_result(self.positions.transfer(caller, index, receiver));

        Ok(())
    }

    pub fn remove_position(
        &mut self,
        index: u32,
    ) -> Result<(TokenAmount, TokenAmount), InvariantError> {
        let caller = contract_env::caller();
        let current_timestamp = contract_env::get_block_time();

        let mut position = unwrap_invariant_result(self.positions.get(caller, index));
        let withdrawed_liquidity = position.liquidity;

        let mut lower_tick =
            unwrap_invariant_result(self.ticks.get(position.pool_key, position.lower_tick_index));

        let mut upper_tick =
            unwrap_invariant_result(self.ticks.get(position.pool_key, position.upper_tick_index));

        let pool = &mut unwrap_invariant_result(self.pools.get(position.pool_key));

        let (amount_x, amount_y, deinitialize_lower_tick, deinitialize_upper_tick) = position
            .remove(
                pool,
                current_timestamp,
                &mut lower_tick,
                &mut upper_tick,
                position.pool_key.fee_tier.tick_spacing,
            );

        unwrap_invariant_result(self.pools.update(position.pool_key, pool));

        if deinitialize_lower_tick {
            unwrap_invariant_result(self.remove_tick(position.pool_key, lower_tick));
        } else {
            unwrap_invariant_result(self.ticks.update(
                position.pool_key,
                position.lower_tick_index,
                &lower_tick,
            ));
        }

        if deinitialize_upper_tick {
            unwrap_invariant_result(self.remove_tick(position.pool_key, upper_tick));
        } else {
            unwrap_invariant_result(self.ticks.update(
                position.pool_key,
                position.upper_tick_index,
                &upper_tick,
            ));
        }

        unwrap_invariant_result(self.positions.remove(caller, index));

        Erc20Ref::at(&position.pool_key.token_x).transfer(&caller, &amount_x.get());
        Erc20Ref::at(&position.pool_key.token_y).transfer(&caller, &amount_y.get());

        self.emit_remove_position_event(
            caller,
            position.pool_key,
            withdrawed_liquidity,
            lower_tick.index,
            upper_tick.index,
            pool.sqrt_price,
        );

        Ok((amount_x, amount_y))
    }

    pub fn get_position(&mut self, owner: Address, index: u32) -> Result<Position, InvariantError> {
        self.positions.get(owner, index)
    }

    pub fn get_all_positions(&mut self, owner: Address) -> Vec<Position> {
        self.positions.get_all(owner)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn quote(
        &self,
        token_0: Address,
        token_1: Address,
        fee: U128,
        tick_spacing: u32,
        x_to_y: bool,
        amount: U256,
        by_amount_in: bool,
        sqrt_price_limit: U128,
    ) -> Result<QuoteResult, InvariantError> {
        let pool_key = unwrap_invariant_result(PoolKey::new(
            token_0,
            token_1,
            FeeTier::new(Percentage::new(fee), tick_spacing).unwrap(),
        ));
        let amount = TokenAmount::new(amount);
        let sqrt_price_limit = SqrtPrice::new(sqrt_price_limit);

        let calculate_swap_result = unwrap_invariant_result(self.calculate_swap(
            pool_key,
            x_to_y,
            amount,
            by_amount_in,
            sqrt_price_limit,
        ));

        Ok(QuoteResult {
            amount_in: calculate_swap_result.amount_in,
            amount_out: calculate_swap_result.amount_out,
            target_sqrt_price: calculate_swap_result.pool.sqrt_price,
            ticks: calculate_swap_result.ticks,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn swap(
        &mut self,
        token_0: Address,
        token_1: Address,
        fee: U128,
        tick_spacing: u32,
        x_to_y: bool,
        amount: U256,
        by_amount_in: bool,
        sqrt_price_limit: U128,
    ) -> Result<CalculateSwapResult, InvariantError> {
        let pool_key = unwrap_invariant_result(PoolKey::new(
            token_0,
            token_1,
            unwrap_invariant_result(FeeTier::new(Percentage::new(fee), tick_spacing)),
        ));
        let amount = TokenAmount::new(amount);
        let sqrt_price_limit = SqrtPrice::new(sqrt_price_limit);

        let caller = contract_env::caller();
        let contract = contract_env::self_address();

        let calculate_swap_result = unwrap_invariant_result(self.calculate_swap(
            pool_key,
            x_to_y,
            amount,
            by_amount_in,
            sqrt_price_limit,
        ));

        let mut crossed_tick_indexes: Vec<i32> = vec![];

        for tick in calculate_swap_result.ticks.iter() {
            unwrap_invariant_result(self.ticks.update(pool_key, tick.index, tick));
            crossed_tick_indexes.push(tick.index);
        }

        if !crossed_tick_indexes.is_empty() {
            self.emit_cross_tick_event(caller, pool_key, crossed_tick_indexes);
        }

        unwrap_invariant_result(self.pools.update(pool_key, &calculate_swap_result.pool));

        if x_to_y {
            Erc20Ref::at(&pool_key.token_x).transfer_from(
                &caller,
                &contract,
                &calculate_swap_result.amount_in.get(),
            );
            Erc20Ref::at(&pool_key.token_y)
                .transfer(&caller, &calculate_swap_result.amount_out.get());
        } else {
            Erc20Ref::at(&pool_key.token_y).transfer_from(
                &caller,
                &contract,
                &calculate_swap_result.amount_in.get(),
            );
            Erc20Ref::at(&pool_key.token_x)
                .transfer(&caller, &calculate_swap_result.amount_out.get());
        };

        self.emit_swap_event(
            caller,
            pool_key,
            calculate_swap_result.amount_in,
            calculate_swap_result.amount_out,
            calculate_swap_result.fee,
            calculate_swap_result.start_sqrt_price,
            calculate_swap_result.target_sqrt_price,
            x_to_y,
        );

        Ok(calculate_swap_result)
    }

    pub fn quote_route(
        &mut self,
        amount_in: U256,
        swaps: Vec<SwapHop>,
    ) -> Result<TokenAmount, InvariantError> {
        let amount_in = TokenAmount::new(amount_in);

        let amount_out = unwrap_invariant_result(self.route(false, amount_in, swaps));

        Ok(amount_out)
    }

    pub fn swap_route(
        &mut self,
        amount_in: U256,
        expected_amount_out: U256,
        slippage: U128,
        swaps: Vec<SwapHop>,
    ) -> Result<(), InvariantError> {
        let amount_in = TokenAmount::new(amount_in);
        let expected_amount_out = TokenAmount::new(expected_amount_out);
        let slippage = Percentage::new(slippage);

        let amount_out = unwrap_invariant_result(self.route(true, amount_in, swaps));

        let min_amount_out = calculate_min_amount_out(expected_amount_out, slippage);

        if amount_out < min_amount_out {
            contract_env::revert(InvariantErrorReturn::AmountUnderMinimumAmountOut);
        }

        Ok(())
    }
}
