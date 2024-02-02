use decimal::*;
use odra::{
    types::{U128, U256},
    OdraType,
};

#[derive(OdraType)]
pub struct Test(pub U128);
