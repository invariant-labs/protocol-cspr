#[cfg(test)]
mod tests {
    use crate::contracts::PoolKey;
    use crate::math::liquidity::Liquidity;
    use crate::math::sqrt_price::{calculate_sqrt_price, SqrtPrice};
    use crate::token::TokenDeployer;
    use crate::{contracts::FeeTier, math::percentage::Percentage, InvariantDeployer};
    use decimal::Decimal;
    use odra::types::U256;
    use odra::{prelude::string::String, test_env, types::U128};

    #[test]
    fn test_create_position() {
        let alice = test_env::get_account(0);
        test_env::set_caller(alice);

        let mut token_x =
            TokenDeployer::init(String::from(""), String::from(""), 0, &U256::from(500));
        let mut token_y =
            TokenDeployer::init(String::from(""), String::from(""), 0, &U256::from(500));
        let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

        let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();
        invariant.add_fee_tier(fee_tier).unwrap();

        let init_tick = 10;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                *token_x.address(),
                *token_y.address(),
                fee_tier,
                init_sqrt_price,
                init_tick,
            )
            .unwrap();

        token_x.approve(invariant.address(), &U256::from(500));
        token_y.approve(invariant.address(), &U256::from(500));

        let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();

        invariant
            .create_position(
                pool_key,
                -10,
                10,
                Liquidity::new(U256::from(10)),
                SqrtPrice::new(U128::from(0)),
                SqrtPrice::max_instance(),
            )
            .unwrap();
    }
}
