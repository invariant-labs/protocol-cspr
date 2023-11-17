use crate::uints::{U256T, U512T};
use decimal::*;
#[decimal(5, U512T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct Liquidity {
    pub v: U256T,
}
