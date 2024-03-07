use crate::uints::U192T;
use crate::{convert, decimal_ops};
use decimal::*;
use js_sys::BigInt;
use odra::{types::U128, OdraType};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
#[decimal(12, U192T)]
#[derive(
    OdraType, Default, Debug, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FixedPoint {
    #[tsify(type = "bigint")]
    pub v: U128,
}

decimal_ops!(FixedPoint);
