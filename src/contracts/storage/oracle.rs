
use odra::types::U128;
use odra::OdraType;
use odra::prelude::vec::Vec;

#[derive(OdraType)]
pub struct Oracle {
    pub data: Vec<Record>,
    pub head: u16,
    pub amount: u16,
    pub size: u16,
}

#[derive(OdraType)]
pub struct Record {
    pub timestamp: u64,
    pub price: U128,
}