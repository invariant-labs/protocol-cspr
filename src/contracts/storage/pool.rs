use crate::contracts::Oracle;
use decimal::num_traits::WrappingAdd;
use decimal::*;
use invariant_math::liquidity::Liquidity;
use invariant_math::seconds_per_liquidity::SecondsPerLiquidity;
use invariant_math::U256T;
use odra::types::{Address, U128, U256};
use odra::OdraType;
use traceable_result::*;

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

impl Pool {
    pub fn update_seconds_per_liquidity_global(
        &mut self,
        current_timestamp: u64,
    ) -> TrackableResult<()> {
        let seconds_per_liquidity_global =
            SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                Liquidity::new(U256T(self.liquidity.0)),
                current_timestamp,
                self.last_timestamp,
            )?;

        self.seconds_per_liquidity_global = self
            .seconds_per_liquidity_global
            .wrapping_add(&U128(seconds_per_liquidity_global.get().0));
        self.last_timestamp = current_timestamp;
        Ok(())
    }
}
