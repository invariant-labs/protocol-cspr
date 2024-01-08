use super::snippets::{init, positions_equals};
use crate::contracts::{InvariantError, PoolKey};
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
fn test_interaction_with_pool_on_removed_fee_tier() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);

    let mint_amount = U256::from(10u128.pow(10));
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
    // Remove fee tier
    {
        invariant.remove_fee_tier(fee_tier).unwrap();
        let exist = invariant.fee_tier_exist(fee_tier);
        assert!(!exist);
    }
    // Attempt to create same pool again
    {
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let result = invariant.create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            init_sqrt_price,
            init_tick,
        );
        assert_eq!(result, Err(InvariantError::FeeTierNotFound));
    }
    // Init position
    {
        let mint_amount = U256::from(10u128.pow(10));
        token_x.mint(&deployer, &mint_amount);
        token_y.mint(&deployer, &mint_amount);
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key,
                lower_tick,
                upper_tick,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let pos = invariant.get_all_positions(deployer);
        assert_eq!(1, pos.len());
        let position = invariant.get_position(deployer, 0).unwrap();
        assert_eq!(position.liquidity, liquidity);
        assert_eq!(pool_after.liquidity, liquidity)
    }
    // Perform swap
    {
        let caller = test_env::get_account(1);
        let amount = U256::from(1000);
        token_x.mint(&caller, &amount);

        test_env::set_caller(caller);
        token_x.approve(invariant.address(), &amount);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage = SqrtPrice::new(U128::from(MIN_SQRT_PRICE));
        let swap_amount = TokenAmount::new(amount);
        invariant
            .swap(pool_key, true, swap_amount, true, slippage)
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
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
        let (claimed_x, claimed_y) = invariant.claim_fee(0).unwrap();
        assert_eq!(claimed_x, TokenAmount::new(U256::from(5)));
        assert_eq!(claimed_y, TokenAmount::new(U256::from(0)));
    }
    // Change fee receiver
    {
        let new_receiver = test_env::get_account(2);
        invariant
            .change_fee_receiver(pool_key, new_receiver)
            .unwrap();
    }
    // Withdraw protocol fee
    {
        let protocol_fee_receiver = test_env::get_account(2);
        test_env::set_caller(protocol_fee_receiver);
        let receiver_x_before = token_x.balance_of(&protocol_fee_receiver);
        let receiver_y_before = token_y.balance_of(&protocol_fee_receiver);
        invariant.withdraw_protocol_fee(pool_key).unwrap();
        let receiver_x_after = token_x.balance_of(&protocol_fee_receiver);
        let receiver_y_after = token_y.balance_of(&protocol_fee_receiver);

        let expected_withdrawn_x = TokenAmount::new(U256::from(1));
        let expected_withdrawn_y = TokenAmount::new(U256::from(0));
        assert_eq!(
            receiver_x_before + expected_withdrawn_x.get(),
            receiver_x_after
        );
        assert_eq!(
            receiver_y_before + expected_withdrawn_y.get(),
            receiver_y_after
        );
    }
    // Close position
    {
        test_env::set_caller(deployer);
        invariant.remove_position(0).unwrap();
    }
    // Get pool
    {
        invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();
    }
    // Get pools
    {
        let pools = invariant.get_pools();
        assert_eq!(pools.len(), 1);
    }
    // Transfer Position
    {
        let position_owner = test_env::get_account(0);
        test_env::set_caller(position_owner);
        let mint_amount = U256::from(10u128.pow(10));
        token_x.mint(&deployer, &mint_amount);
        token_y.mint(&deployer, &mint_amount);
        token_x.approve(invariant.address(), &mint_amount);
        token_y.approve(invariant.address(), &mint_amount);

        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;

        invariant
            .create_position(
                pool_key,
                lower_tick,
                upper_tick,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            )
            .unwrap();

        let pool_after = invariant
            .get_pool(pool_key.token_x, pool_key.token_y, fee_tier)
            .unwrap();

        assert_eq!(pool_after.liquidity, liquidity);

        let recipient = test_env::get_account(1);

        let transferred_index = 0;
        let owner_list_before = invariant.get_all_positions(position_owner);
        test_env::set_caller(recipient);
        let recipient_list_before = invariant.get_all_positions(recipient);
        test_env::set_caller(position_owner);
        let removed_position = invariant
            .get_position(position_owner, transferred_index)
            .unwrap();

        invariant
            .transfer_position(transferred_index, recipient)
            .unwrap();

        test_env::set_caller(recipient);
        let recipient_position = invariant
            .get_position(recipient, transferred_index)
            .unwrap();
        let recipient_list_after = invariant.get_all_positions(recipient);
        test_env::set_caller(position_owner);
        let owner_positions_after = invariant.get_all_positions(position_owner);
        let owner_list_after = invariant.get_all_positions(position_owner);

        assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
        assert_eq!(owner_list_before.len() - 1, owner_list_after.len());
        assert_eq!(owner_positions_after.len(), 0);

        // Equals fields od transferred position
        assert!(positions_equals(recipient_position, removed_position));
    }
    // Readd fee tier and create same pool
    {
        invariant.add_fee_tier(fee_tier).unwrap();
        let exist = invariant.fee_tier_exist(fee_tier);
        assert!(exist);

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let result = invariant.create_pool(
            *token_x.address(),
            *token_y.address(),
            fee_tier,
            init_sqrt_price,
            init_tick,
        );
        assert_eq!(result, Err(InvariantError::PoolAlreadyExist));
    }
}
