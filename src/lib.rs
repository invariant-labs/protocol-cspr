#![no_std]

extern crate alloc;

pub mod contracts;
pub mod math;
pub mod token;

use odra_modules::erc20::Erc20Ref;

#[cfg(test)]
pub mod e2e;

use crate::contracts::errors::InvariantError;
use crate::math::{check_tick, percentage::Percentage, sqrt_price::SqrtPrice};
use contracts::events::*;
use contracts::{
    FeeTier, FeeTiers, Pool, PoolKey, PoolKeys, Pools, Position, Positions, State, Tick, Tickmap,
    Ticks,
};
use decimal::Decimal;
use math::clamm::{compute_swap_step, SwapResult};
use math::get_tick_at_sqrt_price;
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

impl Invariant {
    fn create_tick(&mut self, pool_key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        let current_timestamp = contract_env::get_block_time();

        check_tick(index, pool_key.fee_tier.tick_spacing)
            .map_err(|_| InvariantError::InvalidTickIndexOrTickSpacing)?;

        let pool = self.pools.get(pool_key)?;

        let tick = Tick::create(index, &pool, current_timestamp);
        self.ticks.add(pool_key, index, &tick)?;

        self.tickmap
            .flip(true, index, pool_key.fee_tier.tick_spacing, pool_key);

        Ok(tick)
    }

    fn remove_tick(&mut self, key: PoolKey, tick: Tick) -> Result<(), InvariantError> {
        if !tick.liquidity_gross.is_zero() {
            return Err(InvariantError::NotEmptyTickDeinitialization);
        }

        self.tickmap
            .flip(false, tick.index, key.fee_tier.tick_spacing, key);
        self.ticks.remove(key, tick.index)?;
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
        let state = self.state.get().unwrap_or_revert();
        if amount.is_zero() {
            return Err(InvariantError::AmountIsZero);
        }

        let mut ticks: Vec<Tick> = vec![];

        let mut pool = self.pools.get(pool_key)?;

        if x_to_y {
            if pool.sqrt_price <= sqrt_price_limit
                || sqrt_price_limit > SqrtPrice::new(U128::from(MAX_SQRT_PRICE))
            {
                return Err(InvariantError::WrongLimit);
            }
        } else if pool.sqrt_price >= sqrt_price_limit
            || sqrt_price_limit < SqrtPrice::new(U128::from(MIN_SQRT_PRICE))
        {
            return Err(InvariantError::WrongLimit);
        }

        let mut remaining_amount = amount;

        let mut total_amount_in = TokenAmount::new(U256::from(0));
        let mut total_amount_out = TokenAmount::new(U256::from(0));

        let event_start_sqrt_price = pool.sqrt_price;
        let mut event_fee_amount = TokenAmount::new(U256::from(0));

        while !remaining_amount.is_zero() {
            let (swap_limit, limiting_tick) = self.tickmap.get_closer_limit(
                sqrt_price_limit,
                x_to_y,
                pool.current_tick_index,
                pool_key.fee_tier.tick_spacing,
                pool_key,
            );

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

            unwrap!(pool.add_fee(result.fee_amount, x_to_y, state.protocol_fee));
            event_fee_amount += result.fee_amount;

            pool.sqrt_price = result.next_sqrt_price;

            total_amount_in += result.amount_in + result.fee_amount;
            total_amount_out += result.amount_out;

            // Fail if price would go over swap limit
            if pool.sqrt_price == sqrt_price_limit && !remaining_amount.is_zero() {
                return Err(InvariantError::PriceLimitReached);
            }

            if let Some((tick_index, is_initialized)) = limiting_tick {
                if is_initialized {
                    let mut tick = self.ticks.get(pool_key, tick_index)?;

                    let (amount_to_add, has_crossed) = pool.cross_tick(
                        result,
                        swap_limit,
                        &mut tick,
                        &mut remaining_amount,
                        by_amount_in,
                        x_to_y,
                        current_timestamp,
                        state.protocol_fee,
                        pool_key.fee_tier,
                    );

                    total_amount_in += amount_to_add;
                    if has_crossed {
                        ticks.push(tick);
                    }
                }
            } else {
                pool.current_tick_index = unwrap!(get_tick_at_sqrt_price(
                    result.next_sqrt_price,
                    pool_key.fee_tier.tick_spacing
                ));
            }
        }

        if total_amount_out.get().is_zero() {
            return Err(InvariantError::NoGainSwap);
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

        let (fee_protocol_token_x, fee_protocol_token_y) = pool.withdraw_protocol_fee(pool_key);

        Erc20Ref::at(&pool_key.token_x).transfer(&pool.fee_receiver, &fee_protocol_token_x.get());
        Erc20Ref::at(&pool_key.token_y).transfer(&pool.fee_receiver, &fee_protocol_token_y.get());

        self.pools.update(pool_key, &pool)?;

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

    pub fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool {
        self.tickmap.get(index, key.fee_tier.tick_spacing, key)
    }

    pub fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        self.ticks.get(key, index)
    }

    pub fn claim_fee(&mut self, index: u32) -> Result<(TokenAmount, TokenAmount), InvariantError> {
        let caller = odra::contract_env::caller();
        let current_timestamp = odra::contract_env::get_block_time();
        let mut position = self.positions.get(caller, index)?;
        let mut lower_tick = self
            .ticks
            .get(position.pool_key, position.lower_tick_index)?;
        let mut upper_tick = self
            .ticks
            .get(position.pool_key, position.upper_tick_index)?;
        let mut pool = self.pools.get(position.pool_key)?;

        let (x, y) = position.claim_fee(
            &mut pool,
            &mut upper_tick,
            &mut lower_tick,
            current_timestamp,
        );

        self.positions.update(caller, index, &position)?;
        self.pools.update(position.pool_key, &pool)?;
        self.ticks
            .update(position.pool_key, position.lower_tick_index, &lower_tick)?;
        self.ticks
            .update(position.pool_key, position.upper_tick_index, &upper_tick)?;

        if x.get().is_zero() {
            Erc20Ref::at(&position.pool_key.token_x).transfer(&caller, &x.get());
        }

        if y.get().is_zero() {
            Erc20Ref::at(&position.pool_key.token_y).transfer(&caller, &y.get());
        }

        Ok((x, y))
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
            .ticks
            .get(pool_key, lower_tick)
            .unwrap_or_else(|_| Self::create_tick(self, pool_key, lower_tick).unwrap());

        let mut upper_tick = self
            .ticks
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

        self.positions.add(caller, &position);

        self.ticks.update(pool_key, lower_tick.index, &lower_tick)?;
        self.ticks.update(pool_key, upper_tick.index, &upper_tick)?;

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

        self.positions.transfer(caller, index, receiver)?;

        Ok(())
    }

    pub fn remove_position(
        &mut self,
        index: u32,
    ) -> Result<(TokenAmount, TokenAmount), InvariantError> {
        let caller = contract_env::caller();
        let current_timestamp = contract_env::get_block_time();

        let mut position = self.positions.get(caller, index)?;

        let mut lower_tick = self
            .ticks
            .get(position.pool_key, position.lower_tick_index)?;

        let mut upper_tick = self
            .ticks
            .get(position.pool_key, position.upper_tick_index)?;

        let pool = &mut self.pools.get(position.pool_key)?;

        let (amount_x, amount_y, deinitialize_lower_tick, deinitialize_upper_tick) = position
            .remove(
                pool,
                current_timestamp,
                &mut lower_tick,
                &mut upper_tick,
                position.pool_key.fee_tier.tick_spacing,
            );

        self.pools.update(position.pool_key, pool)?;

        if deinitialize_lower_tick {
            self.remove_tick(position.pool_key, lower_tick)?;
        } else {
            self.ticks
                .update(position.pool_key, position.lower_tick_index, &lower_tick)?;
        }

        if deinitialize_upper_tick {
            self.remove_tick(position.pool_key, upper_tick)?;
        } else {
            self.ticks
                .update(position.pool_key, position.upper_tick_index, &upper_tick)?;
        }

        self.positions.remove(caller, index)?;

        Erc20Ref::at(&position.pool_key.token_x).transfer(&caller, &amount_x.get());
        Erc20Ref::at(&position.pool_key.token_y).transfer(&caller, &amount_y.get());

        self.emit_remove_position_event(
            caller,
            position.pool_key,
            position.liquidity,
            lower_tick.index,
            upper_tick.index,
            pool.sqrt_price,
        );

        Ok((amount_x, amount_y))
    }

    pub fn get_position(&mut self, index: u32) -> Result<Position, InvariantError> {
        let caller = contract_env::caller();

        self.positions.get(caller, index)
    }

    pub fn get_all_positions(&mut self) -> Vec<Position> {
        let caller = contract_env::caller();

        self.positions.get_all(caller)
    }

    pub fn quote(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<QuoteResult, InvariantError> {
        let calculate_swap_result =
            self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

        Ok(QuoteResult {
            amount_in: calculate_swap_result.amount_in,
            amount_out: calculate_swap_result.amount_out,
            target_sqrt_price: calculate_swap_result.pool.sqrt_price,
            ticks: calculate_swap_result.ticks,
        })
    }

    pub fn swap(
        &mut self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<CalculateSwapResult, InvariantError> {
        let caller = contract_env::caller();
        let contract = contract_env::self_address();

        let calculate_swap_result =
            self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

        let mut crossed_tick_indexes: Vec<i32> = vec![];

        for tick in calculate_swap_result.ticks.iter() {
            self.ticks.update(pool_key, tick.index, tick)?;
            crossed_tick_indexes.push(tick.index);
        }

        if !crossed_tick_indexes.is_empty() {
            self.emit_cross_tick_event(caller, pool_key, crossed_tick_indexes);
        }

        self.pools.update(pool_key, &calculate_swap_result.pool)?;

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
}
