use super::{Pool, Tick, Tickmap};
use crate::contracts::PoolKey;
use crate::{U128T, U256T};
use decimal::*;
use invariant_math::calculate_max_liquidity_per_tick;
use invariant_math::fee_growth::calculate_fee_growth_inside;
use invariant_math::{fee_growth::FeeGrowth, liquidity::Liquidity, token_amount::TokenAmount};
use odra::types::{U128, U256};
use odra::OdraType;
use traceable_result::*;
#[derive(OdraType, Default)]
pub struct Position {
    pub pool_key: PoolKey,
    pub liquidity: U256,
    pub lower_tick_index: i32,
    pub upper_tick_index: i32,
    pub fee_growth_inside_x: U128,
    pub fee_growth_inside_y: U128,
    pub seconds_per_liquidity_inside: U128,
    pub last_block_number: u64,
    pub tokens_owed_x: U256,
    pub tokens_owed_y: U256,
}

impl Position {
    pub fn modify(
        &mut self,
        pool: &mut Pool,
        upper_tick: &mut Tick,
        lower_tick: &mut Tick,
        liquidity_delta: Liquidity,
        add: bool,
        current_timestamp: u64,
        tick_spacing: u16,
    ) -> TrackableResult<(TokenAmount, TokenAmount)> {
        if !pool.liquidity.is_zero() {
            // ok_or_mark_trace!(pool.update_seconds_per_liquidity_global(current_timestamp))?;
        } else {
            pool.last_timestamp = current_timestamp;
        }

        // calculate dynamically limit allows easy modification
        let max_liquidity_per_tick = calculate_max_liquidity_per_tick(tick_spacing);

        // update initialized tick
        // lower_tick.update(liquidity_delta, max_liquidity_per_tick, false, add)?;

        // upper_tick.update(liquidity_delta, max_liquidity_per_tick, true, add)?;

        // update fee inside position
        let (fee_growth_inside_x, fee_growth_inside_y) = calculate_fee_growth_inside(
            lower_tick.index,
            FeeGrowth::new(U128T(lower_tick.fee_growth_outside_x.0)),
            FeeGrowth::new(U128T(lower_tick.fee_growth_outside_y.0)),
            upper_tick.index,
            FeeGrowth::new(U128T(upper_tick.fee_growth_outside_x.0)),
            FeeGrowth::new(U128T(upper_tick.fee_growth_outside_y.0)),
            pool.current_tick_index,
            FeeGrowth::new(U128T(pool.fee_growth_global_x.0)),
            FeeGrowth::new(U128T(pool.fee_growth_global_y.0)),
        );

        self.update(
            add,
            liquidity_delta,
            fee_growth_inside_x,
            fee_growth_inside_y,
        )?;

        // calculate tokens amounts and update pool liquidity
        // ok_or_mark_trace!(pool.update_liquidity(
        //     liquidity_delta,
        //     add,
        //     upper_tick.index,
        //     lower_tick.index
        // ))
    }

    pub fn update(
        &mut self,
        sign: bool,
        liquidity_delta: Liquidity,
        fee_growth_inside_x: FeeGrowth,
        fee_growth_inside_y: FeeGrowth,
    ) -> TrackableResult<()> {
        if liquidity_delta.v == U256T::from(0) && self.liquidity == U256::from(0) {
            return Err(err!("EmptyPositionPokes"));
        }

        // calculate accumulated fee
        let tokens_owed_x = ok_or_mark_trace!(fee_growth_inside_x
            .unchecked_sub(FeeGrowth::new(U128T(self.fee_growth_inside_x.0)))
            .to_fee(Liquidity::new(U256T(self.liquidity.0))))?;
        let tokens_owed_y = ok_or_mark_trace!(fee_growth_inside_y
            .unchecked_sub(FeeGrowth::new(U128T(self.fee_growth_inside_y.0)))
            .to_fee(Liquidity::new(U256T(self.liquidity.0))))?;

        self.liquidity =
            ok_or_mark_trace!(self.calculate_new_liquidity_safely(sign, liquidity_delta))?;
        self.fee_growth_inside_x = U128(fee_growth_inside_x.get().0);
        self.fee_growth_inside_y = U128(fee_growth_inside_y.get().0);

        self.tokens_owed_x = self.tokens_owed_x + U256(tokens_owed_x.get().0);
        self.tokens_owed_y = self.tokens_owed_y + U256(tokens_owed_y.get().0);
        Ok(())
    }

    fn calculate_new_liquidity_safely(
        &mut self,
        sign: bool,
        liquidity_delta: Liquidity,
    ) -> TrackableResult<U256> {
        // validate in decrease liquidity case
        if !sign && { Liquidity::new(U256T(self.liquidity.0)) } < liquidity_delta {
            return Err(err!("InsufficientLiquidity"));
        }

        match sign {
            true => self
                .liquidity
                .checked_add(U256(liquidity_delta.get().0))
                .ok_or_else(|| err!("position add liquidity overflow")),
            false => self
                .liquidity
                .checked_sub(U256(liquidity_delta.get().0))
                .ok_or_else(|| err!("position sub liquidity underflow")),
        }
    }

    // pub fn claim_fee(
    //     &mut self,
    //     pool: &mut Pool,
    //     upper_tick: &mut Tick,
    //     lower_tick: &mut Tick,
    //     current_timestamp: u64,
    // ) -> (TokenAmount, TokenAmount) {
    //     unwrap!(self.modify(
    //         pool,
    //         upper_tick,
    //         lower_tick,
    //         Liquidity::new(0),
    //         true,
    //         current_timestamp,
    //         self.pool_key.fee_tier.tick_spacing
    //     ));

    //     let tokens_owed_x = self.tokens_owed_x;
    //     let tokens_owed_y = self.tokens_owed_y;

    //     self.tokens_owed_x = TokenAmount(0);
    //     self.tokens_owed_y = TokenAmount(0);

    //     (tokens_owed_x, tokens_owed_y)
    // }
    // pub fn create(
    //     pool: &mut Pool,
    //     pool_key: PoolKey,
    //     lower_tick: &mut Tick,
    //     upper_tick: &mut Tick,
    //     current_timestamp: u64,
    //     // tickmap: &mut Tickmap,
    //     liquidity_delta: Liquidity,
    //     slippage_limit_lower: SqrtPrice,
    //     slippage_limit_upper: SqrtPrice,
    //     block_number: u64,
    //     tick_spacing: u16,
    // ) -> Result<(Self, TokenAmount, TokenAmount), ContractErrors> {
    //     if pool.sqrt_price < slippage_limit_lower || pool.sqrt_price > slippage_limit_upper {
    //         return Err(ContractErrors::PriceLimitReached);
    //     }

    //     // if !tickmap.get(lower_tick.index, pool.tick_spacing) {
    //     //     tickmap.flip(true, lower_tick.index, pool.tick_spacing)
    //     // }
    //     // if !tickmap.get(upper_tick.index, pool.tick_spacing) {
    //     //     tickmap.flip(true, upper_tick.index, pool.tick_spacing)
    //     // }

    //     // init position
    //     let mut position = Position {
    //         pool_key,
    //         liquidity: Liquidity::new(0),
    //         lower_tick_index: lower_tick.index,
    //         upper_tick_index: upper_tick.index,
    //         fee_growth_inside_x: FeeGrowth::new(0),
    //         fee_growth_inside_y: FeeGrowth::new(0),
    //         seconds_per_liquidity_inside: SecondsPerLiquidity::new(0),
    //         last_block_number: block_number,
    //         tokens_owed_x: TokenAmount::new(0),
    //         tokens_owed_y: TokenAmount::new(0),
    //     };

    //     let (required_x, required_y) = unwrap!(position.modify(
    //         pool,
    //         upper_tick,
    //         lower_tick,
    //         liquidity_delta,
    //         true,
    //         current_timestamp,
    //         tick_spacing
    //     ));

    //     Ok((position, required_x, required_y))
    // }

    // pub fn remove(
    //     &mut self,
    //     pool: &mut Pool,
    //     current_timestamp: u64,
    //     lower_tick: &mut Tick,
    //     upper_tick: &mut Tick,
    //     tick_spacing: u16,
    // ) -> (TokenAmount, TokenAmount, bool, bool) {
    //     let liquidity_delta = self.liquidity;
    //     let (mut amount_x, mut amount_y) = unwrap!(self.modify(
    //         pool,
    //         upper_tick,
    //         lower_tick,
    //         liquidity_delta,
    //         false,
    //         current_timestamp,
    //         tick_spacing
    //     ));

    //     amount_x += self.tokens_owed_x;
    //     amount_y += self.tokens_owed_y;

    //     let deinitialize_lower_tick = lower_tick.liquidity_gross.is_zero();
    //     let deinitialize_upper_tick = upper_tick.liquidity_gross.is_zero();

    //     (
    //         amount_x,
    //         amount_y,
    //         deinitialize_lower_tick,
    //         deinitialize_upper_tick,
    //     )
    // }

    // pub fn update_seconds_per_liquidity(
    //     &mut self,
    //     pool: Pool,
    //     lower_tick: Tick,
    //     upper_tick: Tick,
    //     current_timestamp: u64,
    // ) {
    //     self.seconds_per_liquidity_inside = unwrap!(calculate_seconds_per_liquidity_inside(
    //         lower_tick.index,
    //         upper_tick.index,
    //         pool.current_tick_index,
    //         lower_tick.seconds_per_liquidity_outside,
    //         upper_tick.seconds_per_liquidity_outside,
    //         pool.seconds_per_liquidity_global,
    //     ));
    //     self.last_block_number = current_timestamp;
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_new_liquidity_safely() {
        // negative liquidity error
        {
            let mut position = Position {
                liquidity: U256::from(10u128.pow(Liquidity::scale() as u32)),
                ..Default::default()
            };
            let sign: bool = false;
            let liquidity_delta = Liquidity::from_integer(2);

            let result = position.calculate_new_liquidity_safely(sign, liquidity_delta);

            assert!(result.is_err());
        }
        // adding liquidity
        {
            let mut position = Position {
                liquidity: U256::from(2 * 10u128.pow(Liquidity::scale() as u32)),
                ..Default::default()
            };
            let sign: bool = true;
            let liquidity_delta = Liquidity::from_integer(2);

            let new_liquidity = position
                .calculate_new_liquidity_safely(sign, liquidity_delta)
                .unwrap();

            assert_eq!(
                new_liquidity,
                U256::from(4 * 10u128.pow(Liquidity::scale() as u32))
            );
        }
        // subtracting liquidity
        {
            let mut position = Position {
                liquidity: U256::from(2 * 10u128.pow(Liquidity::scale() as u32)),
                ..Default::default()
            };
            let sign: bool = false;
            let liquidity_delta = Liquidity::from_integer(2);

            let new_liquidity = position
                .calculate_new_liquidity_safely(sign, liquidity_delta)
                .unwrap();

            assert_eq!(new_liquidity, U256::from(0));
        }
    }

    #[test]
    fn test_update() {
        // Disable empty position pokes error
        {
            let mut position = Position {
                liquidity: U256::from(0),
                ..Default::default()
            };
            let sign = true;
            let liquidity_delta = Liquidity::from_integer(0);
            let fee_growth_inside_x = FeeGrowth::from_integer(1);
            let fee_growth_inside_y = FeeGrowth::from_integer(1);

            let result = position.update(
                sign,
                liquidity_delta,
                fee_growth_inside_x,
                fee_growth_inside_y,
            );

            assert!(result.is_err());
        }
        // zero liquidity fee shouldn't change
        {
            let mut position = Position {
                liquidity: U256::from(0),
                fee_growth_inside_x: U128::from(4 * 10u128.pow(FeeGrowth::scale() as u32)),
                fee_growth_inside_y: U128::from(4 * 10u128.pow(FeeGrowth::scale() as u32)),
                tokens_owed_x: U256::from(100),
                tokens_owed_y: U256::from(100),
                ..Default::default()
            };
            let sign = true;
            let liquidity_delta = Liquidity::from_integer(1);
            let fee_growth_inside_x = FeeGrowth::from_integer(5);
            let fee_growth_inside_y = FeeGrowth::from_integer(5);

            position
                .update(
                    sign,
                    liquidity_delta,
                    fee_growth_inside_x,
                    fee_growth_inside_y,
                )
                .unwrap();

            assert_eq!(
                { position.liquidity },
                U256::from(1 * 10u128.pow(Liquidity::scale() as u32))
            );
            assert_eq!(
                { position.fee_growth_inside_x },
                U128::from(5 * 10u128.pow(FeeGrowth::scale() as u32))
            );
            assert_eq!(
                { position.fee_growth_inside_y },
                U128::from(5 * 10u128.pow(FeeGrowth::scale() as u32))
            );
            assert_eq!({ position.tokens_owed_x }, U256::from(100));
            assert_eq!({ position.tokens_owed_y }, U256::from(100));
        }
        // fee should change
        {
            let mut position = Position {
                liquidity: U256::from(1 * 10u128.pow(Liquidity::scale() as u32)),
                fee_growth_inside_x: U128::from(4 * 10u128.pow(FeeGrowth::scale() as u32)),
                fee_growth_inside_y: U128::from(4 * 10u128.pow(FeeGrowth::scale() as u32)),
                tokens_owed_x: U256::from(100),
                tokens_owed_y: U256::from(100),
                ..Default::default()
            };
            let sign = true;
            let liquidity_delta = Liquidity::from_integer(1);
            let fee_growth_inside_x = FeeGrowth::from_integer(5);
            let fee_growth_inside_y = FeeGrowth::from_integer(5);

            position
                .update(
                    sign,
                    liquidity_delta,
                    fee_growth_inside_x,
                    fee_growth_inside_y,
                )
                .unwrap();

            assert_eq!(
                { position.liquidity },
                U256::from(2 * 10u128.pow(Liquidity::scale() as u32))
            );
            assert_eq!(
                { position.fee_growth_inside_x },
                U128::from(5 * 10u128.pow(FeeGrowth::scale() as u32))
            );
            assert_eq!(
                { position.fee_growth_inside_y },
                U128::from(5 * 10u128.pow(FeeGrowth::scale() as u32))
            );
            assert_eq!({ position.tokens_owed_x }, U256::from(101));
            assert_eq!({ position.tokens_owed_y }, U256::from(101));
        }
        // previous fee_growth_inside close to max and current close to 0
        {
            let mut position = Position {
                liquidity: U256::from(1 * 10u128.pow(Liquidity::scale() as u32)),
                fee_growth_inside_x: U128::from(u128::MAX)
                    - U128::from(10 * 10u128.pow(FeeGrowth::scale() as u32)),
                fee_growth_inside_y: U128::from(u128::MAX)
                    - U128::from(10 * 10u128.pow(FeeGrowth::scale() as u32)),
                tokens_owed_x: U256::from(100),
                tokens_owed_y: U256::from(100),
                ..Default::default()
            };
            let sign = true;
            let liquidity_delta = Liquidity::from_integer(1);
            let fee_growth_inside_x = FeeGrowth::from_integer(10);
            let fee_growth_inside_y = FeeGrowth::from_integer(10);

            position
                .update(
                    sign,
                    liquidity_delta,
                    fee_growth_inside_x,
                    fee_growth_inside_y,
                )
                .unwrap();

            assert_eq!(
                { position.liquidity },
                U256::from(2 * 10u128.pow(Liquidity::scale() as u32)),
            );
            assert_eq!(
                { position.fee_growth_inside_x },
                U128::from(10 * 10u128.pow(FeeGrowth::scale() as u32))
            );
            assert_eq!(
                { position.fee_growth_inside_y },
                U128::from(10 * 10u128.pow(FeeGrowth::scale() as u32))
            );
            assert_eq!({ position.tokens_owed_x }, U256::from(120));
            assert_eq!({ position.tokens_owed_y }, U256::from(120));
        }
    }

    // #[test]
    // fn test_modify() {
    //     // owed tokens after overflow
    //     {
    //         let mut position = Position {
    //             liquidity: Liquidity::from_integer(123),
    //             fee_growth_inside_x: FeeGrowth::new(u128::MAX) - FeeGrowth::from_integer(1234),
    //             fee_growth_inside_y: FeeGrowth::new(u128::MAX) - FeeGrowth::from_integer(1234),
    //             tokens_owed_x: TokenAmount(0),
    //             tokens_owed_y: TokenAmount(0),
    //             ..Default::default()
    //         };
    //         let mut pool = Pool {
    //             current_tick_index: 0,
    //             fee_growth_global_x: FeeGrowth::from_integer(20),
    //             fee_growth_global_y: FeeGrowth::from_integer(20),
    //             ..Default::default()
    //         };
    //         let mut upper_tick = Tick {
    //             index: -10,
    //             fee_growth_outside_x: FeeGrowth::from_integer(15),
    //             fee_growth_outside_y: FeeGrowth::from_integer(15),
    //             liquidity_gross: Liquidity::from_integer(123),
    //             ..Default::default()
    //         };
    //         let mut lower_tick = Tick {
    //             index: -20,
    //             fee_growth_outside_x: FeeGrowth::from_integer(20),
    //             fee_growth_outside_y: FeeGrowth::from_integer(20),
    //             liquidity_gross: Liquidity::from_integer(123),
    //             ..Default::default()
    //         };
    //         let liquidity_delta = Liquidity::new(0);
    //         let add = true;
    //         let current_timestamp: u64 = 1234567890;

    //         position
    //             .modify(
    //                 &mut pool,
    //                 &mut upper_tick,
    //                 &mut lower_tick,
    //                 liquidity_delta,
    //                 add,
    //                 current_timestamp,
    //                 1,
    //             )
    //             .unwrap();

    //         assert_eq!({ position.tokens_owed_x }, TokenAmount(151167));
    //     }
    // }
}
