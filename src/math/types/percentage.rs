use decimal::*;

use crate::uints::{U128T, U256T};

// TODO: Update underlying type to U64T
#[decimal(12, U256T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct Percentage {
    pub v: U128T,
}
