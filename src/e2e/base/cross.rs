use crate::contracts::{CreatePositionEvent, PoolKey};
use crate::contracts::{CrossTickEvent, SwapEvent};
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
use odra::assert_events;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
fn test_cross() {
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
    // Init cross position
    {
        let mint_amount = U256::from(10u128.pow(10));
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -40;
        let upper_tick = -10;
        let liquidity = Liquidity::from_integer(1000000);

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
                liquidity.get(),
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

        assert_eq!(pool_after.liquidity, pool_before.liquidity)
    }
    // init cross swap
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(1000);
        token_x.mint(&caller, &amount);

        let amount_x = token_x.balance_of(&caller);
        assert_eq!(amount_x, U256::from(1000));
        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(500));
        assert_eq!(amount_y, U256::from(2499));

        let pool_before = invariant
            .get_pool(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let sqrt_price_limit = SqrtPrice::new(U128::from(MIN_SQRT_PRICE)).get();
        let swap_amount = TokenAmount::new(amount).get();
        let result = invariant
            .swap(
                *token_x.address(),
                *token_y.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                true,
                swap_amount,
                true,
                sqrt_price_limit,
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

        let position_liquidity = Liquidity::from_integer(1000000);
        let expected_tick = -20;

        assert_eq!(
            pool_after.liquidity - position_liquidity,
            pool_before.liquidity
        );
        assert_eq!(pool_after.current_tick_index, expected_tick);
        assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

        let amount_x = token_x.balance_of(&caller);
        let amount_y = token_y.balance_of(&caller);
        assert_eq!(amount_x, U256::from(0));
        assert_eq!(amount_y, U256::from(990));

        let amount_x = token_x.balance_of(invariant.address());
        let amount_y = token_y.balance_of(invariant.address());
        assert_eq!(amount_x, U256::from(1500));
        assert_eq!(amount_y, U256::from(1509));

        assert_eq!(
            pool_after.fee_growth_global_x,
            FeeGrowth::new(U128::from(40000000000000000000000u128))
        );
        assert_eq!(
            pool_after.fee_growth_global_y,
            FeeGrowth::new(U128::from(0))
        );

        assert_eq!(
            pool_after.fee_protocol_token_x,
            TokenAmount::new(U256::from(2))
        );
        assert_eq!(
            pool_after.fee_protocol_token_y,
            TokenAmount::new(U256::from(0))
        );

        assert_events!(
            invariant,
            CreatePositionEvent {
                timestamp: 0,
                address: deployer,
                pool: pool_key,
                liquidity: Liquidity::from_integer(1000000),
                lower_tick: -20,
                upper_tick: 10,
                current_sqrt_price: calculate_sqrt_price(0).unwrap(),
            },
            CreatePositionEvent {
                timestamp: 0,
                address: deployer,
                pool: pool_key,
                liquidity: Liquidity::from_integer(1000000),
                lower_tick: -40,
                upper_tick: -10,
                current_sqrt_price: calculate_sqrt_price(0).unwrap()
            },
            CrossTickEvent {
                timestamp: 0,
                address: caller,
                pool: pool_key,
                indexes: result.ticks.iter().map(|tick| tick.index).collect(),
            },
            SwapEvent {
                timestamp: 0,
                address: caller,
                pool: pool_key,
                amount_in: TokenAmount::new(U256::from(1000)),
                amount_out: TokenAmount::new(U256::from(990)),
                fee: TokenAmount::new(U256::from(7)),
                start_sqrt_price: pool_before.sqrt_price,
                target_sqrt_price: pool_after.sqrt_price,
                x_to_y: true,
            }
        );
    }
}
