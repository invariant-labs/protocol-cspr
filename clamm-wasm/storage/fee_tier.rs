use crate::errors::InvariantError;
use crate::percentage::Percentage;
use decimal::*;
use odra::types::U128;
use odra::OdraType;

use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(OdraType, Eq, PartialEq, Copy, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FeeTier {
    pub fee: Percentage,
    #[tsify(type = "bigint")]
    pub tick_spacing: u32,
}

impl Default for FeeTier {
    fn default() -> Self {
        Self {
            fee: Percentage::new(U128::from(0)),
            tick_spacing: 1,
        }
    }
}

impl FeeTier {
    pub fn new(fee: Percentage, tick_spacing: u32) -> Result<Self, InvariantError> {
        if tick_spacing == 0 || tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        if fee > Percentage::from_integer(1) {
            return Err(InvariantError::InvalidFee);
        }

        Ok(Self { fee, tick_spacing })
    }
}
