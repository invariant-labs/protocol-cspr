use alloc::string::ToString;
use decimal::*;
use invariant_math::{
    fee_growth::FeeGrowth, liquidity::Liquidity, seconds_per_liquidity::SecondsPerLiquidity,
    sqrt_price::SqrtPrice, U128T, U256T,
};
use odra::types::{U128, U256};

pub fn sqrt_price_to_uint(sqrt_price: SqrtPrice) -> U128 {
    U128::from_dec_str(sqrt_price.get().to_string().as_str()).unwrap()
}

pub fn uint_to_sqrt_price(uint: U128) -> SqrtPrice {
    SqrtPrice::new(U128T::from_dec_str(uint.to_string().as_str()).unwrap())
}

pub fn fee_growth_to_uint(fee_growth: FeeGrowth) -> U128 {
    U128::from_dec_str(fee_growth.get().to_string().as_str()).unwrap()
}

pub fn uint_to_fee_growth(uint: U128) -> FeeGrowth {
    FeeGrowth::new(U128T::from_dec_str(uint.to_string().as_str()).unwrap())
}

pub fn uint_to_liquidity(uint: U256) -> Liquidity {
    Liquidity::new(U256T::from_dec_str(uint.to_string().as_str()).unwrap())
}

pub fn liquidity_to_uint(liquidity: Liquidity) -> U256 {
    U256::from_dec_str(liquidity.get().to_string().as_str()).unwrap()
}

pub fn seconds_per_liquidity_to_uint(seconds_per_liquidity: SecondsPerLiquidity) -> U128 {
    U128::from_dec_str(seconds_per_liquidity.get().to_string().as_str()).unwrap()
}

pub fn uint_to_seconds_per_liquidity(uint: U128) -> SecondsPerLiquidity {
    SecondsPerLiquidity::new(U128T::from_dec_str(uint.to_string().as_str()).unwrap())
}
