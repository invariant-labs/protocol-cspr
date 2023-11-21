use decimal::*;

use odra::{
    types::{U256, U512},
    OdraType,
};

#[decimal(0, U512)]
#[derive(OdraType, Default, Debug, Copy, PartialEq, Eq, PartialOrd)]
pub struct TokenAmount {
    pub v: U256,
}
