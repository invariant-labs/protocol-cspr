use crate::{
    math::types::{liquidity::Liquidity, token_amount::TokenAmount},
    uints::{U128T, U256T, U384T},
};
use decimal::*;
use traceable_result::*;

#[decimal(28, U256T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct FeeGrowth {
    pub v: U128T,
}

impl FeeGrowth {
    pub fn unchecked_add(self, other: FeeGrowth) -> FeeGrowth {
        if other.get() > FeeGrowth::max_instance().get() - self.get() {
            FeeGrowth::new((other.get() - (FeeGrowth::max_instance().get() - self.get())) - 1)
        } else {
            FeeGrowth::new(self.get() + other.get())
        }
    }

    pub fn unchecked_sub(self, other: FeeGrowth) -> FeeGrowth {
        if other.get() > self.get() {
            FeeGrowth::new(FeeGrowth::max_instance().get() - (other.get() - self.get()) + 1)
        } else {
            FeeGrowth::new(self.get() - other.get())
        }
    }

    pub fn from_fee(liquidity: Liquidity, fee: TokenAmount) -> TrackableResult<Self> {
        Ok(Self::new(
            Self::checked_from_value(
                U384T::from(fee.cast::<U384T>())
                    .checked_mul(FeeGrowth::one().cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?
                    .checked_mul(Liquidity::one().cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?
                    .checked_div(liquidity.cast())
                    .ok_or_else(|| err!(TrackableError::DIV))?,
            )
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?,
        ))
    }

    pub fn to_fee(self, liquidity: Liquidity) -> TrackableResult<TokenAmount> {
        Ok(TokenAmount::new(
            TokenAmount::checked_from_value(
                U384T::from(self.cast::<U384T>())
                    .checked_mul(liquidity.cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?
                    .checked_div(Liquidity::one().cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?
                    .checked_div(FeeGrowth::one().cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?,
            )
            .map_err(|_| err!(TrackableError::cast::<TokenAmount>().as_str()))?,
        ))
    }
}

pub fn calculate_fee_growth_inside(
    tick_lower: i32,
    tick_lower_fee_growth_outside_x: FeeGrowth,
    tick_lower_fee_growth_outside_y: FeeGrowth,
    tick_upper: i32,
    tick_upper_fee_growth_outside_x: FeeGrowth,
    tick_upper_fee_growth_outside_y: FeeGrowth,
    tick_current: i32,
    fee_growth_global_x: FeeGrowth,
    fee_growth_global_y: FeeGrowth,
) -> (FeeGrowth, FeeGrowth) {
    // determine position relative to current tick
    let current_above_lower = tick_current >= tick_lower;
    let current_below_upper = tick_current < tick_upper;

    // calculate fee growth below
    let fee_growth_below_x = if current_above_lower {
        tick_lower_fee_growth_outside_x
    } else {
        fee_growth_global_x.unchecked_sub(tick_lower_fee_growth_outside_x)
    };
    let fee_growth_below_y = if current_above_lower {
        tick_lower_fee_growth_outside_y
    } else {
        fee_growth_global_y.unchecked_sub(tick_lower_fee_growth_outside_y)
    };

    // calculate fee growth above
    let fee_growth_above_x = if current_below_upper {
        tick_upper_fee_growth_outside_x
    } else {
        fee_growth_global_x.unchecked_sub(tick_upper_fee_growth_outside_x)
    };
    let fee_growth_above_y = if current_below_upper {
        tick_upper_fee_growth_outside_y
    } else {
        fee_growth_global_y.unchecked_sub(tick_upper_fee_growth_outside_y)
    };

    // calculate fee growth inside
    let fee_growth_inside_x = fee_growth_global_x
        .unchecked_sub(fee_growth_below_x)
        .unchecked_sub(fee_growth_above_x);
    let fee_growth_inside_y = fee_growth_global_y
        .unchecked_sub(fee_growth_below_y)
        .unchecked_sub(fee_growth_above_y);

    (fee_growth_inside_x, fee_growth_inside_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{consts::MAX_TICK, types::sqrt_price::SqrtPrice};

    #[test]
    fn test_unchecked_add() {
        let max = FeeGrowth::max_instance();
        let almost_max = FeeGrowth::max_instance() - FeeGrowth { v: U128T::from(1) };
        {
            let result = max.unchecked_add(almost_max);
            assert_eq!(
                result,
                FeeGrowth::new(U128T::from(340282366920938463463374607431768211453u128))
            )
        }
        {
            let addend = FeeGrowth::max_instance() - FeeGrowth { v: U128T::from(6) };
            let result = max.unchecked_add(addend);
            assert_eq!(
                result,
                FeeGrowth::new(U128T::from(340282366920938463463374607431768211448u128))
            )
        }
        {
            let addend = FeeGrowth::max_instance() - FeeGrowth { v: U128T::from(20) };
            let result = max.unchecked_add(addend);
            assert_eq!(
                result,
                FeeGrowth::new(U128T::from(340282366920938463463374607431768211434u128))
            )
        }
        {
            let addend = FeeGrowth::max_instance() - FeeGrowth { v: U128T::from(50) };
            let result = max.unchecked_add(addend);
            assert_eq!(
                result,
                FeeGrowth::new(U128T::from(340282366920938463463374607431768211404u128))
            )
        }
        {
            let num = FeeGrowth { v: U128T::from(11) };
            let addend = FeeGrowth {
                v: U128T::from(1100),
            };
            let result = num.unchecked_add(addend);
            assert_eq!(result, FeeGrowth::new(U128T::from(1111)));
        }
    }

    #[test]
    fn test_unchecked_sub() {
        let one = FeeGrowth::new(U128T::from(1));
        let ten = FeeGrowth::new(U128T::from(10));
        let twenty = FeeGrowth::new(U128T::from(20));
        let five = FeeGrowth::new(U128T::from(5));
        let max = FeeGrowth::max_instance();
        let almost_max = max - one;
        {
            let result = almost_max.unchecked_sub(max);
            assert_eq!(
                result,
                FeeGrowth::new(U128T::from(340282366920938463463374607431768211455u128))
            )
        }
        {
            let result = one.unchecked_sub(one);
            assert_eq!(result, FeeGrowth::new(U128T::from(0)))
        }
        {
            let result = ten.unchecked_sub(twenty);
            assert_eq!(
                result,
                FeeGrowth::new(U128T::from(340282366920938463463374607431768211446u128))
            )
        }
        {
            let result = ten.unchecked_sub(five);
            assert_eq!(result, five)
        }
    }

    #[test]
    fn test_from_fee() {
        // One
        {
            let fee_growth =
                FeeGrowth::from_fee(Liquidity::from_integer(1), TokenAmount::new(U256T::from(1)))
                    .unwrap();
            assert_eq!(fee_growth, FeeGrowth::from_integer(1));
        }
        // Half
        {
            let fee_growth =
                FeeGrowth::from_fee(Liquidity::from_integer(2), TokenAmount::new(U256T::from(1)))
                    .unwrap();
            assert_eq!(fee_growth, FeeGrowth::from_scale(5, 1))
        }
        // Little
        {
            let fee_growth = FeeGrowth::from_fee(
                Liquidity::from_integer(u64::MAX),
                TokenAmount::new(U256T::from(1)),
            )
            .unwrap();
            // real    5.42101086242752217003726400434970855712890625 × 10^-20
            // expected 542101086
            assert_eq!(fee_growth, FeeGrowth::new(U128T::from(542101086)))
        }
        // Fairly big
        {
            let fee_growth = FeeGrowth::from_fee(
                Liquidity::from_integer(100),
                TokenAmount::new(U256T::from(1_000_000)),
            )
            .unwrap();
            assert_eq!(fee_growth, FeeGrowth::from_integer(10000))
        }
    }

    #[test]
    fn test_domain_from_fee() {
        // max FeeGrowth case inside of domain
        {
            let max_tick_spacing = 100;
            let tick_search_range = 256; // TODO: replace it with const
            let p_u = SqrtPrice::from_tick(MAX_TICK).unwrap();
            let p_l =
                SqrtPrice::from_tick(MAX_TICK - max_tick_spacing * tick_search_range).unwrap();
            let max_p_delta = p_u - p_l;
            let max_l = Liquidity::max_instance();

            // token / L < delta_price
            // token < L * delta_price
            // token_max = L_max * delta_price_max
            let max_token: U256T = TokenAmount::from_value(
                U384T::from(max_l.cast::<U384T>()) * U384T::from(max_p_delta.cast::<U384T>())
                    / U384T::from(Liquidity::from_integer(1).cast::<U384T>())
                    / U384T::from(SqrtPrice::from_integer(1).cast::<U384T>()),
            );
            let fee_growth =
                FeeGrowth::from_fee(max_l, TokenAmount::new(U256T::from(max_token))).unwrap();

            assert_eq!(
                fee_growth,
                FeeGrowth::new(U128T::from(473129365723326089999999999999999u128))
            );
        }
        // min FeeGrowth case inside of domain
        {
            // token * fee_Percentages / L < min_delta_price

            // token * fee_Percentage / min_delta_price < L
            // L > token * fee_Percentage / min_delta_price

            // l = token  * fee_Percentage / min_delta_price
            //  min_token_amount, min_fee, max_possible_liquidity
            // basis point = 10^-4
            let basis_point = 10000;
            let min_token = TokenAmount::new(U256T::from(1));
            let max_l = min_token.get()
                * FeeGrowth::from_integer(1).cast::<U256T>()
                * Liquidity::from_integer(1).get()
                * U256T::from(basis_point);

            let fee_growth = FeeGrowth::from_fee(
                Liquidity::new(U256T::from(max_l)),
                TokenAmount::new(U256T::from(min_token.get() * basis_point)),
            )
            .unwrap();

            assert_eq!(fee_growth, FeeGrowth::new(U128T::from(1)));
        }
        // outside of domain trigger overflow due to result not fit into FeeGrowth
        {
            let liquidity = Liquidity::new(U256T::from(1));
            let fee = TokenAmount::max_instance();
            let (_, _, stack) = FeeGrowth::from_fee(liquidity, fee).unwrap_err().get();
            assert_eq!(stack.len(), 1);
        }
        // amount = 0
        {
            let liquidity = Liquidity::from_integer(1_000);
            let fee = TokenAmount::new(U256T::from(0));
            let fee_growth = FeeGrowth::from_fee(liquidity, fee).unwrap();
            assert_eq!(fee_growth, FeeGrowth::new(U128T::from(0)));
        }
        // L = 0
        {
            let liquidity = Liquidity::new(U256T::from(0));
            let fee = TokenAmount::from_integer(1_000);

            let (_format, cause, stack) = FeeGrowth::from_fee(liquidity, fee).unwrap_err().get();
            assert_eq!(cause, "division overflow or division by zero");
            assert_eq!(stack.len(), 1);
        }
    }

    #[test]
    fn test_to_fee() {
        // equal
        {
            let amount = TokenAmount::new(U256T::from(100));
            let liquidity = Liquidity::from_integer(1_000_000);

            let fee_growth = FeeGrowth::from_fee(liquidity, amount).unwrap();
            let out = fee_growth.to_fee(liquidity).unwrap();
            assert_eq!(out, amount);
        }
        // greater liquidity
        {
            let amount = TokenAmount::new(U256T::from(100));
            let liquidity_before = Liquidity::from_integer(1_000_000);
            let liquidity_after = Liquidity::from_integer(10_000_000);

            let fee_growth = FeeGrowth::from_fee(liquidity_before, amount).unwrap();
            let out = fee_growth.to_fee(liquidity_after).unwrap();
            assert_eq!(out, TokenAmount::from_integer(1000))
        }
        // huge liquidity
        {
            let amount = TokenAmount::new(U256T::from(100_000_000__000000u128));
            let liquidity = Liquidity::from_integer(2u128.pow(77));

            let fee_growth = FeeGrowth::from_fee(liquidity, amount).unwrap();
            // real    6.61744490042422139897126953655970282852649688720703125 × 10^-10
            // expected 6617444900424221398
            assert_eq!(
                fee_growth,
                FeeGrowth::new(U128T::from(6617444900424221398u128))
            );

            let out = fee_growth.to_fee(liquidity).unwrap();
            // real    9.99999999999999999853225897430980027744256 × 10^13
            // expected 99999999999999
            assert_eq!(out, TokenAmount::new(U256T::from(99_999_999__999999u128)))
        }
    }

    #[test]
    fn test_domain_to_fee() {
        // overflowing `big_mul`
        {
            let amount = TokenAmount::new(U256T::from(600000000000000000u128));
            let liquidity = Liquidity::from_integer(10000000000000000000u128);

            let fee_growth = FeeGrowth::from_fee(liquidity, amount).unwrap();
            // real     0.06
            // expected 0.06
            assert_eq!(
                fee_growth,
                FeeGrowth::new(U128T::from(600000000000000000000000000u128))
            );

            let out = fee_growth.to_fee(liquidity).unwrap();
            // real     600000000000000000
            // expected 600000000000000000
            assert_eq!(out, TokenAmount::from_integer(1) * amount)
        }
        // max value inside domain
        {
            let liquidity = Liquidity::max_instance();
            let fee_growth = FeeGrowth::from_integer(100000);

            let out = fee_growth.to_fee(liquidity).unwrap();
            assert_eq!(out, TokenAmount::max_instance())
        }
        // overflow
        {
            let liquidity = Liquidity::max_instance();
            let fee_growth = FeeGrowth::max_instance();

            let (_format, cause, stack) = fee_growth.to_fee(liquidity).unwrap_err().get();
            assert_eq!(
                cause,
                "conversion to invariant::math::types::token_amount::TokenAmount type failed"
            );
            assert_eq!(stack.len(), 1);
        }
        // FeeGrowth = 0
        {
            let liquidity = Liquidity::from_integer(1_000);
            let fee_growth = FeeGrowth::new(U128T::from(0));

            let result = fee_growth.to_fee(liquidity).unwrap();
            assert_eq!(result, TokenAmount::new(U256T::from(0)));
        }
        // L = 0
        {
            let liquidity = Liquidity::new(U256T::from(0));
            let fee_growth = FeeGrowth::from_integer(1_000);

            let result = fee_growth.to_fee(liquidity).unwrap();
            assert_eq!(result, TokenAmount::new(U256T::from(0)));
        }
    }

    #[test]
    fn test_calculate_fee_growth_inside() {
        // <──────────────                    ──────────────>
        // fee_outside_t0| fee_growth_inside |fee_outside_t1
        //<───────────── t0 ────── C ────── t1 ───────────────────>

        // fee_growth_inside = fee_growth_global - t0.fee_outside - t1.fee_outside

        let fee_growth_global_x = FeeGrowth::from_integer(15);
        let fee_growth_global_y = FeeGrowth::from_integer(15);

        let tick_lower_index = -2;
        let tick_lower_fee_growth_outside_x = FeeGrowth::new(U128T::from(0));
        let tick_lower_fee_growth_outside_y = FeeGrowth::new(U128T::from(0));

        let tick_upper_index = 2;
        let tick_upper_fee_growth_outside_x = FeeGrowth::from_integer(0);
        let tick_upper_fee_growth_outside_y = FeeGrowth::from_integer(0);

        // current tick inside range
        // lower    current     upper
        // |        |           |
        // -2       0           2
        {
            // index and fee global
            let tick_current = 0;
            let fee_growth_inside = calculate_fee_growth_inside(
                tick_lower_index,
                tick_lower_fee_growth_outside_x,
                tick_lower_fee_growth_outside_y,
                tick_upper_index,
                tick_upper_fee_growth_outside_x,
                tick_upper_fee_growth_outside_y,
                tick_current,
                fee_growth_global_x,
                fee_growth_global_y,
            );

            assert_eq!(fee_growth_inside.0, FeeGrowth::from_integer(15)); // x fee growth inside
            assert_eq!(fee_growth_inside.1, FeeGrowth::from_integer(15)); // y fee growth inside
        }
        //                      ───────fee_outside_t0──────────>
        //                     |fee_growth_inside| fee_outside_t1
        // ─────── c ─────── t0 ──────────────> t1 ───────────>
        //
        // fee_growth_inside = t0.fee_outside - t1.fee_outside
        //
        // current tick below range
        // current  lower       upper
        // |        |           |
        // -4       2           2
        {
            let tick_current = -4;
            let fee_growth_inside = calculate_fee_growth_inside(
                tick_lower_index,
                tick_lower_fee_growth_outside_x,
                tick_lower_fee_growth_outside_y,
                tick_upper_index,
                tick_upper_fee_growth_outside_x,
                tick_upper_fee_growth_outside_y,
                tick_current,
                fee_growth_global_x,
                fee_growth_global_y,
            );

            assert_eq!(fee_growth_inside.0, FeeGrowth::new(U128T::from(0))); // x fee growth inside
            assert_eq!(fee_growth_inside.1, FeeGrowth::new(U128T::from(0))); // y fee growth inside
        }

        // <──────────fee_outside_t0──────────
        // fee_outside_t1  | fee_growth_inside|
        // ────────────── t1 ──────────────── t0 ─────── c ───────────>

        // fee_growth_inside = t0.fee_outside - t1.fee_outside

        // current tick upper range
        // lower    upper       current
        // |        |           |
        // -2       2           4
        {
            let tick_current = 4;
            let fee_growth_inside = calculate_fee_growth_inside(
                tick_lower_index,
                tick_lower_fee_growth_outside_x,
                tick_lower_fee_growth_outside_y,
                tick_upper_index,
                tick_upper_fee_growth_outside_x,
                tick_upper_fee_growth_outside_y,
                tick_current,
                fee_growth_global_x,
                fee_growth_global_y,
            );

            assert_eq!(fee_growth_inside.0, FeeGrowth::new(U128T::from(0))); // x fee growth inside
            assert_eq!(fee_growth_inside.1, FeeGrowth::new(U128T::from(0))); // y fee growth inside
        }

        // current tick upper range
        // lower    upper       current
        // |        |           |
        // -2       2           3
        {
            let tick_lower_index = -2;
            let tick_lower_fee_growth_outside_x = FeeGrowth::new(U128T::from(0));
            let tick_lower_fee_growth_outside_y = FeeGrowth::new(U128T::from(0));

            let tick_upper_index = 2;
            let tick_upper_fee_growth_outside_x = FeeGrowth::new(U128T::from(1));
            let tick_upper_fee_growth_outside_y = FeeGrowth::new(U128T::from(2));

            let fee_growth_global_x = FeeGrowth::from_integer(5);
            let fee_growth_global_y = FeeGrowth::from_integer(5);

            let tick_current = 3;
            let fee_growth_inside = calculate_fee_growth_inside(
                tick_lower_index,
                tick_lower_fee_growth_outside_x,
                tick_lower_fee_growth_outside_y,
                tick_upper_index,
                tick_upper_fee_growth_outside_x,
                tick_upper_fee_growth_outside_y,
                tick_current,
                fee_growth_global_x,
                fee_growth_global_y,
            );

            assert_eq!(fee_growth_inside.0, FeeGrowth::new(U128T::from(1))); // x fee growth inside
            assert_eq!(fee_growth_inside.1, FeeGrowth::new(U128T::from(2))); // y fee growth inside
        }

        // subtracts upper tick if below
        let tick_upper_index = 2;
        let tick_upper_fee_growth_outside_x = FeeGrowth::from_integer(2);
        let tick_upper_fee_growth_outside_y = FeeGrowth::from_integer(3);
        // current tick inside range
        // lower    current     upper
        // |        |           |
        // -2       0           2
        {
            let tick_current = 0;
            let fee_growth_inside = calculate_fee_growth_inside(
                tick_lower_index,
                tick_lower_fee_growth_outside_x,
                tick_lower_fee_growth_outside_y,
                tick_upper_index,
                tick_upper_fee_growth_outside_x,
                tick_upper_fee_growth_outside_y,
                tick_current,
                fee_growth_global_x,
                fee_growth_global_y,
            );

            assert_eq!(fee_growth_inside.0, FeeGrowth::from_integer(13)); // x fee growth inside
            assert_eq!(fee_growth_inside.1, FeeGrowth::from_integer(12)); // y fee growth inside
        }

        // subtracts lower tick if above
        let tick_upper_index = 2;
        let tick_upper_fee_growth_outside_x = FeeGrowth::new(U128T::from(0));
        let tick_upper_fee_growth_outside_y = FeeGrowth::new(U128T::from(0));

        let tick_lower_index = -2;
        let tick_lower_fee_growth_outside_x = FeeGrowth::from_integer(2);
        let tick_lower_fee_growth_outside_y = FeeGrowth::from_integer(3);

        // current tick inside range
        // lower    current     upper
        // |        |           |
        // -2       0           2
        {
            let tick_current = 0;
            let fee_growth_inside = calculate_fee_growth_inside(
                tick_lower_index,
                tick_lower_fee_growth_outside_x,
                tick_lower_fee_growth_outside_y,
                tick_upper_index,
                tick_upper_fee_growth_outside_x,
                tick_upper_fee_growth_outside_y,
                tick_current,
                fee_growth_global_x,
                fee_growth_global_y,
            );

            assert_eq!(fee_growth_inside.0, FeeGrowth::from_integer(13)); // x fee growth inside
            assert_eq!(fee_growth_inside.1, FeeGrowth::from_integer(12)); // y fee growth inside
        }
    }

    #[test]
    fn test_domain_calculate_fee_growth_inside() {
        let tick_current = 0;
        let fee_growth_global_x = FeeGrowth::from_integer(20);
        let fee_growth_global_y = FeeGrowth::from_integer(20);

        let tick_lower_index = -20;
        let tick_lower_fee_growth_outside_x = FeeGrowth::from_integer(20);
        let tick_lower_fee_growth_outside_y = FeeGrowth::from_integer(20);

        let tick_upper_index = -10;
        let tick_upper_fee_growth_outside_x = FeeGrowth::from_integer(15);
        let tick_upper_fee_growth_outside_y = FeeGrowth::from_integer(15);

        let fee_growth_inside = calculate_fee_growth_inside(
            tick_lower_index,
            tick_lower_fee_growth_outside_x,
            tick_lower_fee_growth_outside_y,
            tick_upper_index,
            tick_upper_fee_growth_outside_x,
            tick_upper_fee_growth_outside_y,
            tick_current,
            fee_growth_global_x,
            fee_growth_global_y,
        );

        assert_eq!(
            fee_growth_inside.0,
            FeeGrowth::max_instance() - FeeGrowth::from_integer(5) + FeeGrowth::new(U128T::from(1))
        );
        assert_eq!(
            fee_growth_inside.1,
            FeeGrowth::max_instance() - FeeGrowth::from_integer(5) + FeeGrowth::new(U128T::from(1))
        );
    }
}
