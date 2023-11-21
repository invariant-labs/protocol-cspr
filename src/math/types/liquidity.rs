use decimal::*;

use odra::{
    types::{U256, U512},
    OdraType,
};

#[decimal(5, U512)]
#[derive(OdraType, Default, Debug, Copy, PartialEq, Eq, PartialOrd)]
pub struct Liquidity {
    pub v: U256,
}
