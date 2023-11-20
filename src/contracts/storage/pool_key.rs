use odra::types::Address;
use odra::types::{U128, U256};
use odra::OdraType;
use crate::contracts::FeeTier;

#[derive(OdraType)]
pub struct PoolKey {
    pub token_x: Address,
    pub token_y: Address,
    pub fee_tier: FeeTier,
}
