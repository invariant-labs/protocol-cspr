use decimal::*;
use invariant_math::liquidity::Liquidity;
use odra::types::{U128, U256};
use odra::OdraType;
use traceable_result::*;

use crate::utils::{liquidity_to_uint, uint_to_liquidity};

#[derive(OdraType)]
pub struct Tick {
    pub index: i32,
    pub sign: bool,
    pub liquidity_change: U256,
    pub liquidity_gross: U256,
    pub sqrt_price: U128,
    pub fee_growth_outside_x: U128,
    pub fee_growth_outside_y: U128,
    pub seconds_per_liquidity_outside: U128,
    pub seconds_outside: u64,
}

impl Tick {
    // pub fn create(index: i32, pool: &Pool, current_timestamp: u64) -> Self {
    //     let below_current_tick = index <= pool.current_tick_index;

    //     Self {
    //         index,
    //         sign: true,
    //         sqrt_price: calculate_sqrt_price(index).unwrap(),
    //         fee_growth_outside_x: match below_current_tick {
    //             true => pool.fee_growth_global_x,
    //             false => FeeGrowth::new(0),
    //         },
    //         fee_growth_outside_y: match below_current_tick {
    //             true => pool.fee_growth_global_y,
    //             false => FeeGrowth::new(0),
    //         },
    //         seconds_outside: match below_current_tick {
    //             true => current_timestamp - pool.start_timestamp,
    //             false => 0,
    //         },
    //         seconds_per_liquidity_outside: match below_current_tick {
    //             true => pool.seconds_per_liquidity_global,
    //             false => SecondsPerLiquidity::new(0),
    //         },
    //         ..Self::default()
    //     }
    // }

    // pub fn cross(&mut self, pool: &mut Pool, current_timestamp: u64) -> TrackableResult<()> {
    //     self.fee_growth_outside_x = pool
    //         .fee_growth_global_x
    //         .unchecked_sub(self.fee_growth_outside_x);
    //     self.fee_growth_outside_y = pool
    //         .fee_growth_global_y
    //         .unchecked_sub(self.fee_growth_outside_y);

    //     let seconds_passed: u64 = current_timestamp
    //         .checked_sub(pool.start_timestamp)
    //         .ok_or_else(|| err!("current_timestamp - pool.start_timestamp underflow"))?;
    //     self.seconds_outside = seconds_passed.wrapping_sub(self.seconds_outside);

    //     if !pool.liquidity.is_zero() {
    //         ok_or_mark_trace!(pool.update_seconds_per_liquidity_global(current_timestamp))?;
    //     } else {
    //         pool.last_timestamp = current_timestamp;
    //     }
    //     self.seconds_per_liquidity_outside = pool
    //         .seconds_per_liquidity_global
    //         .unchecked_sub(self.seconds_per_liquidity_outside);

    //     // When going to higher tick net_liquidity should be added and for going lower subtracted
    //     if (pool.current_tick_index >= self.index) ^ self.sign {
    //         // trunk-ignore(clippy/assign_op_pattern)
    //         pool.liquidity = pool
    //             .liquidity
    //             .checked_add(self.liquidity_change)
    //             .map_err(|_| err!("pool.liquidity + tick.liquidity_change overflow"))?;
    //     } else {
    //         // trunk-ignore(clippy/assign_op_pattern)
    //         pool.liquidity = pool
    //             .liquidity
    //             .checked_sub(self.liquidity_change)
    //             .map_err(|_| err!("pool.liquidity - tick.liquidity_change underflow"))?
    //     }

    //     Ok(())
    // }

    pub fn update(
        &mut self,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
        is_upper: bool,
        is_deposit: bool,
    ) -> TrackableResult<()> {
        self.liquidity_gross = liquidity_to_uint(self.calculate_new_liquidity_gross_safely(
            is_deposit,
            liquidity_delta,
            max_liquidity_per_tick,
        )?);

        self.update_liquidity_change(liquidity_delta, is_deposit ^ is_upper);
        Ok(())
    }

    fn update_liquidity_change(&mut self, liquidity_delta: Liquidity, add: bool) {
        if self.sign ^ add {
            if { uint_to_liquidity(self.liquidity_change) } > liquidity_delta {
                self.liquidity_change =
                    liquidity_to_uint(uint_to_liquidity(self.liquidity_change) - liquidity_delta);
            } else {
                self.liquidity_change =
                    liquidity_to_uint(liquidity_delta - uint_to_liquidity(self.liquidity_change));
                self.sign = !self.sign;
            }
        } else {
            self.liquidity_change =
                liquidity_to_uint(uint_to_liquidity(self.liquidity_change) + liquidity_delta);
        }
    }

    fn calculate_new_liquidity_gross_safely(
        &self,
        sign: bool,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
    ) -> TrackableResult<Liquidity> {
        // validate in decrease liquidity case
        if !sign && { uint_to_liquidity(self.liquidity_gross) } < liquidity_delta {
            return Err(err!("InvalidTickLiquidity"));
        }
        let new_liquidity = match sign {
            true => uint_to_liquidity(self.liquidity_gross)
                .checked_add(liquidity_delta)
                .map_err(|_| err!("tick add liquidity overflow")),
            false => uint_to_liquidity(self.liquidity_gross)
                .checked_sub(liquidity_delta)
                .map_err(|_| err!("tick sun liquidity overflow")),
        }?;
        // validate in increase liquidity case
        if sign && new_liquidity >= max_liquidity_per_tick {
            return Err(err!("InvalidTickLiquidity"));
        }

        Ok(new_liquidity)
    }
}
