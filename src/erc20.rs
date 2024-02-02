use odra::prelude::string::String;
use odra::{
    contract_env,
    types::{event::OdraEvent, Address, U256},
    Mapping, UnwrapOrRevert, Variable,
};

use self::{
    errors::Error,
    events::{Approval, Transfer},
};

#[odra::module(events = [Approval, Transfer])]
pub struct Erc20 {
    decimals: Variable<u8>,
    symbol: Variable<String>,
    name: Variable<String>,
    total_supply: Variable<U256>,
    balances: Mapping<Address, U256>,
    allowances: Mapping<Address, Mapping<Address, U256>>,
}

#[odra::module]
impl Erc20 {
    #[odra(init)]
    pub fn init(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: &Option<U256>,
    ) {
        let caller = contract_env::caller();

        self.symbol.set(symbol);
        self.name.set(name);
        self.decimals.set(decimals);

        if let Some(initial_supply) = *initial_supply {
            self.total_supply.set(initial_supply);
            self.balances.set(&caller, initial_supply);

            if !initial_supply.is_zero() {
                Transfer {
                    from: None,
                    to: Some(caller),
                    amount: initial_supply,
                }
                .emit();
            }
        }
    }

    pub fn transfer(&mut self, recipient: &Address, amount: &U256) {
        let caller = contract_env::caller();
        self.raw_transfer(&caller, recipient, amount);
    }

    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        let spender = contract_env::caller();

        self.spend_allowance(owner, &spender, amount);
        self.raw_transfer(owner, recipient, amount);
    }

    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        let owner = contract_env::caller();

        self.allowances.get_instance(&owner).set(spender, *amount);
        Approval {
            owner,
            spender: *spender,
            value: *amount,
        }
        .emit();
    }

    pub fn name(&self) -> String {
        self.name.get().unwrap_or_revert_with(Error::NameNotSet)
    }

    pub fn symbol(&self) -> String {
        self.symbol.get().unwrap_or_revert_with(Error::SymbolNotSet)
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
            .get()
            .unwrap_or_revert_with(Error::DecimalsNotSet)
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    pub fn balance_of(&self, address: &Address) -> U256 {
        self.balances.get_or_default(address)
    }

    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.allowances.get_instance(owner).get_or_default(spender)
    }

    pub fn mint(&mut self, address: &Address, amount: &U256) {
        self.total_supply.add(*amount);
        self.balances.add(address, *amount);

        Transfer {
            from: None,
            to: Some(*address),
            amount: *amount,
        }
        .emit();
    }

    pub fn burn(&mut self, address: &Address, amount: &U256) {
        if self.balance_of(address) < *amount {
            contract_env::revert(Error::InsufficientBalance);
        }
        self.total_supply.subtract(*amount);
        self.balances.subtract(address, *amount);

        Transfer {
            from: Some(*address),
            to: None,
            amount: *amount,
        }
        .emit();
    }
}

impl Erc20 {
    fn raw_transfer(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        if *amount > self.balances.get_or_default(owner) {
            contract_env::revert(Error::InsufficientBalance)
        }

        self.balances.subtract(owner, *amount);
        self.balances.add(recipient, *amount);

        Transfer {
            from: Some(*owner),
            to: Some(*recipient),
            amount: *amount,
        }
        .emit();
    }

    fn spend_allowance(&mut self, owner: &Address, spender: &Address, amount: &U256) {
        let allowance = self.allowances.get_instance(owner).get_or_default(spender);
        if allowance < *amount {
            contract_env::revert(Error::InsufficientAllowance)
        }
        self.allowances
            .get_instance(owner)
            .subtract(spender, *amount);
        Approval {
            owner: *owner,
            spender: *spender,
            value: allowance - *amount,
        }
        .emit();
    }
}

pub mod events {
    use odra::types::{casper_types::U256, Address};
    use odra::Event;

    #[derive(Event, Eq, PartialEq, Debug)]
    pub struct Transfer {
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub amount: U256,
    }

    #[derive(Event, Eq, PartialEq, Debug)]
    pub struct Approval {
        pub owner: Address,
        pub spender: Address,
        pub value: U256,
    }
}

pub mod errors {
    use odra::execution_error;

    execution_error! {
        pub enum Error {
            InsufficientBalance => 30_000,
            InsufficientAllowance => 30_001,
            NameNotSet => 30_002,
            SymbolNotSet => 30_003,
            DecimalsNotSet => 30_004,
        }
    }
}
