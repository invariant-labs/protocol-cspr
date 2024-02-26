use crate::contracts::PoolKey;
use crate::math::liquidity::Liquidity;
use crate::math::percentage::Percentage;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::math::token_amount::TokenAmount;
use crate::FeeTier;
use crate::SwapHop;
use crate::{Erc20Deployer, InvariantDeployer};
use alloc::string::String;
use alloc::vec;
use decimal::*;
use odra::test_env;
use odra::types::{U128, U256};

#[test]
fn test_swap_route() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);
    // Init basic dex and tokens
    let mint_amount = Some(U256::from(10u128.pow(10)));
    let fee = Percentage::from_scale(1, 2);
    let mut invariant = InvariantDeployer::init(fee.get());
    let token_0 = Erc20Deployer::init(String::from(""), String::from(""), 0, &mint_amount);
    let token_1 = Erc20Deployer::init(String::from(""), String::from(""), 0, &mint_amount);
    let token_2 = Erc20Deployer::init(String::from(""), String::from(""), 0, &mint_amount);

    let mut token_vector = [token_0, token_1, token_2];
    token_vector.sort_by(|a, b| a.address().cmp(b.address()));
    let [mut token_x, mut token_y, mut token_z] = token_vector;

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();
    let pool_key_xy = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let pool_key_yz = PoolKey::new(*token_y.address(), *token_z.address(), fee_tier).unwrap();

    // Add fee tier
    {
        invariant
            .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
            .unwrap();

        let exist = invariant.fee_tier_exist(fee_tier.fee.get(), fee_tier.tick_spacing);
        assert!(exist);
    }
    // Init x to y pool
    {
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key_xy.token_x,
                pool_key_xy.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Init y to z pool
    {
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        invariant
            .create_pool(
                pool_key_yz.token_x,
                pool_key_yz.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                init_sqrt_price.get(),
                init_tick,
            )
            .unwrap();
    }
    // Open positions on both pools
    {
        let amount = U256::from(2u128.pow(127));
        token_x.mint(&deployer, &amount);
        token_y.mint(&deployer, &amount);
        token_z.mint(&deployer, &amount);

        token_x.approve(invariant.address(), &amount);
        token_y.approve(invariant.address(), &amount);
        token_z.approve(invariant.address(), &amount);

        let liquidity_delta = Liquidity::new(U256::from(2u128.pow(63) - 1));
        let lower_tick = -1;
        let upper_tick = 1;
        let pool = invariant
            .get_pool(
                *token_x.address(),
                *token_y.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        let slippage_limit = pool.sqrt_price;
        invariant
            .create_position(
                pool_key_xy.token_x,
                pool_key_xy.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick,
                upper_tick,
                liquidity_delta.get(),
                slippage_limit.get(),
                slippage_limit.get(),
            )
            .unwrap();
        invariant
            .create_position(
                pool_key_yz.token_x,
                pool_key_yz.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                lower_tick,
                upper_tick,
                liquidity_delta.get(),
                slippage_limit.get(),
                slippage_limit.get(),
            )
            .unwrap();
    }
    // Perform swaps
    {
        let amount_in = TokenAmount::new(U256::from(1000));
        let swapper = test_env::get_account(1);
        token_x.mint(&swapper, &amount_in.get());

        test_env::set_caller(swapper);
        token_x.approve(invariant.address(), &amount_in.get());
        token_y.approve(invariant.address(), &amount_in.get());

        let slippage = Percentage::new(U128::from(0));
        let swaps = vec![
            SwapHop {
                token_x: pool_key_xy.token_x,
                token_y: pool_key_xy.token_y,
                fee: fee_tier.fee.get(),
                tick_spacing: fee_tier.tick_spacing,
                x_to_y: true,
            },
            SwapHop {
                token_x: pool_key_yz.token_x,
                token_y: pool_key_yz.token_y,
                fee: fee_tier.fee.get(),
                tick_spacing: fee_tier.tick_spacing,
                x_to_y: true,
            },
        ];

        let expected_token_amount = invariant
            .quote_route(amount_in.get(), swaps.clone())
            .unwrap();
        invariant
            .swap_route(
                amount_in.get(),
                expected_token_amount.get(),
                slippage.get(),
                swaps,
            )
            .unwrap();

        // Check states
        let swapper_x = token_x.balance_of(&swapper);
        let swapper_y = token_y.balance_of(&swapper);
        let swapper_z = token_z.balance_of(&swapper);

        assert_eq!(swapper_x, U256::from(0));
        assert_eq!(swapper_y, U256::from(0));
        assert_eq!(swapper_z, U256::from(986));

        let pool_xy = invariant
            .get_pool(
                *token_x.address(),
                *token_y.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(
            pool_xy.fee_protocol_token_x,
            TokenAmount::new(U256::from(1))
        );
        assert_eq!(
            pool_xy.fee_protocol_token_y,
            TokenAmount::new(U256::from(0))
        );

        let pool_yz = invariant
            .get_pool(
                *token_y.address(),
                *token_z.address(),
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
            )
            .unwrap();

        assert_eq!(
            pool_yz.fee_protocol_token_x,
            TokenAmount::new(U256::from(1))
        );
        assert_eq!(
            pool_yz.fee_protocol_token_y,
            TokenAmount::new(U256::from(0))
        );

        let deployer_x_before = token_x.balance_of(&deployer);
        let deployer_y_before = token_y.balance_of(&deployer);
        let deployer_z_before = token_z.balance_of(&deployer);

        test_env::set_caller(deployer);
        invariant.claim_fee(0).unwrap();
        invariant.claim_fee(1).unwrap();

        let deployer_x_after = token_x.balance_of(&deployer);
        let deployer_y_after = token_y.balance_of(&deployer);
        let deployer_z_after = token_z.balance_of(&deployer);

        assert_eq!(deployer_x_after - deployer_x_before, U256::from(4));
        assert_eq!(deployer_y_after - deployer_y_before, U256::from(4));
        assert_eq!(deployer_z_after - deployer_z_before, U256::from(0));
    }
}
