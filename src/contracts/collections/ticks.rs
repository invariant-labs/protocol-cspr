use crate::contracts::{PoolKey, Tick};
use crate::InvariantError;
use odra::Mapping;

#[odra::module]
pub struct Ticks {
    ticks: Mapping<(PoolKey, i32), Option<Tick>>,
}

#[odra::module]
impl Ticks {
    pub fn add(
        &mut self,
        pool_key: PoolKey,
        index: i32,
        tick: &Tick,
    ) -> Result<(), InvariantError> {
        self.get(pool_key, index)
            .map_or(Ok(()), |_| Err(InvariantError::TickAlreadyExist))?;
        self.ticks.set(&(pool_key, index), Some(*tick));
        Ok(())
    }

    pub fn update(
        &mut self,
        pool_key: PoolKey,
        index: i32,
        tick: &Tick,
    ) -> Result<(), InvariantError> {
        self.get(pool_key, index)?;
        self.ticks.set(&(pool_key, index), Some(*tick));
        Ok(())
    }

    pub fn remove(&mut self, pool_key: PoolKey, index: i32) -> Result<(), InvariantError> {
        self.get(pool_key, index)?;
        self.ticks.set(&(pool_key, index), None);
        Ok(())
    }

    pub fn get(&self, pool_key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        let tick = self
            .ticks
            .get(&(pool_key, index))
            .ok_or(InvariantError::TickNotFound)?
            .ok_or(InvariantError::TickNotFound)?;

        Ok(tick)
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
        let ticks = &mut TicksDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick = Tick::default();

        ticks.add(pool_key, 0, &tick).unwrap();
        assert_eq!(ticks.get(pool_key, 0).unwrap(), tick);

        assert_eq!(ticks.get(pool_key, 1), Err(InvariantError::TickNotFound));

        assert_eq!(
            ticks.add(pool_key, 0, &tick),
            Err(InvariantError::TickAlreadyExist)
        );
    }

    #[test]
    fn test_update() {
        let ticks = &mut TicksDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick = Tick::default();
        let new_tick = Tick {
            seconds_outside: 1,
            ..Tick::default()
        };

        ticks.add(pool_key, 0, &tick).unwrap();

        ticks.update(pool_key, 0, &new_tick).unwrap();

        assert_eq!(ticks.get(pool_key, 0).unwrap(), new_tick);

        assert_eq!(
            ticks.update(pool_key, 1, &new_tick),
            Err(InvariantError::TickNotFound)
        );
    }

    #[test]
    fn test_remove() {
        let ticks = &mut TicksDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick = Tick::default();

        ticks.add(pool_key, 0, &tick).unwrap();

        ticks.remove(pool_key, 0).unwrap();

        assert_eq!(ticks.get(pool_key, 0), Err(InvariantError::TickNotFound));

        assert_eq!(ticks.remove(pool_key, 0), Err(InvariantError::TickNotFound));
    }

    #[test]
    fn test_recreation() {
        let ticks = &mut TicksDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick = Tick::default();

        assert_eq!(ticks.get(pool_key, 0), Err(InvariantError::TickNotFound));

        ticks.add(pool_key, 0, &tick).unwrap();
        assert_eq!(ticks.get(pool_key, 0).unwrap(), tick);

        ticks.remove(pool_key, 0).unwrap();
        assert_eq!(ticks.get(pool_key, 0), Err(InvariantError::TickNotFound));

        ticks.add(pool_key, 0, &tick).unwrap();
        assert_eq!(ticks.get(pool_key, 0).unwrap(), tick);
    }
}
