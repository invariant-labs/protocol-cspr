use crate::contracts::InvariantExecutionError;
use crate::contracts::{PoolKey, Tick};
use odra::contract_env;
use odra::Mapping;
use odra::UnwrapOrRevert;

#[odra::module]
pub struct Ticks {
    ticks: Mapping<(PoolKey, i32), Option<Tick>>,
}

#[odra::module]
impl Ticks {
    pub fn add(&mut self, pool_key: PoolKey, index: i32, tick: &Tick) {
        self.ticks.get(&(pool_key, index)).map_or((), |_| {
            contract_env::revert(InvariantExecutionError::TickAlreadyExist)
        });

        self.ticks.set(&(pool_key, index), Some(*tick));
    }

    pub fn update(&mut self, pool_key: PoolKey, index: i32, tick: &Tick) {
        self.get(pool_key, index);

        self.ticks.set(&(pool_key, index), Some(*tick));
    }

    pub fn remove(&mut self, pool_key: PoolKey, index: i32) {
        self.get(pool_key, index);

        self.ticks.set(&(pool_key, index), None);
    }

    pub fn get(&self, pool_key: PoolKey, index: i32) -> Tick {
        self.ticks
            .get(&(pool_key, index))
            .unwrap_or_revert_with(InvariantExecutionError::TickNotFound)
            .unwrap_or_revert_with(InvariantExecutionError::TickNotFound)
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
        let fee_tier = FeeTier {
            fee: Percentage::new(U128::from(0)),
            tick_spacing: 1,
        };
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick = Tick::default();

        ticks.add(pool_key, 0, &tick);
        assert_eq!(ticks.get(pool_key, 0), tick);

        odra::test_env::assert_exception(InvariantExecutionError::TickNotFound, || {
            ticks.get(pool_key, 1);
        });

        odra::test_env::assert_exception(InvariantExecutionError::TickAlreadyExist, || {
            ticks.add(pool_key, 0, &tick);
        })
    }

    #[test]
    fn test_update() {
        let ticks = &mut TicksDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier {
            fee: Percentage::new(U128::from(0)),
            tick_spacing: 1,
        };

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick = Tick::default();
        let new_tick = Tick {
            seconds_outside: 1,
            ..Tick::default()
        };

        ticks.add(pool_key, 0, &tick);

        ticks.update(pool_key, 0, &new_tick);

        assert_eq!(ticks.get(pool_key, 0), new_tick);

        odra::test_env::assert_exception(InvariantExecutionError::TickNotFound, || {
            ticks.update(pool_key, 1, &new_tick);
        });
    }

    #[test]
    fn test_remove() {
        let ticks = &mut TicksDeployer::default();
        let token_x = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_y = Address::Contract(ContractPackageHash::from([0x02; 32]));
        let fee_tier = FeeTier {
            fee: Percentage::new(U128::from(0)),
            tick_spacing: 1,
        };

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let tick = Tick::default();

        ticks.add(pool_key, 0, &tick);

        ticks.remove(pool_key, 0);

        odra::test_env::assert_exception(InvariantExecutionError::TickNotFound, || {
            ticks.get(pool_key, 0);
        });

        odra::test_env::assert_exception(InvariantExecutionError::TickNotFound, || {
            ticks.remove(pool_key, 0);
        });
    }
}
