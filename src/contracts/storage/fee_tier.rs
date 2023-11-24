use crate::math::percentage::Percentage;
use odra::OdraType;

#[derive(OdraType, Default)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u16,
}
