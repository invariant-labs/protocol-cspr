use crate::{convert, decimal_ops};
use decimal::*;
use js_sys::BigInt;
use odra::{
    types::{U128, U256},
    OdraType,
};
use wasm_bindgen::prelude::*;

#[decimal(12, U256)]
#[derive(
    Default,
    OdraType,
    Debug,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    tsify::Tsify,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Percentage {
    #[tsify(type = "bigint")]
    pub v: U128,
}

decimal_ops!(Percentage);
