
use odra::types::U128;
use odra::OdraType;

#[derive(OdraType, Default)]
pub struct FeeTier {
    pub fee: U128,
    pub tick_spacing: u16,
}
