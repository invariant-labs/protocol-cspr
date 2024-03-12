use super::fee_tier::FeeTier;
use crate::errors::InvariantError;
use crate::is_token_x;
use odra::OdraType;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;

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

impl PoolKey {
    pub fn new(
        token_0: String,
        token_1: String,
        fee_tier: FeeTier,
    ) -> Result<Self, InvariantError> {
        if token_0 == token_1 {
            return Err(InvariantError::TokensAreSame);
        }

        Ok(if is_token_x(token_0.clone(), token_1.clone()).unwrap() {
            PoolKey {
                token_x: token_0,
                token_y: token_1,
                fee_tier,
            }
        } else {
            PoolKey {
                token_x: token_1,
                token_y: token_0,
                fee_tier,
            }
        })
    }
}

#[wasm_wrapper]
pub fn new_pool_key(
    token_0: String,
    token_1: String,
    fee_tier: FeeTier,
) -> Result<PoolKey, InvariantError> {
    Ok(PoolKey::new(token_0, token_1, fee_tier)?)
}
