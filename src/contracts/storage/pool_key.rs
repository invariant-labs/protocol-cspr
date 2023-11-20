use odra::types::Address;
use odra::OdraType;
use crate::contracts::FeeTier;
use odra::types::casper_types::contracts::ContractPackageHash;

#[derive(OdraType)]
pub struct PoolKey {
    pub token_x: Address,
    pub token_y: Address,
    pub fee_tier: FeeTier,
}

impl Default for PoolKey {
    fn default() -> Self {
        Self {
            token_x: Address::Contract(ContractPackageHash::from([0x0;32])),
            token_y: Address::Contract(ContractPackageHash::from([0x0;32])),
            fee_tier: FeeTier::default(),
        }
    }
}