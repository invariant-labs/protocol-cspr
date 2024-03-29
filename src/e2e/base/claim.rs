use crate::contracts::InvariantError;
use crate::contracts::PoolKey;
use crate::e2e::snippets::init;
use crate::math::fee_growth::FeeGrowth;
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
fn test_claim() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
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
    {
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000).get();

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price.get();
        let slippage_limit_upper = pool_before.sqrt_price.get();

        invariant
            .create_position(
                *token_x.address(),
                *token_y.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick,
                upper_tick,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(pool_after.liquidity.get(), liquidity)
    }
    // Init basic swap
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(1000);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(500));
        assert_eq!(amount_y, U256::from(1000));

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE)).get();
        let swap_amount = TokenAmount::new(amount).get();
        invariant
            .swap(
                *token_x.address(),
                *token_y.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                true,
                swap_amount,
                true,
                slippage,
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let expected_tick = -20;

        assert_eq!(pool_after.liquidity, pool_before.liquidity);
        assert_eq!(pool_after.current_tick_index, expected_tick);
        assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

        let amount_x = token_x.balance_of(&caller);
        let amount_y = token_y.balance_of(&caller);
        assert_eq!(amount_x, U256::from(0));
        assert_eq!(amount_y, U256::from(993));

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(1500));
        assert_eq!(amount_y, U256::from(7));

        assert_eq!(
            pool_after.fee_growth_global_x,
            FeeGrowth::new(U128::from(50000000000000000000000u128))
        );
        assert_eq!(
            pool_after.fee_growth_global_y,
            FeeGrowth::new(U128::from(0))
        );

        assert_eq!(
            pool_after.fee_protocol_token_x,
            TokenAmount::new(U256::from(1))
        );
        assert_eq!(
            pool_after.fee_protocol_token_y,
            TokenAmount::new(U256::from(0))
        );
    }
    // Claim fee
    {
        let position_owner = test_env::get_account(0);
        test_env::set_caller(position_owner);
        let user_amount_before_claim = token_x.balance_of(&position_owner);
        let dex_amount_before_claim = token_x.balance_of(invariant.address());

        invariant.claim_fee(0).unwrap();

        // Load states
        let pool = invariant
            .get_pool(
                *token_x.address(),
                *token_y.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();
        let position = invariant.get_position(position_owner, 0).unwrap();
        let user_amount_after_claim = token_x.balance_of(&position_owner);
        let dex_amount_after_claim = token_x.balance_of(invariant.address());
        let expected_tokens_claimed = U256::from(5);

        assert_eq!(
            user_amount_after_claim - expected_tokens_claimed,
            user_amount_before_claim
        );
        assert_eq!(
            dex_amount_after_claim + expected_tokens_claimed,
            dex_amount_before_claim
        );
        assert_eq!(position.fee_growth_inside_x, pool.fee_growth_global_x);
        assert_eq!(position.tokens_owed_x, TokenAmount::new(U256::from(0)));
    }
}

#[test]
#[should_panic]
fn test_claim_not_position_owner() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = U256::from(10u128.pow(10));
    let fee = Percentage::from_scale(1, 2);
    let (mut invariant, mut token_x, mut token_y) = init(fee, mint_amount);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
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
    {
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000).get();

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price.get();
        let slippage_limit_upper = pool_before.sqrt_price.get();

        invariant
            .create_position(
                *token_x.address(),
                *token_y.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick,
                upper_tick,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(pool_after.liquidity.get(), liquidity)
    }
    // Init basic swap
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(1000);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(500));
        assert_eq!(amount_y, U256::from(1000));

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE)).get();
        let swap_amount = TokenAmount::new(amount).get();
        invariant
            .swap(
                *token_x.address(),
                *token_y.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                true,
                swap_amount,
                true,
                slippage,
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let expected_tick = -20;

        assert_eq!(pool_after.liquidity, pool_before.liquidity);
        assert_eq!(pool_after.current_tick_index, expected_tick);
        assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

        let amount_x = token_x.balance_of(&caller);
        let amount_y = token_y.balance_of(&caller);
        assert_eq!(amount_x, U256::from(0));
        assert_eq!(amount_y, U256::from(993));

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(1500));
        assert_eq!(amount_y, U256::from(7));

        assert_eq!(
            pool_after.fee_growth_global_x,
            FeeGrowth::new(U128::from(50000000000000000000000u128))
        );
        assert_eq!(
            pool_after.fee_growth_global_y,
            FeeGrowth::new(U128::from(0))
        );

        assert_eq!(
            pool_after.fee_protocol_token_x,
            TokenAmount::new(U256::from(1))
        );
        assert_eq!(
            pool_after.fee_protocol_token_y,
            TokenAmount::new(U256::from(0))
        );
    }
    // Claim fee
    {
        let unauthorized_user = test_env::get_account(1);
        test_env::set_caller(unauthorized_user);
        let result = invariant.claim_fee(0);
        assert_eq!(result, Err(InvariantError::PositionNotFound));
    }
}
