use crate::{convert, decimal_ops};
use decimal::*;
use js_sys::BigInt;
use odra::{
    types::{U256, U512},
    OdraType,
};
use wasm_bindgen::prelude::*;

#[decimal(5, U512)]
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
pub struct Liquidity {
    #[tsify(type = "bigint")]
    pub v: U256,
}

decimal_ops!(Liquidity);
