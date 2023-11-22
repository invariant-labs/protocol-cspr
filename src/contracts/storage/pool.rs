use crate::contracts::Oracle;
use crate::math::seconds_per_liquidity::SecondsPerLiquidity;
use crate::math::{
    fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use odra::types::Address;
use odra::OdraType;
use traceable_result::*;

#[derive(OdraType)]
pub struct Pool {
    pub liquidity: Liquidity,
    pub sqrt_price: SqrtPrice,
    pub current_tick_index: i32, // nearest tick below the current sqrt_price
    pub fee_growth_global_x: FeeGrowth,
    pub fee_growth_global_y: FeeGrowth,
    pub fee_protocol_token_x: TokenAmount,
    pub fee_protocol_token_y: TokenAmount,
    pub seconds_per_liquidity_global: SecondsPerLiquidity,
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
                self.liquidity,
                current_timestamp,
                self.last_timestamp,
            )?;

        self.seconds_per_liquidity_global = self
            .seconds_per_liquidity_global
            .unchecked_add(seconds_per_liquidity_global);
        self.last_timestamp = current_timestamp;
        Ok(())
    }
}
