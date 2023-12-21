use crate::math::percentage::Percentage;
use odra::types::Address;

use odra::OdraType;

#[derive(OdraType)]
pub struct InvariantConfig {
    pub admin: Address,
    pub protocol_fee: Percentage,
}
