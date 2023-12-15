use crate::math::percentage::Percentage;
use crate::{
    token::{TokenDeployer, TokenRef},
    InvariantDeployer, InvariantRef,
};
use alloc::string::String;
use decimal::Decimal;
use odra::types::U256;

pub fn init(fee: Percentage, supply: U256) -> (InvariantRef, TokenRef, TokenRef) {
    let invariant = InvariantDeployer::init(fee.get());
    let token_0 = TokenDeployer::init(String::from(""), String::from(""), 0, &supply);
    let token_1 = TokenDeployer::init(String::from(""), String::from(""), 0, &supply);
    if token_0.address() < token_1.address() {
        (invariant, token_0, token_1)
    } else {
        (invariant, token_1, token_0)
    }
}
