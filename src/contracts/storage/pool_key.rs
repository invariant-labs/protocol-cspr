use odra::types::Address;
use odra::OdraType;

use super::fee_tier::FeeTier;

#[derive(OdraType)]
pub struct PoolKey {
    pub token_x: Address,
    pub token_y: Address,
    pub fee_tier: FeeTier,
}
