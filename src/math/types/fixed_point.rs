use crate::math::uints::U192T;
use decimal::*;
use odra::{types::U128, OdraType};

#[decimal(12, U192T)]
#[derive(OdraType, Default, Debug, Copy, PartialEq, Eq, PartialOrd)]
pub struct FixedPoint {
    pub v: U128,
}
