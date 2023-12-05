use crate::contracts::FeeTier;
use crate::ContractErrors;
use odra::prelude::vec::Vec;
use odra::OdraType;

#[derive(OdraType, Default)]
pub struct FeeTiers {
    pub fee_tiers: Vec<FeeTier>,
}

impl FeeTiers {
    pub fn add(&mut self, fee_tier: FeeTier) -> Result<(), ContractErrors> {
        if self.contains(fee_tier) {
            return Err(ContractErrors::FeeTierAlreadyExist);
        }

        self.fee_tiers.push(fee_tier);
        Ok(())
    }

    pub fn remove(&mut self, fee_tier: FeeTier) -> Result<(), ContractErrors> {
        let index = self
            .fee_tiers
            .iter()
            .position(|vec_fee_tier| *vec_fee_tier == fee_tier)
            .ok_or(ContractErrors::FeeTierNotFound)?;

        self.fee_tiers.remove(index);
        Ok(())
    }

    pub fn contains(&self, fee_tier: FeeTier) -> bool {
        self.fee_tiers.contains(&fee_tier)
    }

    pub fn get_all(&self) -> Vec<FeeTier> {
        self.fee_tiers.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::percentage::Percentage;
    use decimal::*;
    use odra::prelude::vec;
    use odra::types::U128;

    #[test]
    fn test_add() {
        let fee_tier_keys = &mut FeeTiers::default();
        let fee_tier_key = FeeTier::default();
        let new_fee_tier_key = FeeTier::new(Percentage::new(U128::from(0)), 2).unwrap();

        fee_tier_keys.add(fee_tier_key).unwrap();
        assert!(fee_tier_keys.contains(fee_tier_key));
        assert!(!fee_tier_keys.contains(new_fee_tier_key));

        let result = fee_tier_keys.add(fee_tier_key);
        assert_eq!(result, Err(ContractErrors::FeeTierAlreadyExist));
    }

    #[test]
    fn test_remove() {
        let fee_tier_keys = &mut FeeTiers::default();
        let fee_tier_key = FeeTier::default();

        fee_tier_keys.add(fee_tier_key).unwrap();

        fee_tier_keys.remove(fee_tier_key).unwrap();
        assert!(!fee_tier_keys.contains(fee_tier_key));

        let result = fee_tier_keys.remove(fee_tier_key);
        assert_eq!(result, Err(ContractErrors::FeeTierNotFound));
    }

    #[test]
    fn test_get_all() {
        let fee_tier_keys = &mut FeeTiers::default();
        let fee_tier_key = FeeTier::default();
        let new_fee_tier_key = FeeTier::new(Percentage::new(U128::from(0)), 2).unwrap();

        let result = fee_tier_keys.get_all();
        assert_eq!(result, vec![]);
        assert_eq!(result.len(), 0);

        fee_tier_keys.add(fee_tier_key).unwrap();
        fee_tier_keys.add(new_fee_tier_key).unwrap();

        let result = fee_tier_keys.get_all();
        assert_eq!(result, vec![fee_tier_key, new_fee_tier_key]);
        assert_eq!(result.len(), 2);
    }
}
