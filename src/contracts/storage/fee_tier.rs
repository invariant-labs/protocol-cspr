
use odra::types::Address;
use odra::types::{U128, U256};
use odra::OdraType;

#[derive(OdraType)]
pub struct FeeTier {
    pub fee: u64,
    pub tick_spacing: u16,
}
