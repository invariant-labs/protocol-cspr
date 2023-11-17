extern crate alloc;

pub mod consts;
pub mod math;
pub mod types;
pub mod uints;

#[odra::module]
pub struct Invariant {

}

#[odra::module]
impl Invariant {
    #[odra(init)]
    pub fn init(&mut self) {}
}

#[cfg(test)]
mod tests {
    use odra::test_env;

    use super::*;
    #[test]

    fn init_invariant() {
        let deployer = test_env::get_account(0);
        test_env::set_caller(deployer);
        let _invariant = InvariantDeployer::init();
    }
}