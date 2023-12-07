use crate::contracts::{Pool, PoolKey};
use crate::InvariantError;
use odra::Mapping;

#[odra::module]
pub struct Pools {
    pools: Mapping<PoolKey, Option<Pool>>,
}

#[odra::module]
impl Pools {
    pub fn add(&mut self, pool_key: PoolKey, pool: &Pool) -> Result<(), InvariantError> {
        self.get(pool_key)
            .map_or(Ok(()), |_| Err(InvariantError::PoolAlreadyExist))?;
        self.pools.set(&pool_key, Some(*pool));
        Ok(())
    }

    pub fn update(&mut self, pool_key: PoolKey, pool: &Pool) -> Result<(), InvariantError> {
        self.get(pool_key)?;
        self.pools.set(&pool_key, Some(*pool));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
        self.get(pool_key)?;
        self.pools.set(&pool_key, None);
        Ok(())
    }

    pub fn get(&self, pool_key: PoolKey) -> Result<Pool, InvariantError> {
        let pool = self
            .pools
            .get(&pool_key)
            .ok_or(InvariantError::PoolNotFound)?
            .ok_or(InvariantError::PoolNotFound)?;
        Ok(pool)
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

        pools.add(pool_key, &pool).unwrap();

        assert_eq!(pools.get(pool_key).unwrap(), pool);

        assert_eq!(pools.get(new_pool_key), Err(InvariantError::PoolNotFound));

        assert_eq!(
            pools.add(pool_key, &pool),
            Err(InvariantError::PoolAlreadyExist)
        );
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

        pools.add(pool_key, &pool).unwrap();

        pools.update(pool_key, &new_pool).unwrap();
        assert_eq!(pools.get(pool_key).unwrap(), new_pool);

        assert_eq!(
            pools.update(new_pool_key, &new_pool),
            Err(InvariantError::PoolNotFound)
        );
    }

    #[test]
    fn test_remove() {
        let pools = &mut PoolsDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let pool = Pool::default();

        pools.add(pool_key, &pool).unwrap();

        pools.remove(pool_key).unwrap();

        assert_eq!(pools.get(pool_key), Err(InvariantError::PoolNotFound));

        assert_eq!(pools.remove(pool_key), Err(InvariantError::PoolNotFound));
    }
}
