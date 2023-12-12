use super::fee_tier::FeeTier;
use crate::contracts::errors::InvariantError;
use odra::types::casper_types::ContractPackageHash;
use odra::types::Address;
use odra::OdraType;

#[derive(OdraType, Eq, PartialEq, Copy, Debug)]
pub struct PoolKey {
    pub token_x: Address,
    pub token_y: Address,
    pub fee_tier: FeeTier,
}

impl Default for PoolKey {
    fn default() -> Self {
        Self {
            token_x: Address::Contract(ContractPackageHash::from([0x0; 32])),
            token_y: Address::Contract(ContractPackageHash::from([0x0; 32])),
            fee_tier: FeeTier::default(),
        }
    }
}

impl PoolKey {
    pub fn new(
        token_0: Address,
        token_1: Address,
        fee_tier: FeeTier,
    ) -> Result<Self, InvariantError> {
        if token_0 == token_1 {
            return Err(InvariantError::TokensAreSame);
        }

        if token_0 < token_1 {
            Ok(PoolKey {
                token_x: token_0,
                token_y: token_1,
                fee_tier,
            })
        } else {
            Ok(PoolKey {
                token_x: token_1,
                token_y: token_0,
                fee_tier,
            })
        }
    }
}
