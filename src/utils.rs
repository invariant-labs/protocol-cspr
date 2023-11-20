use alloc::string::ToString;
use decimal::*;
use invariant_math::{liquidity::Liquidity, U256T};
use odra::types::U256;

pub fn uint_to_liquidity(uint: U256) -> Liquidity {
    Liquidity::new(U256T::from_dec_str(uint.to_string().as_str()).unwrap())
}

pub fn liquidity_to_uint(liquidity: Liquidity) -> U256 {
    U256::from_dec_str(liquidity.get().to_string().as_str()).unwrap()
}
