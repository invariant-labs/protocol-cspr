use odra::OdraType;

use crate::math::percentage::Percentage;

#[derive(OdraType)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u16,
}
