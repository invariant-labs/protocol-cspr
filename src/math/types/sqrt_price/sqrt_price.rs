use crate::uints::{U128T, U256T, U384T, U448T};
use decimal::*;
use traceable_result::*;

use crate::consts::*;
use crate::types::fixed_point::FixedPoint;
use crate::types::token_amount::TokenAmount;

#[decimal(24, U384T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct SqrtPrice {
    pub v: U128T,
}

impl SqrtPrice {
    pub fn from_tick(i: i32) -> TrackableResult<Self> {
        calculate_sqrt_price(i)
    }

    // TODO - Configure nominator and denominator types
    pub fn big_div_values_to_token(
        nominator: U384T,
        denominator: U384T,
    ) -> TrackableResult<TokenAmount> {
        let nominator: U448T = SqrtPrice::from_value::<U448T, U384T>(nominator);
        let denominator: U448T = SqrtPrice::from_value::<U448T, U384T>(denominator);
        let intermediate_u448 = nominator
            .checked_mul(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        // TODO - add ok_or_mark_trace!
        // Possible overflow | U320T should be enough
        let casted_intermediate: U384T =
            (SqrtPrice::checked_from_value::<U384T, U448T>(intermediate_u448))
                .map_err(|_| err!("Can't parse from U448T to U384T"))?;

        let result: U384T = casted_intermediate
            .checked_div(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let casted_result: U256T = TokenAmount::checked_from_value::<U256T, U384T>(result)
            .map_err(|_| err!("Can't parse from U384T to U256T"))?;

        Ok(TokenAmount::new(casted_result))
    }

    // TODO - Configure nominator and denominator types
    pub fn big_div_values_to_token_up(
        nominator: U384T,
        denominator: U384T,
    ) -> TrackableResult<TokenAmount> {
        let nominator: U448T = SqrtPrice::from_value::<U448T, U384T>(nominator);
        let denominator: U448T = SqrtPrice::from_value::<U448T, U384T>(denominator);

        let intermediate_u448 = nominator
            .checked_mul(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_add(denominator - 1)
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        // TODO - add ok_or_mark_trace!
        // Possible overflow | U320T should be enough
        let casted_intermediate: U384T =
            (SqrtPrice::checked_from_value::<U384T, U448T>(intermediate_u448))
                .map_err(|_| err!("Can't parse from U448T to U384T"))?;

        let result: U384T = casted_intermediate
            .checked_add(Self::almost_one().cast())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let casted_result: U256T = TokenAmount::checked_from_value::<U256T, U384T>(result)
            .map_err(|_| err!("Can't parse from U384T to U256T"))?;

        Ok(TokenAmount::new(casted_result))
    }

    // TODO - Configure nominator and denominator types
    pub fn big_div_values_up(nominator: U384T, denominator: U384T) -> SqrtPrice {
        let result = nominator
            .checked_mul(Self::one().cast())
            .unwrap()
            .checked_add(denominator.checked_sub(U384T::from(1u32)).unwrap())
            .unwrap()
            .checked_div(denominator)
            .unwrap();
        let casted_result = SqrtPrice::from_value::<U128T, U384T>(result);
        SqrtPrice::new(casted_result)
    }

    // TODO - Configure nominator and denominator types
    pub fn checked_big_div_values(
        nominator: U448T,
        denominator: U448T,
    ) -> TrackableResult<SqrtPrice> {
        let result = nominator
            .checked_mul(Self::one().cast())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let casted_result = SqrtPrice::checked_from_value::<U128T, U448T>(result)
            .map_err(|_| err!("Can't parse from U448T to U128T"))?;
        Ok(SqrtPrice::new(casted_result))
    }

    // TODO - Configure nominator and denominator types
    pub fn checked_big_div_values_up(
        nominator: U448T,
        denominator: U448T,
    ) -> TrackableResult<SqrtPrice> {
        let result = nominator
            .checked_mul(Self::one().cast())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_add(
                denominator
                    .checked_sub(U448T::from(1u32))
                    .ok_or_else(|| err!(TrackableError::SUB))?,
            )
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        // TODO - add ok_or_mark_trace!
        let casted_result = SqrtPrice::checked_from_value::<U128T, U448T>(result)
            .map_err(|_| err!("Can't parse from U448T to U128T"))?;
        Ok(SqrtPrice::new(casted_result))
    }
}

pub fn calculate_sqrt_price(tick_index: i32) -> TrackableResult<SqrtPrice> {
    // checking if tick be converted to sqrt_price (overflows if more)
    let tick = tick_index.abs();

    if tick > MAX_TICK {
        return Err(err!("tick over bounds"));
    }

    let mut sqrt_price = FixedPoint::from_integer(1);

    if tick & 0x1 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1000049998750u128));
    }
    if tick & 0x2 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1000100000000u128));
    }
    if tick & 0x4 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1000200010000u128));
    }
    if tick & 0x8 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1000400060004u128));
    }
    if tick & 0x10 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1000800280056u128));
    }
    if tick & 0x20 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1001601200560u128));
    }
    if tick & 0x40 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1003204964963u128));
    }
    if tick & 0x80 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1006420201726u128));
    }
    if tick & 0x100 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1012881622442u128));
    }
    if tick & 0x200 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1025929181080u128));
    }
    if tick & 0x400 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1052530684591u128));
    }
    if tick & 0x800 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1107820842005u128));
    }
    if tick & 0x1000 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1227267017980u128));
    }
    if tick & 0x2000 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(1506184333421u128));
    }
    if tick & 0x4000 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(2268591246242u128));
    }
    if tick & 0x8000 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(5146506242525u128));
    }
    if tick & 0x0001_0000 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(26486526504348u128));
    }
    if tick & 0x0002_0000 != 0 {
        sqrt_price *= FixedPoint::new(U128T::from(701536086265529u128));
    }

    //TODO - refactor converting to sqrt_price with `checked_from_decimal` method

    // Parsing to the Sqrt_price type by the end by convention (should always have 12 zeros at the end)
    let missing_scale = FixedPoint::new(U128T::from(
        10u128.pow((SqrtPrice::scale() - FixedPoint::scale()) as u32),
    ));
    Ok(if tick_index >= 0 {
        let extended_value = sqrt_price
            .get()
            .checked_mul(missing_scale.get())
            .ok_or_else(|| err!("calculate_sqrt_price::checked_mul multiplication failed"))?;

        SqrtPrice::new(extended_value)
    } else {
        let one = FixedPoint::from_integer(1);
        SqrtPrice::new(
            one.get()
                .checked_mul(FixedPoint::one().get())
                .ok_or_else(|| err!("calculate_sqrt_price::checked_mul multiplication failed"))?
                .checked_div(sqrt_price.get())
                .ok_or_else(|| err!("calculate_sqrt_price::checked_div division failed"))?
                .checked_mul(missing_scale.get())
                .ok_or_else(|| err!("calculate_sqrt_price::checked_mul multiplication failed"))?,
        )
    })
    // if tick_index >= 0 {
    // SqrtPrice::checked_from_decimal(sqrt_price)
    // .map_err(|_| err!("calculate_sqrt_price: parsing from scale failed"))?, // } else {
    //     SqrtPrice::checked_from_decimal(
    //         FixedPoint::from_integer(1)
    //             .checked_div(sqrt_price)
    //             .map_err(|_| err!("calculate_sqrt_price::checked_div division failed"))?,
    //     )
    //     .map_err(|_| err!("calculate_sqrt_price: parsing scale failed"))?
    // }
    // Ok(SqrtPrice::new(U128T::from(0)))
}

pub fn get_max_tick(tick_spacing: u16) -> i32 {
    let tick_spacing = tick_spacing as i32;
    MAX_TICK / tick_spacing * tick_spacing
}

pub fn get_min_tick(tick_spacing: u16) -> i32 {
    let tick_spacing = tick_spacing as i32;
    MIN_TICK / tick_spacing * tick_spacing
}

pub fn get_max_sqrt_price(tick_spacing: u16) -> SqrtPrice {
    let max_tick = get_max_tick(tick_spacing);
    SqrtPrice::from_tick(max_tick).unwrap()
}

pub fn get_min_sqrt_price(tick_spacing: u16) -> SqrtPrice {
    let min_tick = get_min_tick(tick_spacing);
    SqrtPrice::from_tick(min_tick).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sqrt_price() {
        {
            let sqrt_price = SqrtPrice::from_tick(20_000).unwrap();
            // expected 2.718145925979
            // real     2.718145926825...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(2718145925979u128, 12));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(200_000).unwrap();
            // expected 22015.455979766288
            // real     22015.456048527954...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(22015455979766288u128, 12));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(-20_000).unwrap();
            // expected 0.367897834491
            // real     0.36789783437712...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(367897834491u128, 12));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(-200_000).unwrap();
            // expected 0.000045422634
            // real     0.00004542263388...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(45422634u128, 12))
        }
        {
            let sqrt_price = SqrtPrice::from_tick(0).unwrap();
            assert_eq!(sqrt_price, SqrtPrice::from_integer(1));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
            // expected 65535.383934512647
            // real     65535.384161610681...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(65535383934512647u128, 12));
            assert_eq!(sqrt_price, SqrtPrice::new(U128T::from(MAX_SQRT_PRICE)));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(MIN_TICK).unwrap();
            // expected 0.000015258932
            // real     0.0000152589324...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(15258932u128, 12));
            assert_eq!(sqrt_price, SqrtPrice::new(U128T::from(MIN_SQRT_PRICE)));
        }
    }

    #[test]
    fn test_domain_calculate_sqrt_price() {
        // over max tick
        {
            let tick_out_of_range = MAX_TICK + 1;
            let (_, cause, stack) = SqrtPrice::from_tick(tick_out_of_range).unwrap_err().get();
            assert_eq!("tick over bounds", cause);
            assert_eq!(1, stack.len());
        }
        // below min tick
        {
            let tick_out_of_range = -MAX_TICK - 1;
            let (_, cause, stack) = SqrtPrice::from_tick(tick_out_of_range).unwrap_err().get();
            assert_eq!("tick over bounds", cause);
            assert_eq!(1, stack.len());
        }
    }

    #[test]
    fn test_sqrt_price_limitation() {
        {
            let global_max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
            assert_eq!(
                global_max_sqrt_price,
                SqrtPrice::new(U128T::from(MAX_SQRT_PRICE))
            ); // ceil(log2(this)) = 96
            let global_min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
            assert_eq!(
                global_min_sqrt_price,
                SqrtPrice::new(U128T::from(MIN_SQRT_PRICE))
            ); // floor(log2(this)) = 63
        }
        {
            let max_sqrt_price = get_max_sqrt_price(1);
            let max_tick: i32 = get_max_tick(1);
            assert_eq!(max_sqrt_price, SqrtPrice::new(U128T::from(MAX_SQRT_PRICE)));
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(U128T::from(MAX_SQRT_PRICE))
            );

            let max_sqrt_price = get_max_sqrt_price(2);
            let max_tick: i32 = get_max_tick(2);
            assert_eq!(max_sqrt_price, SqrtPrice::new(U128T::from(MAX_SQRT_PRICE)));
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(U128T::from(MAX_SQRT_PRICE))
            );

            let max_sqrt_price = get_max_sqrt_price(5);
            let max_tick: i32 = get_max_tick(5);
            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(U128T::from(65525554855399275000000000000u128))
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(U128T::from(65525554855399275000000000000u128))
            );

            let max_sqrt_price = get_max_sqrt_price(10);
            let max_tick: i32 = get_max_tick(10);
            assert_eq!(max_tick, 221810);
            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(U128T::from(65509176333123237000000000000u128))
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(U128T::from(65509176333123237000000000000u128))
            );

            let max_sqrt_price = get_max_sqrt_price(100);
            let max_tick: i32 = get_max_tick(100);
            assert_eq!(max_tick, 221800);

            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(U128T::from(65476431569071896000000000000u128))
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(U128T::from(65476431569071896000000000000u128))
            );
        }
        {
            let min_sqrt_price = get_min_sqrt_price(1);
            let min_tick: i32 = get_min_tick(1);
            assert_eq!(min_sqrt_price, SqrtPrice::new(U128T::from(MIN_SQRT_PRICE)));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(U128T::from(MIN_SQRT_PRICE))
            );

            let min_sqrt_price = get_min_sqrt_price(2);
            let min_tick: i32 = get_min_tick(2);
            assert_eq!(min_sqrt_price, SqrtPrice::new(U128T::from(MIN_SQRT_PRICE)));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(U128T::from(MIN_SQRT_PRICE))
            );

            let min_sqrt_price = get_min_sqrt_price(5);
            let min_tick: i32 = get_min_tick(5);
            assert_eq!(
                min_sqrt_price,
                SqrtPrice::new(U128T::from(15261221000000000000u128))
            );
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(U128T::from(15261221000000000000u128))
            );

            let min_sqrt_price = get_min_sqrt_price(10);
            let min_tick: i32 = get_min_tick(10);
            assert_eq!(min_tick, -221810);
            assert_eq!(
                min_sqrt_price,
                SqrtPrice::new(U128T::from(15265036000000000000u128))
            );
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(U128T::from(15265036000000000000u128))
            );

            let min_sqrt_price = get_min_sqrt_price(100);
            let min_tick: i32 = get_min_tick(100);
            assert_eq!(min_tick, -221800);
            assert_eq!(
                min_sqrt_price,
                SqrtPrice::new(U128T::from(15272671000000000000u128))
            );
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(U128T::from(15272671000000000000u128))
            );
        }
    }
}
