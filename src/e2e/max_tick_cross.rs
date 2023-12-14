use crate::contracts::PoolKey;
use crate::e2e::snippets::init;
use crate::math::get_tick_at_sqrt_price;
use crate::math::liquidity::Liquidity;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::math::sqrt_price::SqrtPrice;
use crate::math::token_amount::TokenAmount;
use crate::math::MIN_SQRT_PRICE;
use crate::FeeTier;
use decimal::*;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
fn test_max_tick_cross() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::MAX;
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant.add_fee_tier(fee_tier).unwrap();
        let exist = invariant.fee_tier_exist(fee_tier);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
            )
            .unwrap();
    }
    // Init positions
    {
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let liquidity = Liquidity::from_integer(10000000);

        for i in (-2500..20).step_by(10) {
            let pool = invariant
                .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
                .unwrap();

            let slippage_limit_lower = pool.sqrt_price;
            let slippage_limit_upper = pool.sqrt_price;

            invariant
                .create_position(
                    pool_key,
                    i,
                    i + 10,
                    liquidity,
                    slippage_limit_lower,
                    slippage_limit_upper,
                )
                .unwrap();
        }

        let pool = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();
        assert_eq!(pool.liquidity, liquidity)
    }
    // Init swap
    {
        // 1.1625m - 218 - -2190
        let amount = U256::from(1_162_500);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        let quote_result = invariant
            .quote(pool_key, true, swap_amount, true, slippage)
            .unwrap();

        let pool_after_quote = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let crosses_after_quote =
            ((pool_after_quote.current_tick_index - pool_before.current_tick_index) / 10).abs();
        assert_eq!(crosses_after_quote, 0);
        assert_eq!(quote_result.ticks.len() - 1, 218);

        let result = invariant
            .swap(pool_key, true, swap_amount, true, slippage)
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let crosses = ((pool_after.current_tick_index - pool_before.current_tick_index) / 10).abs();
        assert_eq!(result.ticks.len() - 1, 218);
        assert_eq!(crosses - 1, 218);
        assert_eq!(
            pool_after.current_tick_index,
            get_tick_at_sqrt_price(quote_result.target_sqrt_price, 10).unwrap()
        );
    }
}
