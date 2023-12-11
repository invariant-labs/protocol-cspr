use crate::math::percentage::Percentage;
use crate::InvariantError;
use decimal::*;
use odra::types::U128;
use odra::OdraType;

#[derive(OdraType, Eq, PartialEq, Copy, Debug)]
pub struct FeeTier {
    pub fee: Percentage,
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
