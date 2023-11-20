use alloc::string::ToString;
use decimal::*;
use invariant_math::{
    fee_growth::{self, FeeGrowth},
    liquidity::Liquidity,
    seconds_per_liquidity::SecondsPerLiquidity,
    sqrt_price::SqrtPrice,
    U128T, U256T,
};
use odra::types::{U128, U256};

pub fn sqrt_price_to_uint(sqrt_price: SqrtPrice) -> U128 {
    U128(sqrt_price.get().0)
}

pub fn uint_to_sqrt_price(uint: U128) -> SqrtPrice {
    SqrtPrice::new(U128T(uint.0))
}

pub fn fee_growth_to_uint(fee_growth: FeeGrowth) -> U128 {
    U128(fee_growth.get().0)
}

pub fn uint_to_fee_growth(uint: U128) -> FeeGrowth {
    FeeGrowth::new(U128T(uint.0))
}

pub fn liquidity_to_uint(liquidity: Liquidity) -> U256 {
    U256(liquidity.get().0)
}

pub fn uint_to_liquidity(uint: U256) -> Liquidity {
    Liquidity::new(U256T(uint.0))
}

pub fn seconds_per_liquidity_to_uint(seconds_per_liquidity: SecondsPerLiquidity) -> U128 {
    U128(seconds_per_liquidity.get().0)
}

pub fn uint_to_seconds_per_liquidity(uint: U128) -> SecondsPerLiquidity {
    SecondsPerLiquidity::new(U128T(uint.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uint_to_decimal() {
        let sqrt_price = uint_to_sqrt_price(U128::from(1));
        let fee_growth = uint_to_fee_growth(U128::from(1));
        let liquidity = uint_to_liquidity(U256::from(1));
        let seconds_per_liquidity = uint_to_seconds_per_liquidity(U128::from(1));

        assert_eq!(sqrt_price, SqrtPrice::new(U128T::from(1)));
        assert_eq!(fee_growth, FeeGrowth::new(U128T::from(1)));
        assert_eq!(liquidity, Liquidity::new(U256T::from(1)));
        assert_eq!(
            seconds_per_liquidity,
            SecondsPerLiquidity::new(U128T::from(1))
        );
    }

    #[test]
    fn test_decimal_to_uint() {
        let sqrt_price = SqrtPrice::new(U128T::from(1));
        let fee_growth = FeeGrowth::new(U128T::from(1));
        let liquidity = Liquidity::new(U256T::from(1));
        let seconds_per_liquidity = SecondsPerLiquidity::new(U128T::from(1));

        assert_eq!(sqrt_price_to_uint(sqrt_price), U128::from(1));
        assert_eq!(fee_growth_to_uint(fee_growth), U128::from(1));
        assert_eq!(liquidity_to_uint(liquidity), U256::from(1));
        assert_eq!(
            seconds_per_liquidity_to_uint(seconds_per_liquidity),
            U128::from(1)
        );
    }
}
