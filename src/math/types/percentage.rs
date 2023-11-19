use decimal::*;

use crate::uints::{U128T, U256T};

#[decimal(12, U256T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct Percentage {
    pub v: U128T,
}
