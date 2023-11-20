use odra::types::Address;
use odra::types::U128;
use odra::OdraType;

#[derive(OdraType)]
pub struct State {
    pub admin: Address,
    pub protocol_fee: U128,
}
