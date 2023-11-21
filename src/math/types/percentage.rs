use decimal::*;
use odra::{
    types::{U128, U256},
    OdraType,
};

#[decimal(12, U256)]
#[derive(OdraType, Default, Debug, Copy, PartialEq, Eq, PartialOrd)]
pub struct Percentage {
    pub v: U128,
}
