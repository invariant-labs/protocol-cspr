use crate::contracts::{get_liquidity_by_x, get_liquidity_by_y, FeeTier, PoolKey};
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::math::token_amount::TokenAmount;
use crate::math::{percentage::Percentage, sqrt_price::SqrtPrice, MAX_SQRT_PRICE, MIN_SQRT_PRICE};
use crate::token::{TokenDeployer, TokenRef};
use crate::{InvariantDeployer, InvariantRef};
use decimal::{Decimal, Factories};
use odra::prelude::string::String;
use odra::test_env;
use odra::types::{U128, U256};

fn init_dex_and_tokens_max_mint_amount() -> (InvariantRef, TokenRef, TokenRef) {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let mint_amount = U256::max_value();
    let token_x = TokenDeployer::init(String::from(""), String::from(""), 0, &mint_amount);
    let token_y = TokenDeployer::init(String::from(""), String::from(""), 0, &mint_amount);
    let invariant = InvariantDeployer::init(Percentage::from_scale(1, 2));

    (invariant, token_x, token_y)
}

fn big_deposit_and_swap(x_to_y: bool) {
    let (mut invariant, mut token_x, mut token_y) = init_dex_and_tokens_max_mint_amount();

    let mint_amount = "102844034832575377634685573909834406561420991602098741459288064"; // 2^206
    token_x.approve(
        invariant.address(),
        &U256::from_dec_str(mint_amount).unwrap(),
    );
    token_y.approve(
        invariant.address(),
        &U256::from_dec_str(mint_amount).unwrap(),
    );

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 1,
    };
    invariant.add_fee_tier(fee_tier).unwrap();

    let init_tick = 0;
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

    let lower_tick = if x_to_y {
        -(fee_tier.tick_spacing as i32)
    } else {
        0
    };
    let upper_tick = if x_to_y {
        0
    } else {
        fee_tier.tick_spacing as i32
    };
    let pool = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    let liquidity_delta = if x_to_y {
        get_liquidity_by_y(
            TokenAmount::new(U256::from_dec_str(mint_amount).unwrap()),
            lower_tick,
            upper_tick,
            pool.sqrt_price,
            true,
        )
        .unwrap()
        .l
    } else {
        get_liquidity_by_x(
            TokenAmount::new(U256::from_dec_str(mint_amount).unwrap()),
            lower_tick,
            upper_tick,
            pool.sqrt_price,
            true,
        )
        .unwrap()
        .l
    };

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;
    invariant
        .create_position(
            pool_key,
            lower_tick,
            upper_tick,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
        )
        .unwrap();

    let alice = test_env::get_account(0);

    let amount_x = token_x.balance_of(&alice);
    let amount_y = token_y.balance_of(&alice);
    if x_to_y {
        assert_eq!(
            amount_x,
            U256::from_dec_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            )
            .unwrap()
        );
        assert_eq!(
            amount_y,
            U256::from_dec_str(
                "115792089237316092579536152433310273167696074831234002618465981909171670351871"
            )
            .unwrap()
        );
    } else {
        assert_eq!(
            amount_x,
            U256::from_dec_str(
                "115792089237316092579536152433310273167696074831234002618465981909171670351871"
            )
            .unwrap()
        );
        assert_eq!(
            amount_y,
            U256::from_dec_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            )
            .unwrap()
        );
    }

    let sqrt_price_limit = if x_to_y {
        SqrtPrice::new(U128::from(MIN_SQRT_PRICE))
    } else {
        SqrtPrice::new(U128::from(MAX_SQRT_PRICE))
    };

    invariant
        .swap(
            pool_key,
            x_to_y,
            TokenAmount::new(U256::from_dec_str(mint_amount).unwrap()),
            true,
            sqrt_price_limit,
        )
        .unwrap();

    let amount_x = token_x.balance_of(&alice);
    let amount_y = token_y.balance_of(&alice);
    if x_to_y {
        assert_eq!(
            amount_x,
            U256::from_dec_str(
                "115792089237316092579536152433310273167696074831234002618465981909171670351871"
            )
            .unwrap()
        );
        assert_ne!(amount_y, U256::from(0));
    } else {
        assert_ne!(amount_x, U256::from(0));
        assert_eq!(
            amount_y,
            U256::from_dec_str(
                "115792089237316092579536152433310273167696074831234002618465981909171670351871"
            )
            .unwrap()
        );
    };
}

#[test]
fn test_limits_big_deposit_x_and_swap_y() {
    big_deposit_and_swap(true);
}

#[test]
fn test_limits_big_deposit_y_and_swap_x() {
    big_deposit_and_swap(false);
}
