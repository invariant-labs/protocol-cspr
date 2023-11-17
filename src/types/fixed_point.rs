use crate::uints::{U128T, U192T};
use decimal::*;

#[decimal(12, U192T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct FixedPoint {
    pub v: U128T,
}
