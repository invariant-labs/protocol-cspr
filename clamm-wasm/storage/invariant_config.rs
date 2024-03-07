use crate::percentage::Percentage;
use odra::OdraType;

use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(OdraType, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct InvariantConfig {
    pub admin: String,
    pub protocol_fee: Percentage,
}
