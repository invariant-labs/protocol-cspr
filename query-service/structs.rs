use odra::types::U128;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FeeTierExistParams {
    pub address: String,
    pub fee: U128,
    pub tick_spacing: u32,
}
