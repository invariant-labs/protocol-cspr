use decimal::*;

use crate::uints::{U256T, U512T};

#[decimal(0, U512T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct TokenAmount {
    pub v: U256T,
}
