use super::fee_tier::FeeTier;

pub trait InvariantTrait {
    fn set_caller(&self, fee_tier: FeeTier);
}
