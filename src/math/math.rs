use decimal::*;
use traceable_result::*;

use crate::math::consts::*;
use crate::math::types::{liquidity::*, percentage::*, sqrt_price::*, token_amount::*};
use crate::uints::{U256T, U384T, U448T};

#[derive(PartialEq, Debug)]
pub struct SwapResult {
    pub next_sqrt_price: SqrtPrice,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee_amount: TokenAmount,
}

pub fn compute_swap_step(
    current_sqrt_price: SqrtPrice,
    target_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    by_amount_in: bool,
    fee: Percentage,
) -> TrackableResult<SwapResult> {
    if liquidity.is_zero() {
        return Ok(SwapResult {
            next_sqrt_price: target_sqrt_price,
            amount_in: TokenAmount::new(U256T::from(0)),
            amount_out: TokenAmount::new(U256T::from(0)),
            fee_amount: TokenAmount::new(U256T::from(0)),
        });
    }

    let x_to_y = current_sqrt_price >= target_sqrt_price;
    let next_sqrt_price: SqrtPrice;
    let (mut amount_in, mut amount_out) = (
        TokenAmount::new(U256T::from(0)),
        TokenAmount::new(U256T::from(0)),
    );

    if by_amount_in {
        let amount_after_fee = amount.big_mul(Percentage::from_integer(1u8) - fee);

        amount_in = ok_or_mark_trace!(if x_to_y {
            get_delta_x(target_sqrt_price, current_sqrt_price, liquidity, true)
        } else {
            get_delta_y(current_sqrt_price, target_sqrt_price, liquidity, true)
        })?;
        // if target sqrt_price was hit it will be the next sqrt_price
        if amount_after_fee >= amount_in {
            next_sqrt_price = target_sqrt_price
        } else {
            next_sqrt_price = ok_or_mark_trace!(get_next_sqrt_price_from_input(
                current_sqrt_price,
                liquidity,
                amount_after_fee,
                x_to_y,
            ))?;
        };
    } else {
        amount_out = ok_or_mark_trace!(if x_to_y {
            get_delta_y(target_sqrt_price, current_sqrt_price, liquidity, false)
        } else {
            get_delta_x(current_sqrt_price, target_sqrt_price, liquidity, false)
        })?;

        if amount >= amount_out {
            next_sqrt_price = target_sqrt_price
        } else {
            next_sqrt_price = ok_or_mark_trace!(get_next_sqrt_price_from_output(
                current_sqrt_price,
                liquidity,
                amount,
                x_to_y
            ))?;
        }
    }

    let not_max = target_sqrt_price != next_sqrt_price;

    if x_to_y {
        if not_max || !by_amount_in {
            amount_in = ok_or_mark_trace!(get_delta_x(
                next_sqrt_price,
                current_sqrt_price,
                liquidity,
                true
            ))?
        };
        if not_max || by_amount_in {
            amount_out = ok_or_mark_trace!(get_delta_y(
                next_sqrt_price,
                current_sqrt_price,
                liquidity,
                false
            ))?
        }
    } else {
        if not_max || !by_amount_in {
            amount_in = ok_or_mark_trace!(get_delta_y(
                current_sqrt_price,
                next_sqrt_price,
                liquidity,
                true
            ))?
        };
        if not_max || by_amount_in {
            amount_out = ok_or_mark_trace!(get_delta_x(
                current_sqrt_price,
                next_sqrt_price,
                liquidity,
                false
            ))?
        };
    }

    // Amount out can not exceed amount
    if !by_amount_in && amount_out > amount {
        amount_out = amount;
    }

    let fee_amount = if by_amount_in && next_sqrt_price != target_sqrt_price {
        amount - amount_in
    } else {
        amount_in.big_mul_up(fee)
    };

    Ok(SwapResult {
        next_sqrt_price,
        amount_in,
        amount_out,
        fee_amount,
    })
}

pub fn get_delta_x(
    sqrt_price_a: SqrtPrice,
    sqrt_price_b: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TrackableResult<TokenAmount> {
    let delta_price: SqrtPrice = if sqrt_price_a > sqrt_price_b {
        sqrt_price_a - sqrt_price_b
    } else {
        sqrt_price_b - sqrt_price_a
    };
    let nominator = delta_price.big_mul_to_value(liquidity);

    ok_or_mark_trace!(match rounding_up {
        true => SqrtPrice::big_div_values_to_token_up(
            nominator,
            sqrt_price_a.big_mul_to_value(sqrt_price_b),
        ),
        false => SqrtPrice::big_div_values_to_token(
            nominator,
            sqrt_price_a.big_mul_to_value_up(sqrt_price_b),
        ),
    })
}

pub fn get_delta_y(
    sqrt_price_a: SqrtPrice,
    sqrt_price_b: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TrackableResult<TokenAmount> {
    let delta: SqrtPrice = if sqrt_price_a > sqrt_price_b {
        sqrt_price_a - sqrt_price_b
    } else {
        sqrt_price_b - sqrt_price_a
    };

    let delta_y = match rounding_up {
        true => delta
            .big_mul_to_value_up(liquidity)
            .checked_add(SqrtPrice::almost_one().cast())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::DIV))?,
        false => delta
            .big_mul_to_value(liquidity)
            .checked_div(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::DIV))?,
    };

    Ok(TokenAmount::new(
        TokenAmount::checked_from_value(delta_y)
            .map_err(|_| err!(TrackableError::cast::<TokenAmount>().as_str()))?,
    ))
}

fn get_next_sqrt_price_from_input(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    x_to_y: bool,
) -> TrackableResult<SqrtPrice> {
    let result = if x_to_y {
        // add x to pool, decrease sqrt_price
        get_next_sqrt_price_x_up(starting_sqrt_price, liquidity, amount, true)
    } else {
        // add y to pool, increase sqrt_price
        get_next_sqrt_price_y_down(starting_sqrt_price, liquidity, amount, true)
    };
    ok_or_mark_trace!(result)
}

fn get_next_sqrt_price_from_output(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    x_to_y: bool,
) -> TrackableResult<SqrtPrice> {
    let result = if x_to_y {
        // remove y from pool, decrease sqrt_price
        get_next_sqrt_price_y_down(starting_sqrt_price, liquidity, amount, false)
    } else {
        // remove x from pool, increase sqrt_price
        get_next_sqrt_price_x_up(starting_sqrt_price, liquidity, amount, false)
    };
    ok_or_mark_trace!(result)
}

pub fn get_next_sqrt_price_x_up(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    x: TokenAmount,
    add_x: bool,
) -> TrackableResult<SqrtPrice> {
    if x.is_zero() {
        return Ok(starting_sqrt_price);
    };
    let price_delta = ok_or_mark_trace!(SqrtPrice::checked_from_decimal_to_value(liquidity)
        .map_err(|_| err!("extending liquidity overflow")))?;

    let denominator = TokenAmount::from_value(ok_or_mark_trace!(match add_x {
        true => price_delta.checked_add(starting_sqrt_price.big_mul_to_value(x)),
        false => price_delta.checked_sub(starting_sqrt_price.big_mul_to_value(x)),
    }
    .ok_or_else(|| err!("big_liquidity -/+ sqrt_price * x")))?); // never should be triggered

    ok_or_mark_trace!(SqrtPrice::checked_big_div_values_up(
        TokenAmount::from_value(starting_sqrt_price.big_mul_to_value_up(liquidity)),
        denominator
    ))
}

fn get_next_sqrt_price_y_down(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    y: TokenAmount,
    add_y: bool,
) -> TrackableResult<SqrtPrice> {
    let numerator: U448T = SqrtPrice::from_value::<U448T, U384T>(
        (SqrtPrice::checked_from_decimal_to_value(y))
            .map_err(|_| err!("extending amount overflow"))?,
    );

    let denominator: U448T = SqrtPrice::from_value::<U448T, U384T>(
        SqrtPrice::checked_from_decimal_to_value(liquidity)
            .map_err(|_| err!("extending liquidity overflow"))?,
    );

    if add_y {
        let quotient =
            ok_or_mark_trace!(SqrtPrice::checked_big_div_values(numerator, denominator))?;
        from_result!(starting_sqrt_price.checked_add(quotient))
    } else {
        let quotient =
            ok_or_mark_trace!(SqrtPrice::checked_big_div_values_up(numerator, denominator))?;
        from_result!(starting_sqrt_price.checked_sub(quotient))
    }
}

pub fn calculate_amount_delta(
    current_tick_index: i32,
    current_sqrt_price: SqrtPrice,
    liquidity_delta: Liquidity,
    liquidity_sign: bool,
    upper_tick: i32,
    lower_tick: i32,
) -> TrackableResult<(TokenAmount, TokenAmount, bool)> {
    if upper_tick < lower_tick {
        return Err(err!("upper_tick is not greater than lower_tick"));
    }
    let mut amount_x = TokenAmount::new(U256T::from(0));
    let mut amount_y = TokenAmount::new(U256T::from(0));
    let mut update_liquidity = false;

    if current_tick_index < lower_tick {
        amount_x = ok_or_mark_trace!(get_delta_x(
            ok_or_mark_trace!(SqrtPrice::from_tick(lower_tick))?,
            ok_or_mark_trace!(SqrtPrice::from_tick(upper_tick))?,
            liquidity_delta,
            liquidity_sign,
        ))?;
    } else if current_tick_index < upper_tick {
        amount_x = ok_or_mark_trace!(get_delta_x(
            current_sqrt_price,
            ok_or_mark_trace!(SqrtPrice::from_tick(upper_tick))?,
            liquidity_delta,
            liquidity_sign,
        ))?;
        amount_y = ok_or_mark_trace!(get_delta_y(
            ok_or_mark_trace!(SqrtPrice::from_tick(lower_tick))?,
            current_sqrt_price,
            liquidity_delta,
            liquidity_sign,
        ))?;
        update_liquidity = true;
    } else {
        amount_y = ok_or_mark_trace!(get_delta_y(
            ok_or_mark_trace!(SqrtPrice::from_tick(lower_tick))?,
            ok_or_mark_trace!(SqrtPrice::from_tick(upper_tick))?,
            liquidity_delta,
            liquidity_sign,
        ))?;
    }

    Ok((amount_x, amount_y, update_liquidity))
}

pub fn is_enough_amount_to_change_price(
    amount: TokenAmount,
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    fee: Percentage,
    by_amount_in: bool,
    x_to_y: bool,
) -> TrackableResult<bool> {
    if liquidity.is_zero() {
        return Ok(true);
    }

    let next_sqrt_price = ok_or_mark_trace!(if by_amount_in {
        let amount_after_fee = amount.big_mul(Percentage::from_integer(1) - fee);
        get_next_sqrt_price_from_input(starting_sqrt_price, liquidity, amount_after_fee, x_to_y)
    } else {
        get_next_sqrt_price_from_output(starting_sqrt_price, liquidity, amount, x_to_y)
    })?;

    Ok(starting_sqrt_price.ne(&next_sqrt_price))
}

pub fn calculate_max_liquidity_per_tick(tick_spacing: u16) -> Liquidity {
    const MAX_TICKS_AMOUNT_SQRT_PRICE_LIMITED: u128 = 2 * MAX_TICK as u128 + 1;
    let ticks_amount_spacing_limited = MAX_TICKS_AMOUNT_SQRT_PRICE_LIMITED / tick_spacing as u128;
    Liquidity::new(Liquidity::max_instance().get() / ticks_amount_spacing_limited)
}

pub fn check_ticks(tick_lower: i32, tick_upper: i32, tick_spacing: u16) -> TrackableResult<()> {
    if tick_lower > tick_upper {
        return Err(err!("tick_lower > tick_upper"));
    }
    ok_or_mark_trace!(check_tick(tick_lower, tick_spacing))?;
    ok_or_mark_trace!(check_tick(tick_upper, tick_spacing))?;

    Ok(())
}

pub fn check_tick(tick_index: i32, tick_spacing: u16) -> TrackableResult<()> {
    let (min_tick, max_tick) = (get_min_tick(tick_spacing), get_max_tick(tick_spacing));
    let tick_spacing = tick_spacing as i32;
    if tick_index % tick_spacing != 0 {
        return Err(err!("InvalidTickSpacing"));
    }
    if tick_index > max_tick || tick_index < min_tick {
        return Err(err!("InvalidTickIndex"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::uints::U128T;

    #[test]
    fn test_domain_get_next_sqrt_price_from_input() {
        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(U256T::from(1));
        let max_amount = TokenAmount::max_instance();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let almost_min_sqrt_price = min_sqrt_price + SqrtPrice::new(U128T::from(1));
        let almost_max_sqrt_price = max_sqrt_price - SqrtPrice::new(U128T::from(1));

        // max result, increase sqrt_price case
        {
            // get_next_sqrt_price_y_down
            let result = get_next_sqrt_price_from_input(
                almost_max_sqrt_price,
                max_liquidity,
                TokenAmount::new(U256T::from(u128::MAX) * U256T::from(10u128.pow(10))),
                false,
            )
            .unwrap();

            assert_eq!(
                result,
                SqrtPrice::new(U128T::from(65535383934512647000000000001u128))
            );
        }
        // min result, decrease sqrt_price case
        {
            // get_next_sqrt_price_x_up
            let result = get_next_sqrt_price_from_input(
                almost_min_sqrt_price,
                max_liquidity,
                TokenAmount::new(U256T::from(u128::MAX) * U256T::from(10u128.pow(20))),
                true,
            )
            .unwrap();

            assert_eq!(
                result,
                SqrtPrice::new(U128T::from(15258931999999999995u128))
            );
        }
        // amount == 0
        {
            let result = get_next_sqrt_price_from_input(
                min_sqrt_price,
                max_liquidity,
                TokenAmount::new(U256T::from(0)),
                true,
            )
            .unwrap();

            assert_eq!(result, min_sqrt_price);
        }
        // liquidity == 0
        {
            let result = get_next_sqrt_price_from_input(
                min_sqrt_price,
                Liquidity::new(U256T::from(0)),
                TokenAmount::new(U256T::from(20)),
                true,
            )
            .unwrap();

            assert_eq!(result, SqrtPrice::new(U128T::from(0)));
        }
        // error handling
        {
            let (_, cause, stack) =
                get_next_sqrt_price_from_input(max_sqrt_price, min_liquidity, max_amount, false)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "Can't parse from U448T to U128T");
            assert_eq!(stack.len(), 3);
        }
    }

    #[test]
    fn test_domain_get_next_sqrt_price_from_output() {
        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(U256T::from(1));
        let max_amount = TokenAmount::max_instance();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let almost_min_sqrt_price = min_sqrt_price + SqrtPrice::new(U128T::from(1));
        let almost_max_sqrt_price = max_sqrt_price - SqrtPrice::new(U128T::from(1));

        // max result, increase sqrt_price case
        {
            // get_next_sqrt_price_x_up
            let result = get_next_sqrt_price_from_output(
                almost_max_sqrt_price, // 65535383934512646999999999999
                max_liquidity,
                TokenAmount::new(U256T::from(1)),
                false,
            )
            .unwrap();

            assert_eq!(
                result,
                SqrtPrice::new(U128T::from(65535383934512647000000000000u128))
            );
        }
        // min result, decrease sqrt_price case
        {
            // get_next_sqrt_price_y_down
            let result = get_next_sqrt_price_from_output(
                almost_min_sqrt_price,
                max_liquidity,
                TokenAmount::new(U256T::from(1)),
                true,
            )
            .unwrap();

            assert_eq!(
                result,
                SqrtPrice::new(U128T::from(15258932000000000000u128))
            );
        }
        // amount == 0
        {
            let result = get_next_sqrt_price_from_output(
                min_sqrt_price,
                max_liquidity,
                TokenAmount::new(U256T::from(0)),
                true,
            )
            .unwrap();

            assert_eq!(result, min_sqrt_price);
        }
        // liquidity == 0
        {
            let (_, cause, stack) = get_next_sqrt_price_from_output(
                min_sqrt_price,
                Liquidity::new(U256T::from(0)),
                TokenAmount::new(U256T::from(20)),
                true,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "subtraction underflow");
            assert_eq!(stack.len(), 3);
        }
        // error handling
        {
            let (_, cause, stack) =
                get_next_sqrt_price_from_output(max_sqrt_price, min_liquidity, max_amount, false)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 3);
        }
    }

    #[test]
    fn test_compute_swap_step() {
        // VALIDATE BASE SAMPLES
        // one token by amount in
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let target = SqrtPrice::new(U128T::from(1004987562112089027021926u128));
            let liquidity = Liquidity::from_integer(2000);
            let amount = TokenAmount::new(U256T::from(1));
            let fee = Percentage::from_scale(6, 4);

            let result =
                compute_swap_step(sqrt_price, target, liquidity, amount, true, fee).unwrap();

            let expected_result = SwapResult {
                next_sqrt_price: sqrt_price,
                amount_in: TokenAmount::new(U256T::from(0)),
                amount_out: TokenAmount::new(U256T::from(0)),
                fee_amount: TokenAmount::new(U256T::from(1)),
            };
            assert_eq!(result, expected_result)
        }
        // amount out capped at target sqrt_price
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let target = SqrtPrice::new(U128T::from(1004987562112089027021926u128));
            let liquidity = Liquidity::from_integer(2000);
            let amount = TokenAmount::new(U256T::from(20));
            let fee = Percentage::from_scale(6, 4);

            let result_in =
                compute_swap_step(sqrt_price, target, liquidity, amount, true, fee).unwrap();
            let result_out =
                compute_swap_step(sqrt_price, target, liquidity, amount, false, fee).unwrap();

            let expected_result = SwapResult {
                next_sqrt_price: target,
                amount_in: TokenAmount::new(U256T::from(10)),
                amount_out: TokenAmount::new(U256T::from(9)),
                fee_amount: TokenAmount::new(U256T::from(1)),
            };
            assert_eq!(result_in, expected_result);
            assert_eq!(result_out, expected_result);
        }
        // amount in not capped
        {
            let sqrt_price = SqrtPrice::from_scale(101, 2);
            let target = SqrtPrice::from_integer(10);
            let liquidity = Liquidity::from_integer(300000000);
            let amount = TokenAmount::new(U256T::from(1000000));
            let fee = Percentage::from_scale(6, 4);

            let result =
                compute_swap_step(sqrt_price, target, liquidity, amount, true, fee).unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::new(U128T::from(1013331333333_333333333333u128)),
                amount_in: TokenAmount::new(U256T::from(999400)),
                amount_out: TokenAmount::new(U256T::from(976487)), // ((1.013331333333 - 1.01) * 300000000) / (1.013331333333 * 1.01)
                fee_amount: TokenAmount::new(U256T::from(600)),
            };
            assert_eq!(result, expected_result)
        }
        // amount out not capped
        {
            let sqrt_price = SqrtPrice::from_integer(101);
            let target = SqrtPrice::from_integer(100);
            let liquidity = Liquidity::from_integer(5000000000000u128);
            let amount = TokenAmount::new(U256T::from(2000000));
            let fee = Percentage::from_scale(6, 4);

            let result =
                compute_swap_step(sqrt_price, target, liquidity, amount, false, fee).unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::new(U128T::from(100999999600000_000000000000u128)),
                amount_in: TokenAmount::new(U256T::from(197)), // (5000000000000 * (101 - 100.9999996)) /  (101 * 100.9999996)
                amount_out: amount,
                fee_amount: TokenAmount::new(U256T::from(1)),
            };
            assert_eq!(result, expected_result)
        }
        // empty swap step when sqrt_price is at tick
        {
            let current_sqrt_price = SqrtPrice::new(U128T::from(999500149965_000000000000u128));
            let target_sqrt_price = SqrtPrice::new(U128T::from(999500149965_000000000000u128));

            let liquidity = Liquidity::new(U256T::from(20006000_000000u128));
            let amount = TokenAmount::new(U256T::from(1_000_000));
            let by_amount_in = true;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: current_sqrt_price,
                amount_in: TokenAmount::new(U256T::from(0)),
                amount_out: TokenAmount::new(U256T::from(0)),
                fee_amount: TokenAmount::new(U256T::from(0)),
            };
            assert_eq!(result, expected_result)
        }
        // if liquidity is high, small amount in should not push sqrt_price
        {
            let current_sqrt_price = SqrtPrice::from_scale(999500149965u128, 12);
            let target_sqrt_price = SqrtPrice::from_scale(1999500149965u128, 12);
            let liquidity = Liquidity::from_integer(100_000000000000_000000000000u128);
            let amount = TokenAmount::new(U256T::from(10));
            let by_amount_in = true;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: current_sqrt_price,
                amount_in: TokenAmount::new(U256T::from(0)),
                amount_out: TokenAmount::new(U256T::from(0)),
                fee_amount: TokenAmount::new(U256T::from(10)),
            };
            assert_eq!(result, expected_result)
        }
        // amount_in > u64 for swap to target sqrt_price and when liquidity > 2^64
        {
            let current_sqrt_price = SqrtPrice::from_integer(1);
            let target_sqrt_price = SqrtPrice::from_scale(100005, 5); // 1.00005
            let liquidity = Liquidity::from_integer(368944000000_000000000000u128);
            let amount = TokenAmount::new(U256T::from(1));
            let by_amount_in = true;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: current_sqrt_price,
                amount_in: TokenAmount::new(U256T::from(0)),
                amount_out: TokenAmount::new(U256T::from(0)),
                fee_amount: TokenAmount::new(U256T::from(1)),
            };
            assert_eq!(result, expected_result)
        }
        // amount_out > u64 for swap to target sqrt_price and when liquidity > 2^64
        {
            let current_sqrt_price = SqrtPrice::from_integer(1);
            let target_sqrt_price = SqrtPrice::from_scale(100005, 5); // 1.00005
            let liquidity = Liquidity::from_integer(368944000000_000000000000u128);
            let amount = TokenAmount::new(U256T::from(1));
            let by_amount_in = false;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::new(U128T::from(1_000000000000_000000000003u128)),
                amount_in: TokenAmount::new(U256T::from(2)),
                amount_out: TokenAmount::new(U256T::from(1)),
                fee_amount: TokenAmount::new(U256T::from(1)),
            };
            assert_eq!(result, expected_result)
        }
        // liquidity is zero and by amount_in should skip to target sqrt_price
        {
            let current_sqrt_price = SqrtPrice::from_integer(1);
            let target_sqrt_price = SqrtPrice::from_scale(100005, 5); // 1.00005
            let liquidity = Liquidity::new(U256T::from(0));
            let amount = TokenAmount::new(U256T::from(100000));
            let by_amount_in = true;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: target_sqrt_price,
                amount_in: TokenAmount::new(U256T::from(0)),
                amount_out: TokenAmount::new(U256T::from(0)),
                fee_amount: TokenAmount::new(U256T::from(0)),
            };
            assert_eq!(result, expected_result)
        }
        // liquidity is zero and by amount_out should skip to target sqrt_price
        {
            let current_sqrt_price = SqrtPrice::from_integer(1);
            let target_sqrt_price = SqrtPrice::from_scale(100005, 5); // 1.00005
            let liquidity = Liquidity::new(U256T::from(0));
            let amount = TokenAmount::new(U256T::from(100000));
            let by_amount_in = false;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: target_sqrt_price,
                amount_in: TokenAmount::new(U256T::from(0)),
                amount_out: TokenAmount::new(U256T::from(0)),
                fee_amount: TokenAmount::new(U256T::from(0)),
            };
            assert_eq!(result, expected_result)
        }
        // normal swap step but fee is set to 0
        {
            let current_sqrt_price = SqrtPrice::from_scale(99995, 5); // 0.99995
            let target_sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(50000000);
            let amount = TokenAmount::new(U256T::from(1000));
            let by_amount_in = true;
            let fee = Percentage::new(U128T::from(0));

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::from_scale(99997, 5),
                amount_in: TokenAmount::new(U256T::from(1000)),
                amount_out: TokenAmount::new(U256T::from(1000)),
                fee_amount: TokenAmount::new(U256T::from(0)),
            };
            assert_eq!(result, expected_result)
        }
        // by_amount_out and x_to_y edge cases
        {
            let target_sqrt_price = SqrtPrice::from_tick(-10).unwrap();
            let current_sqrt_price = target_sqrt_price + SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(340282366920938463463374607u128);
            let one_token = TokenAmount::new(U256T::from(1));
            let tokens_with_same_output = TokenAmount::new(U256T::from(85));
            let zero_token = TokenAmount::new(U256T::from(0));
            let by_amount_in = false;
            let max_fee = Percentage::from_scale(9, 1);
            let min_fee = Percentage::from_integer(0);

            let one_token_result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                one_token,
                by_amount_in,
                max_fee,
            )
            .unwrap();
            let tokens_with_same_output_result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                tokens_with_same_output,
                by_amount_in,
                max_fee,
            )
            .unwrap();
            let zero_token_result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                zero_token,
                by_amount_in,
                min_fee,
            )
            .unwrap();
            /*
                86x -> [1, 85]y
                rounding due to sqrt_price accuracy
                it does not matter if you want 1 or 85 y tokens, will take you the same input amount
            */
            let expected_one_token_result = SwapResult {
                next_sqrt_price: current_sqrt_price - SqrtPrice::new(U128T::from(1)),
                amount_in: TokenAmount::new(U256T::from(86)),
                amount_out: TokenAmount::new(U256T::from(1)),
                fee_amount: TokenAmount::new(U256T::from(78)),
            };
            let expected_tokens_with_same_output_result = SwapResult {
                next_sqrt_price: current_sqrt_price - SqrtPrice::new(U128T::from(1)),
                amount_in: TokenAmount::new(U256T::from(86)),
                amount_out: TokenAmount::new(U256T::from(85)),
                fee_amount: TokenAmount::new(U256T::from(78)),
            };
            let expected_zero_token_result = SwapResult {
                next_sqrt_price: current_sqrt_price,
                amount_in: TokenAmount::new(U256T::from(0)),
                amount_out: TokenAmount::new(U256T::from(0)),
                fee_amount: TokenAmount::new(U256T::from(0)),
            };
            assert_eq!(one_token_result, expected_one_token_result);
            assert_eq!(
                tokens_with_same_output_result,
                expected_tokens_with_same_output_result
            );
            assert_eq!(zero_token_result, expected_zero_token_result);
        }
    }

    #[test]
    fn test_domain_compute_swap_step() {
        let one_sqrt_price = SqrtPrice::from_integer(1);
        let two_sqrt_price = SqrtPrice::from_integer(2);
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let one_liquidity = Liquidity::from_integer(1);
        let max_liquidity = Liquidity::max_instance();
        let max_amount = TokenAmount::max_instance();
        let max_amount_not_reached_target_sqrt_price = TokenAmount::new(U256T::MAX - 1);
        let max_fee = Percentage::from_integer(1);
        let min_fee = Percentage::new(U128T::from(0));

        // 100% fee | max_amount
        {
            let result = compute_swap_step(
                one_sqrt_price,
                two_sqrt_price,
                one_liquidity,
                max_amount,
                true,
                max_fee,
            )
            .unwrap();
            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::from_integer(1),
                    amount_in: TokenAmount::new(U256T::from(0)),
                    amount_out: TokenAmount::new(U256T::from(0)),
                    fee_amount: max_amount,
                }
            )
        }
        // 0% fee | max_amount | max_liquidity | sqrt_price slice
        {
            let result = compute_swap_step(
                one_sqrt_price,
                two_sqrt_price,
                max_liquidity,
                max_amount,
                true,
                min_fee,
            )
            .unwrap();

            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::new(U128T::from(2000000000000000000000000u128)),
                    amount_in: TokenAmount::new(U256T::from_dec_str("1157920892373161954235709850086879078532699846656405640394575840079131297").unwrap()),
                    amount_out: TokenAmount::new(U256T::from_dec_str(
                        "578960446186580977117854925043439539266349923328202820197287920039565648"
                    ).unwrap()),
                    fee_amount: TokenAmount::new(U256T::from(0)),
                }
            )
        }
        // by_amount_in == true || close to target_sqrt_price but not reached
        {
            let big_liquidity = Liquidity::from_integer(100_000_000_000_000u128);
            let amount_pushing_sqrt_price_to_target =
                TokenAmount::new(U256T::from(100000000000000u128));

            let result = compute_swap_step(
                one_sqrt_price,
                two_sqrt_price,
                big_liquidity,
                amount_pushing_sqrt_price_to_target - TokenAmount::new(U256T::from(1)),
                true,
                min_fee,
            )
            .unwrap();
            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::new(U128T::from(1999999999999990000000000u128)),
                    amount_in: TokenAmount::new(U256T::from(99999999999999u128)),
                    amount_out: TokenAmount::new(U256T::from(49999999999999u128)),
                    fee_amount: TokenAmount::new(U256T::from(0)),
                }
            )
        }
        // maximize fee_amount || close to target_sqrt_price but not reached
        {
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::new(U128T::from(1000001899999999999999999u128)),
                amount_in: TokenAmount::new(
                    U256T::from_dec_str(
                        "2200049695509007711889927822791908294976419858560291638216994249494",
                    )
                    .unwrap(),
                ),
                amount_out: TokenAmount::new(U256T::from_dec_str(
                    "2200045515422528409085952759527180615861658807361317178894970210709",
                ).unwrap()),
                fee_amount: TokenAmount::new(U256T::from_dec_str(
                    "115792089235116145728061977296797980030478076370664144180897292369696135390441",
                ).unwrap()),
            };

            let result = compute_swap_step(
                one_sqrt_price,
                two_sqrt_price,
                max_liquidity,
                TokenAmount::max_instance(),
                true,
                max_fee - Percentage::new(U128T::from(19)),
            )
            .unwrap();
            assert_eq!(result, expected_result)
        }
        // get_next_sqrt_price_from_input -> get_next_sqrt_price_x_up
        {
            // by_amount_in == true
            // x_to_y == true => current_sqrt_price >= target_sqrt_price == true

            let result = compute_swap_step(
                max_sqrt_price,
                min_sqrt_price,
                max_liquidity,
                max_amount_not_reached_target_sqrt_price,
                true,
                min_fee,
            )
            .unwrap();

            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::new(U128T::from(15258932000000000000u128)),
                    amount_in: TokenAmount::new(U256T::from_dec_str(
                        "75884792730156830614567103553061795263351065677581979504561495713443442818879"
                    ).unwrap()),
                    amount_out: TokenAmount::new(U256T::from_dec_str(
                        "75884790229800029582010010030152469040784228171629896065450012281800526658805"
                    ).unwrap()),
                    fee_amount: TokenAmount::new(U256T::from(0)),
                }
            )
        }

        // get_next_sqrt_price_from_input -> get_next_sqrt_price_y_down
        {
            // by_amount_in == true
            // x_to_y == false => current_sqrt_price >= target_sqrt_price == false

            // 1. scale - maximize amount_after_fee => (max_amount, min_fee) && not reached target
            {
                let result = compute_swap_step(
                    min_sqrt_price,
                    max_sqrt_price,
                    max_liquidity,
                    max_amount_not_reached_target_sqrt_price,
                    true,
                    min_fee,
                )
                .unwrap();

                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: SqrtPrice::new(U128T::from(
                            65535383934512647000000000000u128
                        )),
                        amount_in: TokenAmount::new(U256T::from_dec_str(
                            "75884790229800029582010010030152469040784228171629896065450012281800526658806"
                        ).unwrap()),
                        amount_out: TokenAmount::new(U256T::from_dec_str(
                            "75884792730156830614567103553061795263351065677581979504561495713443442818878"
                        ).unwrap()),
                        fee_amount: TokenAmount::new(U256T::from(0)),
                    }
                )
            }
            // 2. checked_big_div - no possible to trigger from compute_swap_step
            {
                let min_overflow_token_amount = TokenAmount::new(U256T::from(340282366920939u128));
                let result = compute_swap_step(
                    min_sqrt_price,
                    max_sqrt_price,
                    one_liquidity - Liquidity::new(U256T::from(1)),
                    min_overflow_token_amount - TokenAmount::new(U256T::from(1)),
                    true,
                    min_fee,
                )
                .unwrap();
                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: max_sqrt_price,
                        amount_in: TokenAmount::new(U256T::from(65535)),
                        amount_out: TokenAmount::new(U256T::from(65534)),
                        fee_amount: TokenAmount::new(U256T::from(0)),
                    }
                )
            }
        }
        // get_next_sqrt_price_from_output -> get_next_sqrt_price_x_up
        {
            // by_amount_in == false
            // x_to_y == false => current_sqrt_price >= target_sqrt_price == false
            // TRY TO UNWRAP IN SUBTRACTION

            // min_sqrt_price different at maximum amount
            {
                let min_diff = 232_826_265_438_719_159_684u128;
                let result = compute_swap_step(
                    max_sqrt_price - SqrtPrice::new(U128T::from(min_diff)),
                    max_sqrt_price,
                    max_liquidity,
                    TokenAmount::new(U256T::MAX - 1),
                    false,
                    min_fee,
                )
                .unwrap();

                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: SqrtPrice::new(U128T::from(
                            65535383934512647000000000000u128
                        )),
                        amount_in: TokenAmount::new(U256T::from_dec_str(
                            "269594397044712364927302271135767871256767389391069984018896158734608"
                        ).unwrap()),
                        amount_out: TokenAmount::new(U256T::from_dec_str("62771017353866807635074993554120737773068233085134433767742").unwrap()),
                        fee_amount: TokenAmount::new(U256T::from(0)),
                    }
                )
                // assert_eq!(cause, "multiplication overflow");
                // assert_eq!(stack.len(), 4);
            }
            // min sqrt_price different at maximum amount
            {
                let result = compute_swap_step(
                    min_sqrt_price,
                    max_sqrt_price,
                    Liquidity::from_integer(281_477_613_507_675u128),
                    TokenAmount::new(U256T::MAX - 1),
                    false,
                    min_fee,
                )
                .unwrap();

                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: SqrtPrice::new(U128T::from(
                            65535383934512647000000000000u128
                        )),
                        amount_in: TokenAmount::new(U256T::from(18446743465900796471u128)),
                        amount_out: TokenAmount::new(U256T::from(18446744073709559494u128)),
                        fee_amount: TokenAmount::new(U256T::from(0)),
                    }
                );
            }
            // min token change
            {
                let result = compute_swap_step(
                    max_sqrt_price - SqrtPrice::from_integer(1),
                    max_sqrt_price,
                    Liquidity::from_integer(100_000_000_00u128),
                    TokenAmount::new(U256T::from(1)),
                    false,
                    min_fee,
                )
                .unwrap();

                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: SqrtPrice::new(U128T::from(
                            65534813412874974599766965330u128
                        )),
                        amount_in: TokenAmount::new(U256T::from(4294783624u128)),
                        amount_out: TokenAmount::new(U256T::from(1)),
                        fee_amount: TokenAmount::new(U256T::from(0)),
                    }
                );
            }
        }
        // max amount_out, by_amount_in == false
        {
            let result = compute_swap_step(
                max_sqrt_price,
                min_sqrt_price,
                max_liquidity,
                max_amount,
                false,
                min_fee,
            )
            .unwrap();

            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::new(U128T::from(15258932000000000000u128)),
                    amount_in: TokenAmount::new(U256T::from_dec_str(
                        "75884792730156830614567103553061795263351065677581979504561495713443442818879"
                    ).unwrap()),
                    amount_out: TokenAmount::new(U256T::from_dec_str(
                        "75884790229800029582010010030152469040784228171629896065450012281800526658805"
                    ).unwrap()),
                    fee_amount: TokenAmount::new(U256T::from(0)),
                }
            )
        }
    }

    #[test]
    fn test_get_next_sqrt_price_y_down() {
        // VALIDATE BASE SAMPLES
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(1);
            let y = TokenAmount::new(U256T::from(1));

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, true).unwrap();

            assert_eq!(result, SqrtPrice::from_integer(2));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(2);
            let y = TokenAmount::new(U256T::from(3));

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, true).unwrap();

            assert_eq!(result, SqrtPrice::from_scale(25, 1));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(2);
            let liquidity = Liquidity::from_integer(3);
            let y = TokenAmount::new(U256T::from(5));

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, true).unwrap();

            assert_eq!(
                result,
                SqrtPrice::from_integer(11).big_div(SqrtPrice::from_integer(3))
            );
        }
        {
            let sqrt_price = SqrtPrice::from_integer(24234);
            let liquidity = Liquidity::from_integer(3000);
            let y = TokenAmount::new(U256T::from(5000));

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, true).unwrap();

            assert_eq!(
                result,
                SqrtPrice::from_integer(72707).big_div(SqrtPrice::from_integer(3))
            );
        }
        // // bool = false
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(2);
            let y = TokenAmount::new(U256T::from(1));

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, false).unwrap();

            assert_eq!(result, SqrtPrice::from_scale(5, 1));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(100_000);
            let liquidity = Liquidity::from_integer(500_000_000);
            let y = TokenAmount::new(U256T::from(4_000));

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, false).unwrap();
            assert_eq!(
                result,
                SqrtPrice::new(U128T::from(99999999992000000_000000000000u128))
            );
        }
        {
            let sqrt_price = SqrtPrice::from_integer(3);
            let liquidity = Liquidity::from_integer(222);
            let y = TokenAmount::new(U256T::from(37));

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, false).unwrap();

            // expected 2.833333333333
            // real     2.999999999999833...
            assert_eq!(
                result,
                SqrtPrice::new(U128T::from(2833333333333_333333333333u128))
            );
        }
    }

    #[test]
    fn test_domain_get_next_sqrt_price_y_down() {
        let min_y = TokenAmount::new(U256T::from(1));
        let max_y = TokenAmount::max_instance();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let almost_min_sqrt_price = min_sqrt_price + SqrtPrice::new(U128T::from(1));
        let almost_max_sqrt_price = max_sqrt_price - SqrtPrice::new(U128T::from(1));
        let min_sqrt_price_outside_domain = SqrtPrice::new(U128T::from(1));
        let min_liquidity = Liquidity::new(U256T::from(1));
        let max_liquidity: Liquidity = Liquidity::max_instance();
        // let min_overflow_token_y = TokenAmount::new(340282366920939);
        let min_overflow_token_y = TokenAmount::new(U256T::from(340282366920940u128));
        // let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let one_liquidity: Liquidity = Liquidity::from_integer(1);

        // Max token y is 2^96 to not cause intermediate overflow

        // min value inside domain
        {
            // increases min_sqrt_price
            {
                let target_sqrt_price = get_next_sqrt_price_y_down(
                    min_sqrt_price,
                    max_liquidity,
                    min_y + TokenAmount::new(U256T::from(u128::MAX) * U256T::from(2u128.pow(32))),
                    true,
                )
                .unwrap();

                // 60000000
                // 15258932000000000001u128 expected
                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(15258932000000000001u128))
                );
            }
            // decreases almost_min_sqrt_price
            {
                let target_sqrt_price =
                    get_next_sqrt_price_y_down(almost_min_sqrt_price, max_liquidity, min_y, false)
                        .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(15258932000000000000u128))
                );
            }
        }
        // max value inside domain
        {
            // decreases max_sqrt_price
            {
                let target_sqrt_price = get_next_sqrt_price_y_down(
                    max_sqrt_price,
                    max_liquidity,
                    min_y + TokenAmount::new(U256T::from(u128::MAX) * U256T::from(2u128.pow(32))),
                    false,
                )
                .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(65535383934512646999999999998u128))
                );
            }
            // increases almost_max_sqrt_price
            {
                let target_sqrt_price: SqrtPrice = get_next_sqrt_price_y_down(
                    almost_max_sqrt_price,
                    max_liquidity,
                    min_y + TokenAmount::new(U256T::from(600000000u128)),
                    true,
                )
                .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(65535383934512646999999999999u128))
                );
            }
        }
        // extension TokenAmount to SqrtPrice decimal overflow
        {
            {
                let (_, cause, stack) =
                    get_next_sqrt_price_y_down(max_sqrt_price, min_liquidity, max_y, true)
                        .unwrap_err()
                        .get();
                assert_eq!(cause, "Can't parse from U448T to U128T");
                assert_eq!(stack.len(), 2);
            }
            {
                let (_, cause, stack) = get_next_sqrt_price_y_down(
                    min_sqrt_price_outside_domain,
                    min_liquidity,
                    max_y,
                    false,
                )
                .unwrap_err()
                .get();
                assert_eq!(cause, "Can't parse from U448T to U128T");
                assert_eq!(stack.len(), 2);
            }
        }
        // overflow in sqrt_price difference
        {
            {
                let result = get_next_sqrt_price_y_down(
                    max_sqrt_price,
                    one_liquidity,
                    min_overflow_token_y - TokenAmount::new(U256T::from(2)),
                    true,
                )
                .unwrap_err();
                let (_, cause, stack) = result.get();
                assert_eq!(cause, "checked_add: (self + rhs) additional overflow");
                assert_eq!(stack.len(), 1);
            }
            {
                let result = get_next_sqrt_price_y_down(
                    min_sqrt_price_outside_domain,
                    one_liquidity,
                    min_overflow_token_y - TokenAmount::new(U256T::from(2)),
                    false,
                )
                .unwrap_err();
                let (_, cause, stack) = result.get();
                assert_eq!(cause, "checked_sub: (self - rhs) subtraction underflow");
                assert_eq!(stack.len(), 1);
            }
        }

        // quotient overflow
        // max params to max result
        // unwrap_err on result
        // min liq max amount, max sqrt_price
        // min sqrt highest underflow
        {
            {
                let min_y_overflow_decimal_extension = TokenAmount::new(U256T::from(1) << 225);

                let irrelevant_sqrt_price = SqrtPrice::new(U128T::from(1));
                let irrelevant_liquidity: Liquidity = Liquidity::from_integer(1);

                {
                    let (_, cause, stack) = get_next_sqrt_price_y_down(
                        irrelevant_sqrt_price,
                        irrelevant_liquidity,
                        min_y_overflow_decimal_extension,
                        true,
                    )
                    .unwrap_err()
                    .get();
                    assert_eq!(cause, "Can't parse from U448T to U128T");
                    assert_eq!(stack.len(), 2);
                }
                {
                    let (_, cause, stack) = get_next_sqrt_price_y_down(
                        irrelevant_sqrt_price,
                        irrelevant_liquidity,
                        min_y_overflow_decimal_extension,
                        false,
                    )
                    .unwrap_err()
                    .get();
                    assert_eq!(cause, "Can't parse from U448T to U128T");
                    assert_eq!(stack.len(), 2);
                }
            }
        }
        // y_max
        {
            {
                let target_sqrt_price =
                    get_next_sqrt_price_y_down(min_sqrt_price, max_liquidity, max_y, true).unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(100000000015258932000000000000u128)) // Tick 1
                );
            }
        }

        // L == 0
        {
            {
                let (_, cause, stack) = get_next_sqrt_price_y_down(
                    min_sqrt_price,
                    Liquidity::new(U256T::from(0)),
                    min_y,
                    true,
                )
                .unwrap_err()
                .get();
                assert_eq!(cause, "division overflow or division by zero");
                assert_eq!(stack.len(), 2);
            }
        }
        // TokenAmount is zero
        {
            {
                let target_sqrt_price = get_next_sqrt_price_y_down(
                    min_sqrt_price,
                    max_liquidity,
                    TokenAmount::new(U256T::from(0)),
                    true,
                )
                .unwrap();

                assert_eq!(target_sqrt_price, min_sqrt_price);
            }
        }
    }

    #[test]
    fn test_get_delta_x() {
        // validate base samples
        // zero at zero liquidity
        {
            let result = get_delta_x(
                SqrtPrice::from_integer(1u8),
                SqrtPrice::from_integer(1u8),
                Liquidity::new(U256T::from(0)),
                false,
            )
            .unwrap();
            assert_eq!(result, TokenAmount::new(U256T::from(0)));
        }
        // equal at equal liquidity
        {
            let result = get_delta_x(
                SqrtPrice::from_integer(1u8),
                SqrtPrice::from_integer(2u8),
                Liquidity::from_integer(2u8),
                false,
            )
            .unwrap();
            assert_eq!(result, TokenAmount::new(U256T::from(1)));
        }
        // complex
        {
            let sqrt_price_a = SqrtPrice::new(U128T::from(234__878_324_943_782_000000000000u128));
            let sqrt_price_b = SqrtPrice::new(U128T::from(87__854_456_421_658_000000000000u128));
            let liquidity = Liquidity::new(U256T::from(983_983__249_092u128));

            let result_down = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, false).unwrap();
            let result_up = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, true).unwrap();

            // 7010.8199533068819376891841727789301497024557314488455622925765280
            assert_eq!(result_down, TokenAmount::new(U256T::from(70108)));
            assert_eq!(result_up, TokenAmount::new(U256T::from(70109)));
        }
        // big
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::from_scale(5u8, 1);
            let liquidity = Liquidity::from_integer(2u128.pow(64) - 1);

            let result_down = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, false).unwrap();
            let result_up = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, true).unwrap();

            assert_eq!(
                result_down,
                TokenAmount::new(U256T::from(2u128.pow(64) - 1))
            );
            assert_eq!(result_up, TokenAmount::new(U256T::from(2u128.pow(64) - 1)));
        }
        // no more overflow after extending the type in intermediate operations
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::from_scale(5u8, 1);
            let liquidity = Liquidity::max_instance();

            let result_down = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, false);
            let result_up = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, true);
            assert!(result_down.is_ok());
            assert!(result_up.is_ok());
        }
        // huge liquidity
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::new(U128T::from(
                SqrtPrice::one().get() + SqrtPrice::new(U128T::from(1000000)).get(),
            ));
            let liquidity = Liquidity::from_integer(2u128.pow(80));

            let result_down = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, false);
            let result_up = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, true);

            assert!(result_down.is_ok());
            assert!(result_up.is_ok());
        }
    }

    #[test]
    fn test_domain_get_delta_x() {
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        // let almost_max_sqrt_price = SqrtPrice::from_tick(MAX_TICK - 1);
        let almost_min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK + 1).unwrap();

        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(U256T::from(1));

        // maximize delta_sqrt_price and liquidity
        {
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, max_liquidity, true).unwrap();

                assert_eq!(
                result,
                TokenAmount::new(U256T::from_dec_str("75884792730156830614567103553061795263351065677581979504561495713443442818879").unwrap())
            );
            }
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, max_liquidity, false).unwrap();

                assert_eq!(
                result,
                TokenAmount::new(U256T::from_dec_str("75884792730156830614567103553061795263351065677581979504561495713443442818878").unwrap())
            )
            }
        }
        {
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, min_liquidity, true).unwrap();

                assert_eq!(result, TokenAmount::new(U256T::from(1)));
            }
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, min_liquidity, false).unwrap();

                assert_eq!(result, TokenAmount::new(U256T::from(0)))
            }
        }
        // minimize denominator on maximize liquidity which fit into TokenAmount
        {
            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, max_liquidity, true)
                        .unwrap();

                assert_eq!(
                TokenAmount::new(
                    U256T::from_dec_str(
                        "3794315473971847510172532341754979462199874072217062973965311338137066234"
                    )
                    .unwrap()
                ),
                result
            );
            }
            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, max_liquidity, false)
                        .unwrap();
                assert_eq!(
                TokenAmount::new(
                    U256T::from_dec_str(
                        "3794315473971847510172532341754979462199874072217062973965311338137066233"
                    )
                    .unwrap()
                ),
                result
            );
            }
        }
        // minimize denominator on minimize liquidity which fits into TokenAmount
        {
            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, min_liquidity, true)
                        .unwrap();
                assert_eq!(TokenAmount::new(U256T::from(1)), result);
            }
            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, min_liquidity, false)
                        .unwrap();
                assert_eq!(TokenAmount::new(U256T::from(0)), result);
            }
        }

        {
            let search_limit = 256;
            let tick_spacing = 100;
            let max_search_limit = MAX_TICK - (search_limit * tick_spacing);
            let min_search_sqrt_price = SqrtPrice::from_tick(max_search_limit).unwrap();
            let liquidity = Liquidity::max_instance();

            let result =
                get_delta_x(max_sqrt_price, min_search_sqrt_price, liquidity, true).unwrap();
            /*
                    search_limit 256 * tick_spacing (max 100)
                    MAX_TICK <-> MAX_TICK - (search_limit * tick_spacing)
                    sqrt(1.0001^MAX_TICK) * 10^24 -> sqrt(1.0001^(MAX_TICK - SEARCH_LIMIT * MAX_TICK_SPACING)) * 10^24

                    MAX_TICK - SEARCH_LIMIT * MAX_TICK_SPACING = 196218
                    ceil(log2(max_sqrt_price)) < 96
                    ceil(log2(min_search_price)) < 94

                    max_nominator = (sqrt(1.0001)^221818 - sqrt(1.0001)^196218) * 10^24 * 2^256 / 10^6
                    max_nominator < 2^332
                    max_nominator_intermediate = (sqrt(1.0001)^221818 - sqrt(1.0001)^196218) * 10^24 * 2^256
                    max_nominator < 2^352

                    denominator = (sqrt(1.0001)^221818 - sqrt(1.0001)^196218) * 10^24
                    denominator = 2^96

                    max_big_div_values_to_token_up = ((max_nominator * SqrtPrice::one() + denominator - 1) / denominator + SqrtPrice::almost_one()) / SqrtPrice::one()
                    max_big_div_values_to_token_up = ((2^332 * 10^24 + 2^96 - 1) / 2^96 + 10^24) / 10^24
                    max_big_div_values_to_token_up < 2^236

                    max_big_div_values_to_token_up_intermediate = (max_nominator * SqrtPrice::one() + denominator
                    max_big_div_values_to_token_up_intermediate = 2^332 * 10^24 + 2^96
                    max_big_div_values_to_token_up_intermediate < 2^412 <-- no more overflow  after adding U448T
            */

            assert_eq!(
                result,
                TokenAmount::new(
                    U256T::from_dec_str(
                        "45875017378130362421757891862614875858481775310156442203847653871247"
                    )
                    .unwrap()
                )
            )
        }
        {
            let almost_max_sqrt_price = max_sqrt_price.checked_sub(SqrtPrice::one()).unwrap(); // max_sqrt_price.checked_sub(min_step).unwrap();
            let almost_min_sqrt_price = min_sqrt_price.checked_add(SqrtPrice::one()).unwrap(); //min_sqrt_price.checked_add(min_step).unwrap();

            // max_sqrt_price -> max_sqrt_price - 10^-24  /  max liquidity
            {
                let result =
                    get_delta_x(max_sqrt_price, almost_max_sqrt_price, max_liquidity, true)
                        .unwrap();

                assert_eq!(
                    TokenAmount::new(
                        U256T::from_dec_str(
                            "269608649375997235557394191156352599353486422139915865816324471"
                        )
                        .unwrap()
                    ),
                    result
                );
            }

            // min_sqrt_price -> min_sqrt_price + 10^-24 / max liquidity

            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, max_liquidity, true)
                        .unwrap();

                assert_eq!(
                TokenAmount::new(
                    U256T::from_dec_str(
                        "75883634844601460750582416171430603974060896681619645705711819135499453546638"
                    )
                    .unwrap()
                ),
                result
            );
            }
        }
        // liquidity is zero
        {
            let zero_liquidity = Liquidity::new(U256T::from(0));
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, zero_liquidity, true).unwrap();
                assert_eq!(TokenAmount::new(U256T::from(0)), result);
            }
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, zero_liquidity, false).unwrap();
                assert_eq!(TokenAmount::new(U256T::from(0)), result);
            }
        }
        // }
    }
    #[test]
    fn test_get_delta_y() {
        // base samples
        // zero at zero liquidity
        {
            let result = get_delta_y(
                SqrtPrice::from_integer(1),
                SqrtPrice::from_integer(1),
                Liquidity::new(U256T::from(0)),
                false,
            )
            .unwrap();
            assert_eq!(result, TokenAmount::new(U256T::from(0)));
        }
        // equal at equal liquidity
        {
            let result = get_delta_y(
                SqrtPrice::from_integer(1),
                SqrtPrice::from_integer(2),
                Liquidity::from_integer(2),
                false,
            )
            .unwrap();
            assert_eq!(result, TokenAmount::new(U256T::from(2)));
        }
        // // big numbers
        {
            let sqrt_price_a = SqrtPrice::new(U128T::from(234__878_324_943_782_000000000000u128));
            let sqrt_price_b = SqrtPrice::new(U128T::from(87__854_456_421_658_000000000000u128));
            let liquidity = Liquidity::new(U256T::from(983_983__249_092u128));

            let result_down = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, false).unwrap();
            let result_up = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, true).unwrap();

            // 144669023.842474597804911408
            assert_eq!(result_down, TokenAmount::new(U256T::from(1446690238)));
            assert_eq!(result_up, TokenAmount::new(U256T::from(1446690239)));
        }
        // // big
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::from_integer(2u8);
            let liquidity = Liquidity::from_integer(2u128.pow(64) - 1);

            let result_down = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, false).unwrap();
            let result_up = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, true).unwrap();

            assert_eq!(
                result_down,
                TokenAmount::new(liquidity.get() / Liquidity::one().get())
            );
            assert_eq!(
                result_up,
                TokenAmount::new(liquidity.get() / Liquidity::one().get())
            );
        }
        // // overflow
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::max_instance();
            let liquidity = Liquidity::max_instance();

            let result_down = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, false);
            let result_up = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, true);

            assert!(!result_down.is_ok());
            assert!(!result_up.is_ok());
        }
        // // huge liquidity
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::one() + SqrtPrice::new(U128T::from(1000000));
            let liquidity = Liquidity::new(U256T::MAX);

            let result_down = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, false);
            let result_up = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, true);

            assert!(result_down.is_ok());
            assert!(result_up.is_ok());
        }
    }

    #[test]
    fn test_domain_get_delta_y() {
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(U256T::from(1));
        // maximize delta_sqrt_price and liquidity
        {
            {
                let result =
                    get_delta_y(max_sqrt_price, min_sqrt_price, max_liquidity, true).unwrap();
                assert_eq!(
                result,
                TokenAmount::new(U256T::from_dec_str(
                    "75884790229800029582010010030152469040784228171629896065450012281800526658806"
                ).unwrap())
            );
            }
            {
                let result =
                    get_delta_y(max_sqrt_price, min_sqrt_price, max_liquidity, false).unwrap();
                assert_eq!(
                result,
                TokenAmount::new(U256T::from_dec_str(
                    "75884790229800029582010010030152469040784228171629896065450012281800526658805"
                ).unwrap())
            );
            }
            // can be zero
            {
                let result = get_delta_y(
                    max_sqrt_price,
                    SqrtPrice::new(max_sqrt_price.get() - 1),
                    min_liquidity,
                    false,
                )
                .unwrap();
                assert_eq!(result, TokenAmount::new(U256T::from(0)));
            }
        }
        // liquidity is zero
        {
            let result = get_delta_y(
                max_sqrt_price,
                min_sqrt_price,
                Liquidity::new(U256T::from(0)),
                true,
            )
            .unwrap();
            assert_eq!(result, TokenAmount::new(U256T::from(0)));
        }
        {
            let result = get_delta_y(max_sqrt_price, max_sqrt_price, max_liquidity, true).unwrap();
            assert_eq!(result, TokenAmount::new(U256T::from(0)));
        }
    }

    #[test]
    fn test_get_next_sqrt_price_x_up() {
        // basic samples
        // Add
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(1);
            let x = TokenAmount::new(U256T::from(1));

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, true);

            assert_eq!(result.unwrap(), SqrtPrice::from_scale(5, 1));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(2);
            let x = TokenAmount::new(U256T::from(3));

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, true);

            assert_eq!(result.unwrap(), SqrtPrice::from_scale(4, 1));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(2);
            let liquidity = Liquidity::from_integer(3);
            let x = TokenAmount::new(U256T::from(5));

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, true);

            assert_eq!(
                result.unwrap(),
                SqrtPrice::new(U128T::from(461538461538461538461539u128)) // rounded up Decimal::from_integer(6).div(Decimal::from_integer(13))
            );
        }
        {
            let sqrt_price = SqrtPrice::from_integer(24234);
            let liquidity = Liquidity::from_integer(3000);
            let x = TokenAmount::new(U256T::from(5000));

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, true);

            assert_eq!(
                result.unwrap(),
                SqrtPrice::new(U128T::from(599985145205615112277488u128)) // rounded up Decimal::from_integer(24234).div(Decimal::from_integer(40391))
            );
        }
        // Subtract
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(2);
            let x = TokenAmount::new(U256T::from(1));

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, false);

            assert_eq!(result.unwrap(), SqrtPrice::from_integer(2));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(100_000);
            let liquidity = Liquidity::from_integer(500_000_000);
            let x = TokenAmount::new(U256T::from(4_000));

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, false);

            assert_eq!(result.unwrap(), SqrtPrice::from_integer(500_000));
        }
        {
            let sqrt_price = SqrtPrice::new(U128T::from(3_333333333333333333333333u128));
            let liquidity = Liquidity::new(U256T::from(222_22222));
            let x = TokenAmount::new(U256T::from(37));

            // expected 7490636797542399944773031
            // real     7.490636797542399944773031...
            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, false);

            assert_eq!(
                result.unwrap(),
                SqrtPrice::new(U128T::from(7490636797542399944773031u128))
            );
        }
    }

    #[test]
    fn test_domain_get_next_sqrt_price_x_up() {
        // DOMAIN:
        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(U256T::from(1));
        // let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK);
        let max_x = TokenAmount::max_instance();
        let min_x = TokenAmount::new(U256T::from(1));
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let almost_min_sqrt_price = min_sqrt_price + SqrtPrice::new(U128T::from(1));
        let almost_max_sqrt_price = max_sqrt_price - SqrtPrice::new(U128T::from(1));
        // min value inside domain
        {
            // increases min_sqrt_price
            {
                let target_sqrt_price =
                                  // get_next_sqrt_price_y_down(min_sqrt_price, max_liquidity, min_x, true).unwrap();
                                  get_next_sqrt_price_x_up(min_sqrt_price, max_liquidity, TokenAmount::new(U256T::from(600000000)), false).unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(15258932000000000001u128))
                );
            }
            // decreases almost_min_sqrt_price
            {
                let target_sqrt_price = get_next_sqrt_price_x_up(
                    almost_min_sqrt_price,
                    max_liquidity,
                    TokenAmount::new(U256T::from(u128::MAX) * U256T::from(2u128.pow(64))),
                    true,
                )
                .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(15258932000000000000u128))
                );
            }
        }
        // max value inside domain
        {
            // decreases max_sqrt_price
            {
                let target_sqrt_price = get_next_sqrt_price_x_up(
                    max_sqrt_price,
                    max_liquidity,
                    TokenAmount::new(U256T::from(u128::MAX)),
                    true,
                )
                .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(65535383934512646999999999999u128))
                );
            }
            // increases almost_max_sqrt_price
            {
                let target_sqrt_price: SqrtPrice = get_next_sqrt_price_x_up(
                    almost_max_sqrt_price,
                    max_liquidity,
                    TokenAmount::new(U256T::from(u128::MAX)),
                    false,
                )
                .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(U128T::from(65535383934512647000000000001u128))
                );
            }
        }
        {
            let result =
                get_next_sqrt_price_x_up(max_sqrt_price, max_liquidity, max_x, true).unwrap();

            assert_eq!(result, SqrtPrice::new(U128T::from(9999999998474106750u128)));
        }
        // subtraction underflow (not possible from upper-level function)
        {
            let (_, cause, stack) =
                get_next_sqrt_price_x_up(max_sqrt_price, min_liquidity, max_x, false)
                    .unwrap_err()
                    .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 2);
        }
        // max possible result test
        {
            let result =
                get_next_sqrt_price_x_up(max_sqrt_price, max_liquidity, min_x, true).unwrap();

            assert_eq!(
                result,
                SqrtPrice::new(U128T::from(65535383934512647000000000000u128))
            );
        }
        // Liquidity is zero
        {
            let result = get_next_sqrt_price_x_up(
                max_sqrt_price,
                Liquidity::new(U256T::from(0)),
                min_x,
                true,
            )
            .unwrap();

            assert_eq!(result, SqrtPrice::new(U128T::from(0)));
        }
        // Amount is zero
        {
            let result = get_next_sqrt_price_x_up(
                max_sqrt_price,
                max_liquidity,
                TokenAmount::new(U256T::from(0)),
                true,
            )
            .unwrap();

            assert_eq!(
                result,
                SqrtPrice::new(U128T::from(65535383934512647000000000000u128))
            );
        }
    }

    #[test]
    fn test_domain_is_enough_amount_to_push_price() {
        let zero_liquidity = Liquidity::new(U256T::from(0));
        let max_fee = Percentage::from_integer(1);
        let max_amount = TokenAmount::max_instance();
        let min_amount = TokenAmount::new(U256T::from(1));

        // Validate traceable error
        let min_liquidity = Liquidity::new(U256T::from(1));
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_fee = Percentage::from_integer(0);
        {
            let (_, cause, stack) = is_enough_amount_to_change_price(
                TokenAmount::max_instance(),
                max_sqrt_price,
                min_liquidity,
                min_fee,
                false,
                false,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 4);
        }

        // Percentage Max
        {
            let (_, cause, stack) = is_enough_amount_to_change_price(
                min_amount,
                max_sqrt_price,
                min_liquidity,
                max_fee,
                false,
                false,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 4);
        }

        // Liquidity is 0
        {
            let result = is_enough_amount_to_change_price(
                max_amount,
                max_sqrt_price,
                zero_liquidity,
                max_fee,
                false,
                false,
            )
            .unwrap();
            assert!(result)
        }
        // Amount Min
        {
            let (_, cause, stack) = is_enough_amount_to_change_price(
                min_amount,
                max_sqrt_price,
                min_liquidity,
                min_fee,
                false,
                false,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 4);
        }
        // Amount Max
        {
            let (_, cause, stack) = is_enough_amount_to_change_price(
                max_amount,
                max_sqrt_price,
                min_liquidity,
                min_fee,
                false,
                false,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 4);
        }
    }

    #[test]
    fn test_calculate_amount_delta() {
        // current tick between lower tick and upper tick
        {
            let current_tick_index = 2;
            let current_sqrt_price = SqrtPrice::new(U128T::from(1000140000000_000000000000u128));

            let liquidity_delta = Liquidity::from_integer(5_000_000);
            let liquidity_sign = true;
            let upper_tick = 3;
            let lower_tick = 0;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount::new(U256T::from(51)));
            assert_eq!(y, TokenAmount::new(U256T::from(700)));
            assert_eq!(add, true)
        }
        {
            let current_tick_index = 2;
            let current_sqrt_price = SqrtPrice::new(U128T::from(1000140000000_000000000000u128));

            let liquidity_delta = Liquidity::from_integer(5_000_000);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 0;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount::new(U256T::from(300)));
            assert_eq!(y, TokenAmount::new(U256T::from(700)));
            assert_eq!(add, true)
        }
        // current tick smaller than lower tick
        {
            let current_tick_index = 0;
            let current_sqrt_price = Default::default();
            let liquidity_delta = Liquidity::from_integer(10);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 2;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount::new(U256T::from(1)));
            assert_eq!(y, TokenAmount::new(U256T::from(0)));
            assert_eq!(add, false)
        }
        // current tick greater than upper tick
        {
            let current_tick_index = 6;
            let current_sqrt_price = Default::default();

            let liquidity_delta = Liquidity::from_integer(10);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 2;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount::new(U256T::from(0)));
            assert_eq!(y, TokenAmount::new(U256T::from(1)));
            assert_eq!(add, false)
        }
    }

    #[test]
    fn test_domain_calculate_amount_delta() {
        // DOMAIN
        let max_liquidity = Liquidity::max_instance();

        // max x
        {
            let current_tick_index = -MAX_TICK;
            let current_sqrt_price = Default::default();

            let liquidity_sign = true;
            let upper_tick = MAX_TICK;
            let lower_tick = -MAX_TICK + 1;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                max_liquidity,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();
            assert_eq!(
                x,
                TokenAmount::new(
                    U256T::from_dec_str("75880998414682858767056931020720040283888865803509762441587530402105305752645").unwrap()
                )
            );
            assert_eq!(y, TokenAmount::new(U256T::from(0)));
            assert_eq!(add, false)
        }

        // max y
        {
            let current_tick_index = MAX_TICK;
            let current_sqrt_price = Default::default();
            let liquidity_sign = true;
            let upper_tick = MAX_TICK - 1;
            let lower_tick = -MAX_TICK;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                max_liquidity,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();
            assert_eq!(x, TokenAmount::new(U256T::from(0)));
            assert_eq!(
                y,
                TokenAmount::new(
                    U256T::from_dec_str("75880996274614937472454279923345931777432945506580976077368827511053494714377").unwrap()
                )
            );
            assert_eq!(add, false)
        }

        // delta liquidity = 0
        {
            let current_tick_index = 2;
            let current_sqrt_price = SqrtPrice::new(U128T::from(1000140000000_000000000000u128));

            let liquidity_delta = Liquidity::from_integer(0);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 0;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount::new(U256T::from(0)));
            assert_eq!(y, TokenAmount::new(U256T::from(0)));
            assert_eq!(add, true)
        }

        // Error handling
        {
            let current_tick_index = 0;
            let current_sqrt_price = SqrtPrice::new(U128T::from(1000140000000_000000000000u128));
            let liquidity_delta = Liquidity::from_integer(0);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 10;

            let (_, cause, stack) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap_err()
            .get();
            assert_eq!(cause, "upper_tick is not greater than lower_tick");
            assert_eq!(stack.len(), 1);
        }
        {
            let max_sqrt_price = SqrtPrice::max_instance(); // 2^128 - 1
            let max_liquidity = Liquidity::max_instance();
            {
                let current_tick_index = 0;
                let current_sqrt_price = max_sqrt_price;
                let liquidity_sign = true;
                let upper_tick = MAX_TICK;
                let lower_tick = -MAX_TICK;

                let (_, cause, stack) = calculate_amount_delta(
                    current_tick_index,
                    current_sqrt_price,
                    max_liquidity,
                    liquidity_sign,
                    upper_tick,
                    lower_tick,
                )
                .unwrap_err()
                .get();
                assert_eq!(
                    cause,
                    "conversion to invariant::types::token_amount::TokenAmount type failed"
                );
                assert_eq!(stack.len(), 2)
            }
        }
    }

    #[test]
    fn test_check_ticks() {
        {
            let tick_lower = -10;
            let tick_upper = 10;
            let tick_spacing = 1;
            let result = check_ticks(tick_lower, tick_upper, tick_spacing);
            assert!(result.is_ok())
        }
        {
            let tick_lower = -12;
            let tick_upper = 12;
            let tick_spacing = 4;
            let result = check_ticks(tick_lower, tick_upper, tick_spacing);
            assert!(result.is_ok())
        }
        {
            let tick_lower = 2000;
            let tick_upper = 5000;
            let tick_spacing = 100;
            let result = check_ticks(tick_lower, tick_upper, tick_spacing);
            assert!(result.is_ok())
        }
        // invalid spacing
        {
            let tick_lower = -2;
            let tick_upper = 2;
            let tick_spacing = 4;
            let (_, cause, stack) = check_ticks(tick_lower, tick_upper, tick_spacing)
                .unwrap_err()
                .get();
            assert_eq!(cause, "InvalidTickSpacing");
            assert_eq!(stack.len(), 2);
        }
        // invalid index
        {
            let tick_lower = 0;
            let tick_upper = MAX_TICK + 1;
            let tick_spacing = 1;
            let (_, cause, stack) = check_ticks(tick_lower, tick_upper, tick_spacing)
                .unwrap_err()
                .get();
            assert_eq!(cause, "InvalidTickIndex");
            assert_eq!(stack.len(), 2);
        }
        // lower > upper
        {
            let tick_lower = 20;
            let tick_upper = 0;
            let tick_spacing = 5;
            let (_, cause, stack) = check_ticks(tick_lower, tick_upper, tick_spacing)
                .unwrap_err()
                .get();
            assert_eq!(cause, "tick_lower > tick_upper");
            assert_eq!(stack.len(), 1);
        }
    }

    #[test]
    fn test_calculate_max_liquidity_per_tick() {
        // tick_spacing 1 [L_MAX / 443_637]
        {
            let max_l = calculate_max_liquidity_per_tick(1);
            assert_eq!(
                max_l,
                Liquidity::new(
                    U256T::from_dec_str(
                        "261006384132333857238172165551313140818439365214444611336425014162283870"
                    )
                    .unwrap()
                )
            );
        };
        // tick_spacing 2 [L_MAX / 221_819]
        {
            let max_l = calculate_max_liquidity_per_tick(2);
            assert_eq!(
                max_l,
                Liquidity::new(U256T::from(
                    U256T::from_dec_str(
                        "522013944933757384087725004321957225532959384115087883036803072825077900"
                    )
                    .unwrap()
                ))
            );
        }
        // tick_spacing 5 [L_MAX / 88_727]
        {
            let max_l = calculate_max_liquidity_per_tick(5);
            assert_eq!(
                max_l,
                Liquidity::new(U256T::from(
                    U256T::from_dec_str(
                        "1305037804020379314341417888677492847197245310510223089245185614389229091"
                    )
                    .unwrap()
                ))
            );
        }
        // tick_spacing 100 [L_MAX / 4436]
        {
            let max_l = calculate_max_liquidity_per_tick(100);
            assert_eq!(
            max_l,
            Liquidity::new(U256T::from(
                U256T::from_dec_str(
                    "26102815427708790672581376241814226296949951457538449963809193870133708214"
                )
                .unwrap()
            ))
        );
        }
    }
}
