use super::fee_tier::FeeTier;
use odra::OdraType;

use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(OdraType, Eq, PartialEq, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PoolKey {
    pub token_x: String,
    pub token_y: String,
    pub fee_tier: FeeTier,
}

impl Default for PoolKey {
    fn default() -> Self {
        Self {
            token_x: String::from("0"),
            token_y: String::from("0"),
            fee_tier: FeeTier::default(),
        }
    }
}

// impl PoolKey {
//     pub fn new(
//         token_0: Address,
//         token_1: Address,
//         fee_tier: FeeTier,
//     ) -> Result<Self, InvariantError> {
//         if token_0 == token_1 {
//             return Err(InvariantError::TokensAreSame);
//         }

//         if token_0 < token_1 {
//             Ok(PoolKey {
//                 token_x: token_0,
//                 token_y: token_1,
//                 fee_tier,
//             })
//         } else {
//             Ok(PoolKey {
//                 token_x: token_1,
//                 token_y: token_0,
//                 fee_tier,
//             })
//         }
//     }
// }
