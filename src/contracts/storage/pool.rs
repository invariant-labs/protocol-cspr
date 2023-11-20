use traceable_result::*;
use decimal::*;
use decimal::num_traits::WrappingAdd;
use odra::types::{U128,U256, Address, casper_types::account::AccountHash};
use odra::OdraType;
use crate::contracts::Oracle;
use crate::contracts::PoolKey;
use crate::{U128T, U256T};
use invariant_math::sqrt_price::calculate_sqrt_price;
use invariant_math::calculate_amount_delta;
use invariant_math::token_amount::TokenAmount;
use invariant_math::percentage::Percentage;
use invariant_math::fee_growth::FeeGrowth;
use invariant_math::liquidity::Liquidity;
use invariant_math::sqrt_price::SqrtPrice;
use invariant_math::seconds_per_liquidity::{SecondsPerLiquidity, calculate_seconds_per_liquidity_inside};
use alloc::string::ToString;
#[derive(OdraType)]
pub struct Pool {
    pub liquidity: U256,
    pub sqrt_price: U128,
    pub current_tick_index: i32, // nearest tick below the current sqrt_price
    pub fee_growth_global_x: U128,
    pub fee_growth_global_y: U128,
    pub fee_protocol_token_x: U256,
    pub fee_protocol_token_y: U256,
    pub seconds_per_liquidity_global: U128,
    pub start_timestamp: u64,
    pub last_timestamp: u64,
    pub fee_receiver: Address,
    pub oracle_address: Oracle,
    pub oracle_initialized: bool,
}

impl Default for Pool {
    fn default() -> Self {
        Self{
            fee_receiver: Address::Account(AccountHash::new([0x0;32])),
            liquidity: U256::default(),
            sqrt_price:  U128::default(),
            current_tick_index: i32::default(),
            fee_growth_global_x: U128::default(),
            fee_growth_global_y: U128::default(),
            fee_protocol_token_x: U256::default(),
            fee_protocol_token_y: U256::default(),
            seconds_per_liquidity_global: U128::default(),
            start_timestamp: u64::default(),
            last_timestamp: u64::default(),
            oracle_address: Oracle::default(),
            oracle_initialized: bool::default(),
        }
    }
}


impl Pool {
    pub fn create(init_tick: i32, current_timestamp: u64, fee_receiver: Address) -> Self {
        Self {
            sqrt_price: U128::from_dec_str((unwrap!(calculate_sqrt_price(init_tick))).get().to_string().as_str()).unwrap(),
            current_tick_index: init_tick,
            start_timestamp: current_timestamp,
            last_timestamp: current_timestamp,
            fee_receiver,
            ..Self::default()
        }
    }

    pub fn add_fee(
        &mut self,
        amount: TokenAmount,
        in_x: bool,
        protocol_fee: Percentage,
    ) -> TrackableResult<()> {
        let protocol_fee = amount.big_mul_up(protocol_fee);

        let pool_fee = amount - protocol_fee;

        if (pool_fee.is_zero() && protocol_fee.is_zero()) || self.liquidity.is_zero() {
            return Ok(());
        }
        let liquidity = Liquidity::new(U256T(self.liquidity.0));

        let fee_growth = ok_or_mark_trace!(FeeGrowth::from_fee(liquidity, pool_fee))?;
        let formatted_fee_growth = U128(fee_growth.get().0);

        if in_x {
            self.fee_growth_global_x = self.fee_growth_global_x.wrapping_add(&formatted_fee_growth);
            self.fee_protocol_token_x = self.fee_protocol_token_x + U256(protocol_fee.get().0);
        } else {
            self.fee_growth_global_y = self.fee_growth_global_y.wrapping_add(&formatted_fee_growth);
            self.fee_protocol_token_y = self.fee_protocol_token_y + U256(protocol_fee.get().0);
        }
        Ok(())
    }

    pub fn update_liquidity(
        &mut self,
        liquidity_delta: Liquidity,
        liquidity_sign: bool,
        upper_tick: i32,
        lower_tick: i32,
    ) -> TrackableResult<(TokenAmount, TokenAmount)> {
        let (x, y, update_liquidity) = ok_or_mark_trace!(calculate_amount_delta(
            self.current_tick_index,
            SqrtPrice::new(U128T(self.sqrt_price.0)),
            liquidity_delta,
            liquidity_sign,
            upper_tick,
            lower_tick,
        ))?;

        if !update_liquidity {
            return Ok((x, y));
        }

        if liquidity_sign {
            self.liquidity = self
                .liquidity
                .checked_add(U256(liquidity_delta.get().0))
                .ok_or_else(|| err!("update_liquidity: liquidity + liquidity_delta overflow"))?;
            Ok((x, y))
        } else {
            self.liquidity = self
                .liquidity
                .checked_sub(U256(liquidity_delta.get().0))
                .ok_or_else(|| err!("update_liquidity: liquidity - liquidity_delta underflow"))?;
            Ok((x, y))
        }
    }

    pub fn update_seconds_per_liquidity_global(
        &mut self,
        current_timestamp: u64,
    ) -> TrackableResult<()> {
        // let seconds_per_liquidity_global =
        //     SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
        //         self.liquidity,
        //         current_timestamp,
        //         self.last_timestamp,
        //     )?;

        // self.seconds_per_liquidity_global = self
        //     .seconds_per_liquidity_global
        //     .unchecked_add(seconds_per_liquidity_global);
        self.last_timestamp = current_timestamp;
        Ok(())
    }

    pub fn update_seconds_per_liquidity_inside(
        &mut self,
        tick_lower: i32,
        tick_lower_seconds_per_liquidity_outside: SecondsPerLiquidity,
        tick_upper: i32,
        tick_upper_seconds_per_liquidity_outside: SecondsPerLiquidity,
        current_timestamp: u64,
    ) -> TrackableResult<SecondsPerLiquidity> {
        if !self.liquidity.is_zero() {
            ok_or_mark_trace!(self.update_seconds_per_liquidity_global(current_timestamp))?;
        } else {
            self.last_timestamp = current_timestamp;
        }

        ok_or_mark_trace!(calculate_seconds_per_liquidity_inside(
            tick_lower,
            tick_upper,
            self.current_tick_index,
            tick_lower_seconds_per_liquidity_outside,
            tick_upper_seconds_per_liquidity_outside,
            SecondsPerLiquidity::new(U128T(self.seconds_per_liquidity_global.0)),
        ))
    }

    pub fn withdraw_protocol_fee(&mut self, _pool_key: PoolKey) -> (TokenAmount, TokenAmount) {
        let fee_protocol_token_x = self.fee_protocol_token_x;
        let fee_protocol_token_y = self.fee_protocol_token_y;

        self.fee_protocol_token_x = U256::from(0);
        self.fee_protocol_token_y = U256::from(0);

        (TokenAmount::new(U256T(fee_protocol_token_x.0)), TokenAmount::new(U256T(fee_protocol_token_y.0)))
    }


    // TODO - tick needed
    // pub fn cross_tick(
    //     &mut self,
    //     result: SwapResult,
    //     swap_limit: SqrtPrice,
    //     limiting_tick: Option<(i32, Option<&mut Tick>)>,
    //     remaining_amount: &mut TokenAmount,
    //     by_amount_in: bool,
    //     x_to_y: bool,
    //     current_timestamp: u64,
    //     total_amount_in: &mut TokenAmount,
    //     protocol_fee: Percentage,
    //     fee_tier: FeeTier,
    // ) {
    //     if result.next_sqrt_price == swap_limit && limiting_tick.is_some() {
    //         let (tick_index, tick) = limiting_tick.unwrap();

    //         let is_enough_amount_to_cross = unwrap!(is_enough_amount_to_change_price(
    //             *remaining_amount,
    //             result.next_sqrt_price,
    //             self.liquidity,
    //             fee_tier.fee,
    //             by_amount_in,
    //             x_to_y,
    //         ));

    //         // crossing tick
    //         if tick.is_some() {
    //             if !x_to_y || is_enough_amount_to_cross {
    //                 let _ = tick.unwrap().cross(self, current_timestamp);
    //             } else if !remaining_amount.is_zero() {
    //                 if by_amount_in {
    //                     self.add_fee(*remaining_amount, x_to_y, protocol_fee)
    //                         .unwrap();
    //                     *total_amount_in += *remaining_amount
    //                 }
    //                 *remaining_amount = TokenAmount(0);
    //             }
    //         }

    //         // set tick to limit (below if price is going down, because current tick should always be below price)
    //         self.current_tick_index = if x_to_y && is_enough_amount_to_cross {
    //             tick_index - fee_tier.tick_spacing as i32
    //         } else {
    //             tick_index
    //         };
    //     } else {
    //         self.current_tick_index = unwrap!(get_tick_at_sqrt_price(
    //             result.next_sqrt_price,
    //             fee_tier.tick_spacing
    //         ));
    //     };
    // }


    // TODO - tickmap needed
    // pub fn swap_step(
    //     &mut self,
    //     remaining_amount: &mut TokenAmount,
    //     // tickmap: Tickmap,
    //     sqrt_price_limit: SqrtPrice,
    //     x_to_y: bool,
    //     by_amount_in: bool,
    //     total_amount_in: &mut TokenAmount,
    //     total_amount_out: &mut TokenAmount,
    // ) -> Option<(i32, bool)> {
    //     // let (swap_limit, limiting_tick) = tickmap.get_closer_limit(
    //     //     sqrt_price_limit,
    //     //     x_to_y,
    //     //     self.current_tick_index,
    //     //     self.tick_spacing,
    //     // );

    //     // let result = unwrap!(compute_swap_step(
    //     //     self.sqrt_price,
    //     //     swap_limit,
    //     //     self.liquidity,
    //     //     *remaining_amount,
    //     //     by_amount_in,
    //     //     self.fee,
    //     // ));

    //     // // make remaining amount smaller
    //     // if by_amount_in {
    //     //     *remaining_amount -= result.amount_in + result.fee_amount;
    //     // } else {
    //     //     *remaining_amount -= result.amount_out;
    //     // }

    //     // unwrap!(self.add_fee(result.fee_amount, x_to_y));

    //     // self.sqrt_price = result.next_sqrt_price;

    //     // *total_amount_in += result.amount_in + result.fee_amount;
    //     // *total_amount_out += result.amount_out;

    //     // // Fail if price would go over swap limit
    //     // if { self.sqrt_price } == sqrt_price_limit && !remaining_amount.is_zero() {
    //     //     panic!("PriceLimitReached");
    //     // }

    //     // limiting_tick
    // }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let init_tick = 100;
        let current_timestamp = 100;
        let fee_receiver =Address::Account(AccountHash::new([0x0;32]));

        let pool = Pool::create(init_tick, current_timestamp, fee_receiver);

        assert_eq!(pool.sqrt_price, U128::from_dec_str((calculate_sqrt_price(init_tick)).unwrap().get().to_string().as_str()).unwrap());
        assert_eq!(pool.current_tick_index, init_tick);
        assert_eq!(pool.start_timestamp, current_timestamp);
        assert_eq!(pool.last_timestamp, current_timestamp);
        assert_eq!(pool.fee_receiver, fee_receiver);
    }

    #[test]
    fn test_add_fee() {
        // fee is set to 20%
        let protocol_fee = Percentage::from_scale(2, 1);
        let pool = Pool {
            liquidity: U256::from(10u128.pow(Liquidity::scale() as u32)),
            ..Default::default()
        };
        // in_x
        {
            let mut pool = pool.clone();
            let amount = TokenAmount::from_integer(6);
            pool.add_fee(amount, true, protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, U128::from(40000000000000000000000000000u128));
            assert_eq!({ pool.fee_growth_global_y }, U128::from(0));
            assert_eq!({ pool.fee_protocol_token_x }, U256::from(2));
            assert_eq!({ pool.fee_protocol_token_y }, U256::from(0));
        }
        // in_y
        {
            let mut pool = pool.clone();
            let amount = TokenAmount::from_integer(200);
            pool.add_fee(amount, false, protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, U128::from(0));
            assert_eq!({ pool.fee_growth_global_y }, U128::from(1600000000000000000000000000000u128));
            assert_eq!({ pool.fee_protocol_token_x }, U256::from(0));
            assert_eq!({ pool.fee_protocol_token_y }, U256::from(40));
        }
        // some new comment
        {
            let mut pool = pool.clone();
            let amount = TokenAmount::new(U256T::from(1));
            pool.add_fee(amount, true, protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, U128::from(0));
            assert_eq!({ pool.fee_growth_global_y }, U128::from(0));
            assert_eq!({ pool.fee_protocol_token_x }, U256::from(1));
            assert_eq!({ pool.fee_protocol_token_y },U256::from(0));
        }
        //DOMAIN
        let max_amount = TokenAmount::max_instance();
        // let min_amount = TokenAmount(1);
        let max_liquidity = Liquidity::max_instance();
        // let min_liquidity = Liquidity::new(1);
        let max_protocol_fee = Percentage::from_integer(1);
        let min_protocol_fee = Percentage::from_integer(0);

        // max fee max amount max liquidity in x
        {
            let mut pool = Pool {
                liquidity: U256(max_liquidity.get().0),
                ..Default::default()
            };
            pool.add_fee(max_amount, true, max_protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, U128::from(0));
            assert_eq!({ pool.fee_growth_global_y }, U128::from(0));
            assert_eq!(
                { pool.fee_protocol_token_x },
                U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap(),
            );
            assert_eq!({ pool.fee_protocol_token_y }, U256::from(0));
        }
        // max fee max amount max liquidity in y
        {
            let mut pool = Pool {
                liquidity: U256(max_liquidity.get().0),
                ..Default::default()
            };
            pool.add_fee(max_amount, false, max_protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, U128::from(0));
            assert_eq!({ pool.fee_growth_global_y }, U128::from(0));
            assert_eq!({ pool.fee_protocol_token_x }, U256::from(0));
            assert_eq!(
                { pool.fee_protocol_token_y },
                U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap(),
            );
        }
        // min fee max amount max liquidity in x
        {
            let mut pool = Pool {
                liquidity: U256(max_liquidity.get().0),
                ..Default::default()
            };
            pool.add_fee(max_amount, true, min_protocol_fee).unwrap();
            assert_eq!(
                { pool.fee_growth_global_x },
                U128::from(1000000000000000000000000000000000u128)
            );
            assert_eq!({ pool.fee_growth_global_y }, U128::from(0));
            assert_eq!({ pool.fee_protocol_token_x }, U256::from(0));
            assert_eq!({ pool.fee_protocol_token_y }, U256::from(0));
        }
    }
   #[test]
    fn test_update_liquidity() {
        // Add liquidity
        // current tick between lower tick and upper tick
        {
            let mut pool = Pool {
                liquidity: U256::from(0),
                sqrt_price: U128::from(1000140000000_000000000000u128),
                current_tick_index: 2,
                ..Default::default()
            };

            let liquidity_delta = Liquidity::from_integer(5_000_000);
            let liquidity_sign = true;
            let upper_tick = 3;
            let lower_tick = 0;

            let (x, y) = pool
                .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                .unwrap();

            assert_eq!(x, TokenAmount::new(U256T::from(51)));
            assert_eq!(y, TokenAmount::new(U256T::from(700)));

            assert_eq!(pool.liquidity, U256(liquidity_delta.get().0))
        }
        {
            let mut pool = Pool {
                liquidity: U256::from(0),
                sqrt_price: U128::from(1000140000000_000000000000u128),
                current_tick_index: 2,
                ..Default::default()
            };

            let liquidity_delta = Liquidity::from_integer(5_000_000);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 0;

            let (x, y) = pool
                .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                .unwrap();

            assert_eq!(x, TokenAmount::new(U256T::from(300)));
            assert_eq!(y, TokenAmount::new(U256T::from(700)));
            assert_eq!(pool.liquidity, U256(liquidity_delta.get().0))
        }
        // delta liquidity = 0
        // No Change
        {
            {
                let mut pool = Pool {
                    liquidity: U256::from(10u128.pow(Liquidity::scale() as u32)),
                    sqrt_price: U128::from(1000140000000_000000000000u128),
                    current_tick_index: 6,
                    ..Default::default()
                };

                let liquidity_delta = Liquidity::from_integer(12);
                let liquidity_sign = true;
                let upper_tick = 4;
                let lower_tick = 0;

                let (x, y) = pool
                    .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                    .unwrap();

                assert_eq!(x, TokenAmount::new(U256T::from(0)));
                assert_eq!(y, TokenAmount::new(U256T::from(1)));
                assert_eq!(pool.liquidity, U256::from(10u128.pow(Liquidity::scale() as u32)),)
            }
            {
                let mut pool = Pool {
                    liquidity: U256::from(10u128.pow(Liquidity::scale() as u32)),
                    sqrt_price: U128::from(1000140000000_000000000000u128),
                    current_tick_index: -2,
                    ..Default::default()
                };

                let liquidity_delta = Liquidity::from_integer(12);
                let liquidity_sign = true;
                let upper_tick = 4;
                let lower_tick = 0;

                let (x, y) = pool
                    .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                    .unwrap();

                assert_eq!(x, TokenAmount::new(U256T::from(1)));
                assert_eq!(y, TokenAmount::new(U256T::from(0)));
                assert_eq!(pool.liquidity, U256::from(10u128.pow(Liquidity::scale() as u32)))
            }
        }
        // Remove Liquidity
        {
            let mut pool = Pool {
                liquidity: U256::from(10 * 10u128.pow(Liquidity::scale() as u32)),
                current_tick_index: 2,
                sqrt_price: U128::from(1),
                ..Default::default()
            };

            let liquidity_delta = Liquidity::from_integer(5);
            let liquidity_sign = false;
            let upper_tick = 3;
            let lower_tick = 1;

            let (x, y) = pool
                .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                .unwrap();

            assert_eq!(x, TokenAmount::new(U256T::from(2500375009372499999999997u128)));
            assert_eq!(y, TokenAmount::new(U256T::from(5)));
            assert_eq!(pool.liquidity, U256::from(5 * 10u128.pow(Liquidity::scale() as u32)),)
        }
    }

    // #[test]
    // fn test_update_seconds_per_liquidity_inside() {
    //     let mut tick_lower = Tick {
    //         index: 0,
    //         seconds_per_liquidity_outside: SecondsPerLiquidity::new(3012300000),
    //         ..Default::default()
    //     };
    //     let mut tick_upper = Tick {
    //         index: 10,
    //         seconds_per_liquidity_outside: SecondsPerLiquidity::new(2030400000),
    //         ..Default::default()
    //     };
    //     let mut pool = Pool {
    //         liquidity: Liquidity::from_integer(1000),
    //         start_timestamp: 0,
    //         last_timestamp: 0,
    //         seconds_per_liquidity_global: SecondsPerLiquidity::new(0),
    //         ..Default::default()
    //     };
    //     let mut current_timestamp = 0;

    //     {
    //         current_timestamp += 100;
    //         pool.current_tick_index = -10;
    //         let seconds_per_liquidity_inside = pool.update_seconds_per_liquidity_inside(
    //             tick_lower.index,
    //             tick_lower.seconds_per_liquidity_outside,
    //             tick_upper.index,
    //             tick_upper.seconds_per_liquidity_outside,
    //             current_timestamp,
    //         );
    //         assert_eq!(seconds_per_liquidity_inside.unwrap().get(), 981900000);
    //     }
    //     {
    //         current_timestamp += 100;
    //         pool.current_tick_index = 0;
    //         let seconds_per_liquidity_inside = pool.update_seconds_per_liquidity_inside(
    //             tick_lower.index,
    //             tick_lower.seconds_per_liquidity_outside,
    //             tick_upper.index,
    //             tick_upper.seconds_per_liquidity_outside,
    //             current_timestamp,
    //         );
    //         assert_eq!(
    //             seconds_per_liquidity_inside.unwrap().get(),
    //             199999999999994957300000
    //         );
    //     }
    //     {
    //         current_timestamp += 100;
    //         tick_lower.seconds_per_liquidity_outside = SecondsPerLiquidity::new(2012333200);
    //         tick_upper.seconds_per_liquidity_outside = SecondsPerLiquidity::new(3012333310);
    //         pool.current_tick_index = 20;
    //         let seconds_per_liquidity_inside = pool.update_seconds_per_liquidity_inside(
    //             tick_lower.index,
    //             tick_lower.seconds_per_liquidity_outside,
    //             tick_upper.index,
    //             tick_upper.seconds_per_liquidity_outside,
    //             current_timestamp,
    //         );
    //         assert_eq!(seconds_per_liquidity_inside.unwrap().get(), 1000000110);
    //     }
    //     {
    //         current_timestamp += 100;
    //         tick_lower.seconds_per_liquidity_outside = SecondsPerLiquidity::new(201233320000);
    //         tick_upper.seconds_per_liquidity_outside = SecondsPerLiquidity::new(301233331000);
    //         pool.current_tick_index = 20;
    //         let seconds_per_liquidity_inside = pool.update_seconds_per_liquidity_inside(
    //             tick_lower.index,
    //             tick_lower.seconds_per_liquidity_outside,
    //             tick_upper.index,
    //             tick_upper.seconds_per_liquidity_outside,
    //             current_timestamp,
    //         );
    //         assert_eq!(seconds_per_liquidity_inside.unwrap().get(), 100000011000);
    //     }
    //     {
    //         current_timestamp += 100;
    //         tick_lower.seconds_per_liquidity_outside = SecondsPerLiquidity::new(201233320000);
    //         tick_upper.seconds_per_liquidity_outside = SecondsPerLiquidity::new(301233331000);
    //         pool.current_tick_index = -20;
    //         let seconds_per_liquidity_inside = pool.update_seconds_per_liquidity_inside(
    //             tick_lower.index,
    //             tick_lower.seconds_per_liquidity_outside,
    //             tick_upper.index,
    //             tick_upper.seconds_per_liquidity_outside,
    //             current_timestamp,
    //         );
    //         assert_eq!(
    //             seconds_per_liquidity_inside.unwrap().get(),
    //             340282366920938463463374607331768200456
    //         );
    //         assert_eq!(
    //             pool.seconds_per_liquidity_global.get(),
    //             500000000000000000000000
    //         );
    //     }
    //     // updates timestamp
    //     {
    //         current_timestamp += 100;
    //         pool.liquidity = Liquidity::new(0);
    //         let seconds_per_liquidity_inside = pool.update_seconds_per_liquidity_inside(
    //             tick_lower.index,
    //             tick_lower.seconds_per_liquidity_outside,
    //             tick_upper.index,
    //             tick_upper.seconds_per_liquidity_outside,
    //             current_timestamp,
    //         );
    //         assert_eq!(pool.last_timestamp, current_timestamp);
    //         assert_eq!(
    //             seconds_per_liquidity_inside.unwrap().get(),
    //             340282366920938463463374607331768200456
    //         );
    //         assert_eq!(
    //             pool.seconds_per_liquidity_global.get(),
    //             500000000000000000000000
    //         );
    //     }
    //     // L > 0
    //     {
    //         current_timestamp += 100;
    //         pool.liquidity = Liquidity::from_integer(1000);
    //         let seconds_per_liquidity_inside = pool.update_seconds_per_liquidity_inside(
    //             tick_lower.index,
    //             tick_lower.seconds_per_liquidity_outside,
    //             tick_upper.index,
    //             tick_upper.seconds_per_liquidity_outside,
    //             current_timestamp,
    //         );
    //         assert_eq!(pool.last_timestamp, current_timestamp);
    //         assert_eq!(
    //             seconds_per_liquidity_inside.unwrap().get(),
    //             340282366920938463463374607331768200456
    //         );
    //         assert_eq!(
    //             pool.seconds_per_liquidity_global.get(),
    //             600000000000000000000000
    //         );
    //     }
    //     // L == 0
    //     {
    //         current_timestamp += 100;
    //         pool.liquidity = Liquidity::new(0);
    //         let seconds_per_liquidity_inside = pool.update_seconds_per_liquidity_inside(
    //             tick_lower.index,
    //             tick_lower.seconds_per_liquidity_outside,
    //             tick_upper.index,
    //             tick_upper.seconds_per_liquidity_outside,
    //             current_timestamp,
    //         );
    //         assert_eq!(pool.last_timestamp, current_timestamp);
    //         assert_eq!(
    //             seconds_per_liquidity_inside.unwrap().get(),
    //             340282366920938463463374607331768200456
    //         );
    //         assert_eq!(
    //             pool.seconds_per_liquidity_global.get(),
    //             600000000000000000000000
    //         );
    //     }
    // }

}