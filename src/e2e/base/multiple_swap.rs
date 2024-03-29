use crate::contracts::logic::get_liquidity;
use crate::contracts::PoolKey;
use crate::e2e::snippets::init;
use crate::math::fee_growth::FeeGrowth;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::math::sqrt_price::SqrtPrice;
use crate::math::token_amount::TokenAmount;
use crate::math::MAX_SQRT_PRICE;
use crate::math::MIN_SQRT_PRICE;
use crate::FeeTier;
use decimal::*;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
fn test_multiple_swap_x_to_y() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);
    let fee_tier = FeeTier::new(Percentage::from_scale(1, 3), 1).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant
            .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
            .unwrap();

        let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Init basic position

    let mint_amount = U256::from(10u128.pow(10));
    token_x.mint(&deployer, &mint_amount);
    token_y.mint(&deployer, &mint_amount);
    token_x.approve(invariant.address(), &mint_amount);
    token_y.approve(invariant.address(), &mint_amount);

    let upper_tick = 953;
    let lower_tick = -953;

    let amount = 100;

    let pool = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();

    let liquidity = get_liquidity(
        TokenAmount::new(U256::from(amount)),
        TokenAmount::new(U256::from(amount)),
        lower_tick,
        upper_tick,
        pool.sqrt_price,
        true,
    )
    .unwrap()
    .l;

    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;

    invariant
        .create_position(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            lower_tick,
            upper_tick,
            liquidity.get(),
            slippage_limit_lower.get(),
            slippage_limit_upper.get(),
        )
        .unwrap();

    // Init swaps
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(100);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let swap_amount = TokenAmount::new(U256::from(10));

        let sqrt_price_limit = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        for _ in 1..=10 {
            let quoted_target_sqrt_price = invariant
                .quote(
                    pool_key.token_x,
                    pool_key.token_y,
                    fee_tier.fee.get(),
                    fee_tier.tick_spacing,
                    true,
                    swap_amount.get(),
                    true,
                    sqrt_price_limit.get(),
                )
                .unwrap()
                .target_sqrt_price;

            invariant
                .swap(
                    pool_key.token_x,
                    pool_key.token_y,
                    fee_tier.fee.get(),
                    fee_tier.tick_spacing,
                    true,
                    swap_amount.get(),
                    true,
                    quoted_target_sqrt_price.get(),
                )
                .unwrap();
        }
    }
    // Check states
    {
        let caller = test_env::get_account(1);
        let pool = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();
        let dex_amount_x = token_x.balance_of(invariant.address());
        let dex_amount_y = token_y.balance_of(invariant.address());
        let user_amount_x = token_x.balance_of(&caller);
        let user_amount_y = token_y.balance_of(&caller);

        assert_eq!(pool.fee_growth_global_x, FeeGrowth::new(U128::from(0)));
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(U128::from(0)));
        assert_eq!(pool.liquidity, liquidity);
        assert_eq!(pool.current_tick_index, -821);
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(10)));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(0)));
        assert_eq!(
            pool.sqrt_price,
            SqrtPrice::new(U128::from(959805958530842759275220u128))
        );
        assert_eq!(dex_amount_x, U256::from(200));
        assert_eq!(dex_amount_y, U256::from(20));
        assert_eq!(user_amount_x, U256::from(0));
        assert_eq!(user_amount_y, U256::from(80));
    }
}

#[test]
fn test_multiple_swap_y_to_x() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);
    let fee_tier = FeeTier::new(Percentage::from_scale(1, 3), 1).unwrap();
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    // Init basic pool
    {
        invariant
            .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
            .unwrap();

        let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Init basic position

    let mint_amount = U256::from(10u128.pow(10));
    token_x.mint(&deployer, &mint_amount);
    token_y.mint(&deployer, &mint_amount);
    token_x.approve(invariant.address(), &mint_amount);
    token_y.approve(invariant.address(), &mint_amount);

    let upper_tick = 953;
    let lower_tick = -953;

    let amount = 100;

    let pool = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();

    let liquidity = get_liquidity(
        TokenAmount::new(U256::from(amount)),
        TokenAmount::new(U256::from(amount)),
        lower_tick,
        upper_tick,
        pool.sqrt_price,
        true,
    )
    .unwrap()
    .l;

    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;

    invariant
        .create_position(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
            lower_tick,
            upper_tick,
            liquidity.get(),
            slippage_limit_lower.get(),
            slippage_limit_upper.get(),
        )
        .unwrap();

    // Init swaps
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(100);
        token_y.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_y.approve(invariant.address(), &amount);

        let swap_amount = TokenAmount::new(U256::from(10));

        let sqrt_price_limit = SqrtPrice::new(U128::from(MAX_SQRT_PRICE));
        for _ in 1..=10 {
            let quoted_target_sqrt_price = invariant
                .quote(
                    pool_key.token_x,
                    pool_key.token_y,
                    fee_tier.fee.get(),
                    fee_tier.tick_spacing,
                    false,
                    swap_amount.get(),
                    true,
                    sqrt_price_limit.get(),
                )
                .unwrap()
                .target_sqrt_price;

            invariant
                .swap(
                    pool_key.token_x,
                    pool_key.token_y,
                    fee_tier.fee.get(),
                    fee_tier.tick_spacing,
                    false,
                    swap_amount.get(),
                    true,
                    quoted_target_sqrt_price.get(),
                )
                .unwrap();
        }
    }
    // Check states
    {
        let caller = test_env::get_account(1);
        let pool = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();
        let dex_amount_x = token_x.balance_of(invariant.address());
        let dex_amount_y = token_y.balance_of(invariant.address());
        let user_amount_x = token_x.balance_of(&caller);
        let user_amount_y = token_y.balance_of(&caller);

        assert_eq!(pool.fee_growth_global_x, FeeGrowth::new(U128::from(0)));
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(U128::from(0)));
        assert_eq!(pool.liquidity, liquidity);
        assert_eq!(pool.current_tick_index, 820);
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(0)));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(10)));
        assert_eq!(
            pool.sqrt_price,
            SqrtPrice::new(U128::from(1041877257701839564633600u128))
        );
        assert_eq!(dex_amount_x, U256::from(20));
        assert_eq!(dex_amount_y, U256::from(200));
        assert_eq!(user_amount_x, U256::from(80));
        assert_eq!(user_amount_y, U256::from(0));
    }
}
