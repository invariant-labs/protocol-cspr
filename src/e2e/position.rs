#[cfg(test)]
mod tests {
    use crate::contracts::PoolKey;
    use crate::math::fee_growth::FeeGrowth;
    use crate::math::liquidity::Liquidity;
    use crate::math::sqrt_price::SqrtPrice;
    use crate::math::token_amount::TokenAmount;
    use crate::math::MIN_SQRT_PRICE;
    use crate::token::TokenDeployer;
    use crate::{contracts::FeeTier, math::percentage::Percentage, InvariantDeployer};
    use decimal::{Decimal, Factories};
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

        invariant
            .create_pool(*token_x.address(), *token_y.address(), fee_tier, 10)
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

    #[test]
    fn test_remove_position() {
        let alice = test_env::get_account(0);
        let bob = test_env::get_account(1);
        test_env::set_caller(alice);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

        let init_tick = 0;
        let remove_position_index = 0;

        let initial_mint = 10u128.pow(10);

        let mut token_x = TokenDeployer::init(
            String::from(""),
            String::from(""),
            0,
            &U256::from(initial_mint),
        );
        let mut token_y = TokenDeployer::init(
            String::from(""),
            String::from(""),
            0,
            &U256::from(initial_mint),
        );
        let mut invariant = InvariantDeployer::init(Percentage::from_scale(1, 2));

        let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();

        invariant.add_fee_tier(fee_tier).unwrap();

        invariant
            .create_pool(*token_x.address(), *token_y.address(), fee_tier, init_tick)
            .unwrap();

        let lower_tick_index = -20;
        let upper_tick_index = 10;
        let liquidity_delta = Liquidity::from_integer(1_000_000);

        token_x.approve(invariant.address(), &U256::from(initial_mint));
        token_y.approve(invariant.address(), &U256::from(initial_mint));

        let pool_state = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();

        invariant
            .create_position(
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
            )
            .unwrap();

        let pool_state = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();

        assert_eq!(pool_state.liquidity, liquidity_delta);

        let liquidity_delta = Liquidity::new(liquidity_delta.get() * 1_000_000);
        {
            let incorrect_lower_tick_index = lower_tick_index - 50;
            let incorrect_upper_tick_index = upper_tick_index + 50;

            token_x.approve(invariant.address(), &U256::from(liquidity_delta.get()));
            token_y.approve(invariant.address(), &U256::from(liquidity_delta.get()));

            invariant
                .create_position(
                    pool_key,
                    incorrect_lower_tick_index,
                    incorrect_upper_tick_index,
                    liquidity_delta,
                    pool_state.sqrt_price,
                    pool_state.sqrt_price,
                )
                .unwrap();

            let position_state = invariant.get_position(1).unwrap();
            // Check position
            assert!(position_state.lower_tick_index == incorrect_lower_tick_index);
            assert!(position_state.upper_tick_index == incorrect_upper_tick_index);
        }

        let amount = 1000;
        token_x.mint(&bob, &U256::from(amount));
        let amount_x = token_x.balance_of(&bob);
        assert_eq!(amount_x, U256::from(amount));

        token_x.approve(invariant.address(), &U256::from(amount));

        let pool_state_before = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();

        let swap_amount = TokenAmount::new(U256::from(amount));
        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
    }

    #[test]
    fn test_position_within_current_tick() {
        let alice = test_env::get_account(0);
        test_env::set_caller(alice);

        let max_tick_test = 177_450; // for tickSpacing 4
        let min_tick_test = -max_tick_test;
        let init_tick = -23028;

        let initial_balance = 100_000_000;
        let mut token_x = TokenDeployer::init(
            String::from(""),
            String::from(""),
            0,
            &U256::from(initial_balance),
        );
        let mut token_y = TokenDeployer::init(
            String::from(""),
            String::from(""),
            0,
            &U256::from(initial_balance),
        );
        let mut invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

        invariant.add_fee_tier(fee_tier).unwrap();

        invariant
            .create_pool(*token_x.address(), *token_y.address(), fee_tier, init_tick)
            .unwrap();

        token_x.approve(invariant.address(), &U256::from(initial_balance));
        token_y.approve(invariant.address(), &U256::from(initial_balance));

        let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
        let lower_tick_index = min_tick_test + 10;
        let upper_tick_index = max_tick_test - 10;

        let liquidity_delta = Liquidity::new(U256::from(initial_balance));

        let pool_state = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();

        invariant
            .create_position(
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
            )
            .unwrap();

        // Load states
        let position_state = invariant.get_position(0).unwrap();
        let pool_state = invariant
            .get_pool(*token_x.address(), *token_y.address(), fee_tier)
            .unwrap();
        let lower_tick = invariant.get_tick(pool_key, lower_tick_index).unwrap();
        let upper_tick = invariant.get_tick(pool_key, upper_tick_index).unwrap();
        let alice_x = token_x.balance_of(&alice);
        let alice_y = token_y.balance_of(&alice);
        let dex_x = token_x.balance_of(invariant.address());
        let dex_y = token_y.balance_of(invariant.address());

        let zero_fee = FeeGrowth::new(U128::from(0));
        let expected_x_increase = 317;
        let expected_y_increase = 32;

        // Check ticks
        assert!(lower_tick.index == lower_tick_index);
        assert!(upper_tick.index == upper_tick_index);
        assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
        assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
        assert_eq!(lower_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.liquidity_change, liquidity_delta);
        assert!(lower_tick.sign);
        assert!(!upper_tick.sign);

        // Check pool
        assert!(pool_state.liquidity == liquidity_delta);
        assert!(pool_state.current_tick_index == init_tick);

        // Check position
        assert!(position_state.pool_key == pool_key);
        assert!(position_state.liquidity == liquidity_delta);
        assert!(position_state.lower_tick_index == lower_tick_index);
        assert!(position_state.upper_tick_index == upper_tick_index);
        assert!(position_state.fee_growth_inside_x == zero_fee);
        assert!(position_state.fee_growth_inside_y == zero_fee);

        // Check balances
        assert_eq!(
            alice_x,
            U256::from(initial_balance).checked_sub(dex_x).unwrap()
        );
        assert_eq!(
            alice_y,
            U256::from(initial_balance).checked_sub(dex_y).unwrap()
        );
        assert_eq!(dex_x, U256::from(expected_x_increase));
        assert_eq!(dex_y, U256::from(expected_y_increase));
    }
}
