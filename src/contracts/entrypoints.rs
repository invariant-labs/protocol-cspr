use super::FeeTier;
use crate::InvariantError;
use odra::prelude::vec::Vec;

pub trait Entrypoints {
    fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;
    fn fee_tier_exist(&self, fee_tier: FeeTier) -> bool;
    fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;
    fn get_fee_tiers(&self) -> Vec<FeeTier>;
}
