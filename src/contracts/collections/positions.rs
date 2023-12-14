use crate::contracts::errors::InvariantError;
use crate::contracts::Position;
use odra::prelude::vec::Vec;
use odra::types::Address;
use odra::Mapping;

#[odra::module]
pub struct Positions {
    positions_length: Mapping<Address, u32>,
    positions: Mapping<(Address, u32), Option<Position>>,
}

#[odra::module]
impl Positions {
    pub fn add(&mut self, account_id: Address, position: &Position) {
        let positions_length = self.get_length(account_id);

        self.positions
            .set(&(account_id, positions_length), Some(*position));

        self.positions_length.add(&account_id, 1);
    }

    pub fn update(
        &mut self,
        account_id: Address,
        index: u32,
        position: &Position,
    ) -> Result<(), InvariantError> {
        let positions_length = self.get_length(account_id);

        if index >= positions_length {
            return Err(InvariantError::PositionNotFound);
        }

        self.positions.set(&(account_id, index), Some(*position));
        Ok(())
    }

    pub fn remove(&mut self, account_id: Address, index: u32) -> Result<Position, InvariantError> {
        let positions_length = self.get_length(account_id);
        let position = self.get(account_id, index)?;

        if index < positions_length - 1 {
            let last_position = self
                .positions
                .get(&(account_id, positions_length - 1))
                .unwrap();
            self.positions
                .set(&(account_id, positions_length - 1), None);
            self.positions.set(&(account_id, index), last_position);
        } else {
            self.positions.set(&(account_id, index), None);
        }

        self.positions_length.subtract(&account_id, 1);
        Ok(position)
    }

    pub fn transfer(
        &mut self,
        account_id: Address,
        index: u32,
        receiver_account_id: Address,
    ) -> Result<(), InvariantError> {
        let position = self.remove(account_id, index)?;
        self.add(receiver_account_id, &position);
        Ok(())
    }

    pub fn get(&self, account_id: Address, index: u32) -> Result<Position, InvariantError> {
        let position = self
            .positions
            .get(&(account_id, index))
            .ok_or(InvariantError::PositionNotFound)?
            .ok_or(InvariantError::PositionNotFound)?;

        Ok(position)
    }

    pub fn get_all(&self, account_id: Address) -> Vec<Position> {
        (0..self.get_length(account_id))
            .flat_map(|index| {
                self.positions.get(&(account_id, index))
                // .map(|opt_position| opt_position)
            })
            .flatten()
            .collect()
    }

    pub fn get_length(&self, account_id: Address) -> u32 {
        self.positions_length.get(&account_id).unwrap_or(0)
    }
}

#[cfg(all(test, not(feature = "casper")))]
mod tests {
    use super::*;
    use odra::prelude::vec;
    use odra::types::casper_types::account::AccountHash;

    #[test]
    fn test_add() {
        let positions = &mut PositionsDeployer::default();
        let account_id = Address::Account(AccountHash::new([0x01; 32]));
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        positions.add(account_id, &position);
        positions.add(account_id, &new_position);

        assert_eq!(positions.get(account_id, 0).unwrap(), position);
        assert_eq!(positions.get(account_id, 1).unwrap(), new_position);
        assert_eq!(
            positions.get(account_id, 2),
            Err(InvariantError::PositionNotFound)
        );
        assert_eq!(positions.get_length(account_id), 2);
    }

    #[test]
    fn test_update() {
        let positions = &mut PositionsDeployer::default();
        let account_id = Address::Account(AccountHash::new([0x01; 32]));
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        positions.add(account_id, &position);

        positions.update(account_id, 0, &new_position).unwrap();

        assert_eq!(positions.get(account_id, 0).unwrap(), new_position);
        assert_eq!(positions.get_length(account_id), 1);

        assert_eq!(
            positions.update(account_id, 1, &new_position),
            Err(InvariantError::PositionNotFound)
        );
    }

    #[test]
    fn test_remove() {
        let positions = &mut PositionsDeployer::default();
        let account_id = Address::Account(AccountHash::new([0x01; 32]));
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        positions.add(account_id, &position);
        positions.add(account_id, &new_position);

        let result = positions.remove(account_id, 0).unwrap();
        assert_eq!(result, position);
        assert_eq!(positions.get(account_id, 0).unwrap(), new_position);
        assert_eq!(positions.get_length(account_id), 1);

        let result = positions.remove(account_id, 0).unwrap();
        assert_eq!(result, new_position);
        assert_eq!(
            positions.get(account_id, 0),
            Err(InvariantError::PositionNotFound)
        );
        assert_eq!(positions.get_length(account_id), 0);

        assert_eq!(
            positions.remove(account_id, 0),
            Err(InvariantError::PositionNotFound)
        )
    }

    #[test]
    fn test_transfer() {
        let positions = &mut PositionsDeployer::default();
        let account_id = Address::Account(AccountHash::new([0x01; 32]));
        let receiver_account_id = Address::Account(AccountHash::new([0x02; 32]));
        let position = Position::default();

        positions.add(account_id, &position);

        positions
            .transfer(account_id, 0, receiver_account_id)
            .unwrap();

        assert_eq!(
            positions.get(account_id, 0),
            Err(InvariantError::PositionNotFound)
        );
        assert_eq!(positions.get_length(account_id), 0);
        assert_eq!(positions.get(receiver_account_id, 0).unwrap(), position);
        assert_eq!(positions.get_length(receiver_account_id), 1);

        assert_eq!(
            positions.transfer(account_id, 0, receiver_account_id),
            Err(InvariantError::PositionNotFound)
        )
    }

    #[test]
    fn test_get_all() {
        let positions = &mut PositionsDeployer::default();
        let account_id = Address::Account(AccountHash::new([0x01; 32]));
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        let result = positions.get_all(account_id);
        assert_eq!(result, vec![]);
        assert_eq!(result.len(), 0);
        assert_eq!(positions.get_length(account_id), 0);

        positions.add(account_id, &position);
        positions.add(account_id, &new_position);

        let result = positions.get_all(account_id);
        assert_eq!(result, vec![position, new_position]);
        assert_eq!(result.len(), 2);
        assert_eq!(positions.get_length(account_id), 2);
    }

    #[test]
    fn test_get_length() {
        let positions = &mut PositionsDeployer::default();
        let account_id = Address::Account(AccountHash::new([0x01; 32]));
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        let result = positions.get_length(account_id);
        assert_eq!(result, 0);

        positions.add(account_id, &position);
        positions.add(account_id, &new_position);

        let result = positions.get_length(account_id);
        assert_eq!(result, 2);
    }
}
