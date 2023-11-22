use crate::math::percentage::Percentage;
use odra::types::Address;

use odra::OdraType;

#[derive(OdraType)]
pub struct State {
    pub admin: Address,
    pub protocol_fee: Percentage,
}
