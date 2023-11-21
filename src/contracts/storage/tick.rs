use decimal::*;
use invariant_math::fee_growth::FeeGrowth;
use invariant_math::liquidity::Liquidity;
use invariant_math::seconds_per_liquidity::SecondsPerLiquidity;
use invariant_math::sqrt_price::{calculate_sqrt_price, SqrtPrice};
use invariant_math::{U128T, U256T};
use odra::types::{U128, U256};
use odra::OdraType;
use traceable_result::*;

use crate::contracts::Pool;

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

impl Default for Tick {
    fn default() -> Self {
        Tick {
            index: 0i32,
            sign: false,
            liquidity_change: U256::from(0),
            liquidity_gross: U256::from(0),
            sqrt_price: U128(SqrtPrice::from_integer(1).get().0),
            fee_growth_outside_x: U128::from(0),
            fee_growth_outside_y: U128::from(0),
            seconds_per_liquidity_outside: U128::from(0),
            seconds_outside: 0u64,
        }
    }
}

impl Tick {
    pub fn create(index: i32, pool: &Pool, current_timestamp: u64) -> Self {
        let below_current_tick = index <= pool.current_tick_index;

        Self {
            index,
            sign: true,
            sqrt_price: U128(calculate_sqrt_price(index).unwrap().get().0),
            fee_growth_outside_x: match below_current_tick {
                true => pool.fee_growth_global_x,
                false => U128(FeeGrowth::new(U128T::from(0)).get().0),
            },
            fee_growth_outside_y: match below_current_tick {
                true => pool.fee_growth_global_y,
                false => U128(FeeGrowth::new(U128T::from(0)).get().0),
            },
            seconds_outside: match below_current_tick {
                true => current_timestamp - pool.start_timestamp,
                false => 0,
            },
            seconds_per_liquidity_outside: match below_current_tick {
                true => pool.seconds_per_liquidity_global,
                false => U128(SecondsPerLiquidity::new(U128T::from(0)).get().0),
            },
            ..Self::default()
        }
    }

    pub fn cross(&mut self, pool: &mut Pool, current_timestamp: u64) -> TrackableResult<()> {
        self.fee_growth_outside_x = U128(
            FeeGrowth::new(U128T(pool.fee_growth_global_x.0))
                .unchecked_sub(FeeGrowth::new(U128T(self.fee_growth_outside_x.0)))
                .get()
                .0,
        );
        self.fee_growth_outside_y = U128(
            FeeGrowth::new(U128T(pool.fee_growth_global_y.0))
                .unchecked_sub(FeeGrowth::new(U128T(self.fee_growth_outside_y.0)))
                .get()
                .0,
        );

        let seconds_passed: u64 = current_timestamp
            .checked_sub(pool.start_timestamp)
            .ok_or_else(|| err!("current_timestamp - pool.start_timestamp underflow"))?;
        self.seconds_outside = seconds_passed.wrapping_sub(self.seconds_outside);

        if !pool.liquidity.is_zero() {
            ok_or_mark_trace!(pool.update_seconds_per_liquidity_global(current_timestamp))?;
        } else {
            pool.last_timestamp = current_timestamp;
        }
        self.seconds_per_liquidity_outside = U128(
            SecondsPerLiquidity::new(U128T(pool.seconds_per_liquidity_global.0))
                .unchecked_sub(SecondsPerLiquidity::new(U128T(
                    self.seconds_per_liquidity_outside.0,
                )))
                .get()
                .0,
        );

        // When going to higher tick net_liquidity should be added and for going lower subtracted
        if (pool.current_tick_index >= self.index) ^ self.sign {
            // trunk-ignore(clippy/assign_op_pattern)
            pool.liquidity = U256(
                Liquidity::new(U256T(pool.liquidity.0))
                    .checked_add(Liquidity::new(U256T(self.liquidity_change.0)))
                    .map_err(|_| err!("pool.liquidity + tick.liquidity_change overflow"))?
                    .get()
                    .0,
            );
        } else {
            // trunk-ignore(clippy/assign_op_pattern)
            pool.liquidity = U256(
                Liquidity::new(U256T(pool.liquidity.0))
                    .checked_sub(Liquidity::new(U256T(self.liquidity_change.0)))
                    .map_err(|_| err!("pool.liquidity - tick.liquidity_change underflow"))?
                    .get()
                    .0,
            )
        }

        Ok(())
    }

    pub fn update(
        &mut self,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
        is_upper: bool,
        is_deposit: bool,
    ) -> TrackableResult<()> {
        self.liquidity_gross = U256(
            self.calculate_new_liquidity_gross_safely(
                is_deposit,
                liquidity_delta,
                max_liquidity_per_tick,
            )?
            .get()
            .0,
        );

        self.update_liquidity_change(liquidity_delta, is_deposit ^ is_upper);
        Ok(())
    }

    fn update_liquidity_change(&mut self, liquidity_delta: Liquidity, add: bool) {
        if self.sign ^ add {
            if { Liquidity::new(U256T(self.liquidity_change.0)) } > liquidity_delta {
                self.liquidity_change = U256(
                    (Liquidity::new(U256T(self.liquidity_change.0)) - liquidity_delta)
                        .get()
                        .0,
                );
            } else {
                self.liquidity_change = U256(
                    (liquidity_delta - Liquidity::new(U256T(self.liquidity_change.0)))
                        .get()
                        .0,
                );
                self.sign = !self.sign;
            }
        } else {
            self.liquidity_change = U256(
                (Liquidity::new(U256T(self.liquidity_change.0)) + liquidity_delta)
                    .get()
                    .0,
            );
        }
    }

    fn calculate_new_liquidity_gross_safely(
        &self,
        sign: bool,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
    ) -> TrackableResult<Liquidity> {
        // validate in decrease liquidity case
        if !sign && { Liquidity::new(U256T(self.liquidity_gross.0)) } < liquidity_delta {
            return Err(err!("InvalidTickLiquidity"));
        }
        let new_liquidity = match sign {
            true => Liquidity::new(U256T(self.liquidity_gross.0))
                .checked_add(liquidity_delta)
                .map_err(|_| err!("tick add liquidity overflow")),
            false => Liquidity::new(U256T(self.liquidity_gross.0))
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

#[cfg(test)]
mod tests {
    use decimal::{Decimal, Factories};

    use invariant_math::{
        fee_growth::FeeGrowth, math::calculate_max_liquidity_per_tick, U128T, U256T,
    };

    use super::*;

    // #[test]
    // fn test_cross() {
    //     {
    //         let mut pool = Pool {
    //             fee_growth_global_x: FeeGrowth::new(45),
    //             fee_growth_global_y: FeeGrowth::new(35),
    //             liquidity: Liquidity::from_integer(4),
    //             last_timestamp: 15,
    //             start_timestamp: 4,
    //             seconds_per_liquidity_global: SecondsPerLiquidity::from_integer(11),
    //             current_tick_index: 7,
    //             ..Default::default()
    //         };
    //         let mut tick = Tick {
    //             fee_growth_outside_x: FeeGrowth::new(30),
    //             fee_growth_outside_y: FeeGrowth::new(25),
    //             index: 3,
    //             seconds_outside: 5,
    //             seconds_per_liquidity_outside: SecondsPerLiquidity::new(3),
    //             liquidity_change: Liquidity::from_integer(1),
    //             ..Default::default()
    //         };
    //         let result_pool = Pool {
    //             fee_growth_global_x: FeeGrowth::new(45),
    //             fee_growth_global_y: FeeGrowth::new(35),
    //             liquidity: Liquidity::from_integer(5),
    //             last_timestamp: 315360015,
    //             start_timestamp: 4,
    //             seconds_per_liquidity_global: SecondsPerLiquidity::new(
    //                 89840000000000000000000000000000,
    //             ),
    //             current_tick_index: 7,
    //             ..Default::default()
    //         };
    //         let result_tick = Tick {
    //             fee_growth_outside_x: FeeGrowth::new(15),
    //             fee_growth_outside_y: FeeGrowth::new(10),
    //             index: 3,
    //             seconds_outside: 315360006,
    //             seconds_per_liquidity_outside: SecondsPerLiquidity::new(
    //                 89839999999999999999999999999997,
    //             ),
    //             liquidity_change: Liquidity::from_integer(1),
    //             ..Default::default()
    //         };
    //         tick.cross(&mut pool, 315360015).ok();
    //         assert_eq!(tick, result_tick);
    //         assert_eq!(pool, result_pool);
    //     }
    //     {
    //         // let mut pool = Pool {
    //         let mut pool = Pool {
    //             fee_growth_global_x: FeeGrowth::new(68),
    //             fee_growth_global_y: FeeGrowth::new(59),
    //             liquidity: Liquidity::new(0),
    //             last_timestamp: 9,
    //             start_timestamp: 34,
    //             seconds_per_liquidity_global: SecondsPerLiquidity::new(32),
    //             current_tick_index: 4,
    //             ..Default::default()
    //         };
    //         let mut tick = Tick {
    //             fee_growth_outside_x: FeeGrowth::new(42),
    //             fee_growth_outside_y: FeeGrowth::new(14),
    //             index: 9,
    //             seconds_outside: 41,
    //             seconds_per_liquidity_outside: SecondsPerLiquidity::new(23),
    //             liquidity_change: Liquidity::new(0),
    //             ..Default::default()
    //         };
    //         // let result_pool = Pool {
    //         let result_pool = Pool {
    //             fee_growth_global_x: FeeGrowth::new(68),
    //             fee_growth_global_y: FeeGrowth::new(59),
    //             liquidity: Liquidity::new(0),
    //             last_timestamp: 315360000,
    //             start_timestamp: 34,
    //             seconds_per_liquidity_global: SecondsPerLiquidity::new(32),
    //             current_tick_index: 4,
    //             ..Default::default()
    //         };
    //         // let result_tick = Tick {
    //         let result_tick = Tick {
    //             fee_growth_outside_x: FeeGrowth::new(26),
    //             fee_growth_outside_y: FeeGrowth::new(45),
    //             index: 9,
    //             seconds_outside: 315359925,
    //             seconds_per_liquidity_outside: SecondsPerLiquidity::new(9),
    //             liquidity_change: Liquidity::from_integer(0),
    //             ..Default::default()
    //         };

    //         tick.cross(&mut pool, 315360000).ok();
    //         assert_eq!(tick, result_tick);
    //         assert_eq!(pool, result_pool);
    //     }
    //     // fee_growth_outside should underflow
    //     {
    //         let mut pool = Pool {
    //             fee_growth_global_x: FeeGrowth::new(3402),
    //             fee_growth_global_y: FeeGrowth::new(3401),
    //             liquidity: Liquidity::from_integer(14),
    //             last_timestamp: 9,
    //             start_timestamp: 15,
    //             seconds_per_liquidity_global: SecondsPerLiquidity::new(22),
    //             current_tick_index: 9,
    //             ..Default::default()
    //         };
    //         let mut tick = Tick {
    //             fee_growth_outside_x: FeeGrowth::new(26584),
    //             fee_growth_outside_y: FeeGrowth::new(1256588),
    //             index: 45,
    //             seconds_outside: 74,
    //             seconds_per_liquidity_outside: SecondsPerLiquidity::new(23),
    //             liquidity_change: Liquidity::new(10),
    //             ..Default::default()
    //         };
    //         let result_pool = Pool {
    //             fee_growth_global_x: FeeGrowth::new(3402),
    //             fee_growth_global_y: FeeGrowth::new(3401),
    //             liquidity: Liquidity::new(13999990),
    //             last_timestamp: 31536000,
    //             start_timestamp: 15,
    //             seconds_per_liquidity_global: SecondsPerLiquidity::new(
    //                 2252570785714285714285714285736,
    //             ),
    //             current_tick_index: 9,
    //             ..Default::default()
    //         };
    //         let result_tick = Tick {
    //             fee_growth_outside_x: FeeGrowth::new(340282366920938463463374607431768188274),
    //             fee_growth_outside_y: FeeGrowth::new(340282366920938463463374607431766958269),
    //             index: 45,
    //             seconds_outside: 31535911,
    //             seconds_per_liquidity_outside: SecondsPerLiquidity::new(
    //                 2252570785714285714285714285713,
    //             ),
    //             liquidity_change: Liquidity::new(10),
    //             ..Default::default()
    //         };

    //         tick.cross(&mut pool, 31536000).ok();
    //         assert_eq!(tick, result_tick);
    //         assert_eq!(pool, result_pool);
    //     }
    //     // seconds_per_liquidity_outside should underflow
    //     {
    //         let mut pool = Pool {
    //             fee_growth_global_x: FeeGrowth::new(145),
    //             fee_growth_global_y: FeeGrowth::new(364),
    //             liquidity: Liquidity::new(14),
    //             last_timestamp: 16,
    //             start_timestamp: 15,
    //             seconds_per_liquidity_global: SecondsPerLiquidity::new(354),
    //             current_tick_index: 9,
    //             ..Default::default()
    //         };
    //         let mut tick = Tick {
    //             fee_growth_outside_x: FeeGrowth::new(99),
    //             fee_growth_outside_y: FeeGrowth::new(256),
    //             index: 45,
    //             seconds_outside: 74,
    //             seconds_per_liquidity_outside: SecondsPerLiquidity::new(35),
    //             liquidity_change: Liquidity::new(10),
    //             ..Default::default()
    //         };
    //         let result_pool = Pool {
    //             fee_growth_global_x: FeeGrowth::new(145),
    //             fee_growth_global_y: FeeGrowth::new(364),
    //             liquidity: Liquidity::new(4),
    //             last_timestamp: 315360000,
    //             start_timestamp: 15,
    //             seconds_per_liquidity_global: SecondsPerLiquidity::new(
    //                 22525713142857142857142857142857143211,
    //             ),
    //             current_tick_index: 9,
    //             ..Default::default()
    //         };
    //         let result_tick = Tick {
    //             fee_growth_outside_x: FeeGrowth::new(46),
    //             fee_growth_outside_y: FeeGrowth::new(108),
    //             index: 45,
    //             seconds_outside: 315359911,
    //             seconds_per_liquidity_outside: SecondsPerLiquidity::new(
    //                 22525713142857142857142857142857143176,
    //             ),
    //             liquidity_change: Liquidity::new(10),
    //             ..Default::default()
    //         };

    //         tick.cross(&mut pool, 315360000).ok();
    //         assert_eq!(tick, result_tick);
    //         assert_eq!(pool, result_pool);
    //     }
    // }

    #[test]
    fn test_update_liquidity_change() {
        // update when tick sign and sign of liquidity change are the same
        {
            let mut tick = Tick {
                sign: true,
                liquidity_change: U256(Liquidity::from_integer(2).get().0),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = true;
            tick.update_liquidity_change(liquidity_delta, add);

            assert_eq!(tick.sign, true);
            assert_eq!(
                { Liquidity::new(U256T(tick.liquidity_change.0)) },
                Liquidity::from_integer(5)
            );
        }
        {
            let mut tick = Tick {
                sign: false,
                liquidity_change: U256(Liquidity::from_integer(2).get().0),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = false;
            tick.update_liquidity_change(liquidity_delta, add);

            assert_eq!(tick.sign, false);
            assert_eq!(
                { Liquidity::new(U256T(tick.liquidity_change.0)) },
                Liquidity::from_integer(5)
            );
        }
        // update when tick sign and sign of liquidity change are different
        {
            let mut tick = Tick {
                sign: true,
                liquidity_change: U256(Liquidity::from_integer(2).get().0),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = false;
            tick.update_liquidity_change(liquidity_delta, add);

            assert_eq!(tick.sign, false);
            assert_eq!(
                { Liquidity::new(U256T(tick.liquidity_change.0)) },
                Liquidity::from_integer(1)
            );
        }
        {
            let mut tick = Tick {
                sign: false,
                liquidity_change: U256(Liquidity::from_integer(2).get().0),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = true;
            tick.update_liquidity_change(liquidity_delta, add);

            assert_eq!(tick.sign, true);
            assert_eq!(
                { Liquidity::new(U256T(tick.liquidity_change.0)) },
                Liquidity::from_integer(1)
            );
        }
    }

    #[test]
    fn test_update() {
        let max_liquidity = Liquidity::new(U256T::from(u128::MAX));
        {
            let mut tick = Tick {
                index: 0,
                sign: true,
                liquidity_change: U256(Liquidity::from_integer(2).get().0),
                liquidity_gross: U256(Liquidity::from_integer(2).get().0),
                fee_growth_outside_x: U128(FeeGrowth::from_integer(2).get().0),
                fee_growth_outside_y: U128(FeeGrowth::from_integer(2).get().0),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(1);
            let is_upper = false;
            let is_deposit = true;

            tick.update(liquidity_delta, max_liquidity, is_upper, is_deposit)
                .unwrap();

            assert_eq!(tick.sign, true);
            assert_eq!(
                { Liquidity::new(U256T(tick.liquidity_change.0)) },
                Liquidity::from_integer(3)
            );
            assert_eq!(
                { Liquidity::new(U256T(tick.liquidity_gross.0)) },
                Liquidity::from_integer(3)
            );
            assert_eq!(
                { FeeGrowth::new(U128T(tick.fee_growth_outside_x.0)) },
                FeeGrowth::from_integer(2)
            );
            assert_eq!(
                { FeeGrowth::new(U128T(tick.fee_growth_outside_y.0)) },
                FeeGrowth::from_integer(2)
            );
        }
        {
            let mut tick = Tick {
                index: 5,
                sign: true,
                liquidity_change: U256(Liquidity::from_integer(3).get().0),
                liquidity_gross: U256(Liquidity::from_integer(7).get().0),
                fee_growth_outside_x: U128(FeeGrowth::from_integer(13).get().0),
                fee_growth_outside_y: U128(FeeGrowth::from_integer(11).get().0),
                ..Default::default()
            };
            let liquidity_delta: Liquidity = Liquidity::from_integer(1);
            let is_upper: bool = true;
            let is_deposit: bool = true;

            tick.update(liquidity_delta, max_liquidity, is_upper, is_deposit)
                .unwrap();

            assert_eq!(tick.sign, true);
            assert_eq!(
                { Liquidity::new(U256T(tick.liquidity_change.0)) },
                Liquidity::from_integer(2)
            );
            assert_eq!(
                { Liquidity::new(U256T(tick.liquidity_gross.0)) },
                Liquidity::from_integer(8)
            );
            assert_eq!(
                { FeeGrowth::new(U128T(tick.fee_growth_outside_x.0)) },
                FeeGrowth::from_integer(13)
            );
            assert_eq!(
                { FeeGrowth::new(U128T(tick.fee_growth_outside_y.0)) },
                FeeGrowth::from_integer(11)
            );
        }
        // exceed max tick liquidity
        {
            let mut tick = Tick {
                // index: 5,
                sign: true,
                liquidity_change: U256(Liquidity::from_integer(100_000).get().0),
                liquidity_gross: U256(Liquidity::from_integer(100_000).get().0),
                fee_growth_outside_x: U128(FeeGrowth::from_integer(1000).get().0),
                fee_growth_outside_y: U128(FeeGrowth::from_integer(1000).get().0),
                ..Default::default()
            };

            let max_liquidity_per_tick = calculate_max_liquidity_per_tick(1);
            let liquidity_delta = max_liquidity_per_tick + Liquidity::new(U256T::from(1));
            let result = tick.update(liquidity_delta, max_liquidity_per_tick, false, true);
            assert!(result.is_err());
        }
    }
}
