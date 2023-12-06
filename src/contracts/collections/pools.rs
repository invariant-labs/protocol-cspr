use crate::contracts::InvariantExecutionError;
use crate::contracts::{Pool, PoolKey};
use odra::contract_env;
use odra::Mapping;
use odra::UnwrapOrRevert;

#[odra::module]
pub struct Pools {
    pools: Mapping<PoolKey, Option<Pool>>,
}

#[odra::module]
impl Pools {
    pub fn add(&mut self, pool_key: PoolKey, pool: &Pool) {
        if self
            .pools
            .get(&pool_key)
            .map_or(true, |pool_opt| pool_opt.is_none())
        {
            self.pools.set(&pool_key, Some(*pool));
        } else {
            contract_env::revert(InvariantExecutionError::PoolAlreadyExist);
        }
    }

    pub fn update(&mut self, pool_key: PoolKey, pool: &Pool) {
        self.get(pool_key)
            .unwrap_or_revert_with(InvariantExecutionError::PoolNotFound);

        self.pools.set(&pool_key, Some(*pool));
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, pool_key: PoolKey) {
        self.get(pool_key)
            .unwrap_or_revert_with(InvariantExecutionError::PoolNotFound);

        self.pools.set(&pool_key, None);
    }

    pub fn get(&self, pool_key: PoolKey) -> Option<Pool> {
        match self.pools.get(&pool_key) {
            Some(p) => p,
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{contracts::FeeTier, math::percentage::Percentage};
    use decimal::*;
    use odra::types::casper_types::ContractPackageHash;
    use odra::types::Address;
    use odra::types::U128;

    #[test]
    fn test_add() {
        let pools = &mut PoolsDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        let new_fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 2).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let new_pool_key = PoolKey::new(token_x, token_y, new_fee_tier).unwrap();
        let pool = Pool::default();

        pools.add(pool_key, &pool);

        assert_eq!(pools.get(pool_key).unwrap(), pool);

        assert_eq!(pools.get(new_pool_key), None);

        odra::test_env::assert_exception(InvariantExecutionError::PoolAlreadyExist, || {
            pools.add(pool_key, &pool);
        });
    }

    #[test]
    fn test_update() {
        let pools = &mut PoolsDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        let new_fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 2).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let new_pool_key = PoolKey::new(token_x, token_y, new_fee_tier).unwrap();
        let pool = Pool::default();
        let new_pool = Pool {
            current_tick_index: 1,
            ..Pool::default()
        };

        pools.add(pool_key, &pool);

        pools.update(pool_key, &new_pool);
        assert_eq!(pools.get(pool_key).unwrap(), new_pool);

        odra::test_env::assert_exception(InvariantExecutionError::PoolNotFound, || {
            pools.update(new_pool_key, &new_pool);
        });
    }

    #[test]
    fn test_remove() {
        let pools = &mut PoolsDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let pool = Pool::default();

        pools.add(pool_key, &pool);

        pools.remove(pool_key);

        assert_eq!(pools.get(pool_key), None);

        odra::test_env::assert_exception(InvariantExecutionError::PoolNotFound, || {
            pools.remove(pool_key);
        });
    }
}
