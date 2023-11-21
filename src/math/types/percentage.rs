use decimal::*;

// use crate::uints::{U128T, U256T};
use odra::types::{U128, U256};

#[decimal(12, U256)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct Percentage {
    pub v: U128,
}
