use odra::types::Address;
use odra::OdraType;

use crate::ContractErrors;

use super::fee_tier::FeeTier;

#[derive(OdraType)]
pub struct PoolKey {
    pub token_x: Address,
    pub token_y: Address,
    pub fee_tier: FeeTier,
}

impl PoolKey {
    pub fn new(
        token_0: Address,
        token_1: Address,
        fee_tier: FeeTier,
    ) -> Result<Self, ContractErrors> {
        if token_0 == token_1 {
            return Err(ContractErrors::TokensAreTheSame);
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
