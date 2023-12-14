use crate::contracts::{get_liquidity_by_x, get_liquidity_by_y, FeeTier, PoolKey};
use crate::e2e::snippets::init;
use crate::math::get_delta_y;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::math::sqrt_price::get_max_tick;
use crate::math::token_amount::TokenAmount;
use crate::math::{
    percentage::Percentage, sqrt_price::SqrtPrice, MAX_SQRT_PRICE, MAX_TICK, MIN_SQRT_PRICE,
};
use crate::token::{TokenDeployer, TokenRef};
use crate::{InvariantDeployer, InvariantRef};
use decimal::{Decimal, Factories};
use odra::prelude::string::String;
use odra::test_env;
use odra::types::{U128, U256};

fn big_deposit_and_swap(x_to_y: bool) {
    let (mut invariant, mut token_x, mut token_y) =
        init(Percentage::from_scale(1, 2), U256::max_value());

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

#[test]
fn test_limits_big_deposit_both_tokens() {
    let (mut invariant, mut token_x, mut token_y) =
        init(Percentage::from_scale(1, 2), U256::max_value());

    let mint_amount = "95780971304118053647396689196894323976171195136475136"; // 2^176

    token_x.approve(invariant.address(), &U256::max_value());
    token_y.approve(invariant.address(), &U256::max_value());

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();

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

    let lower_tick = -(fee_tier.tick_spacing as i32);
    let upper_tick = fee_tier.tick_spacing as i32;
    let pool = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();
    let liquidity_delta = get_liquidity_by_x(
        TokenAmount::new(U256::from_dec_str(mint_amount).unwrap()),
        lower_tick,
        upper_tick,
        pool.sqrt_price,
        false,
    )
    .unwrap()
    .l;
    let y = get_delta_y(
        calculate_sqrt_price(lower_tick).unwrap(),
        pool.sqrt_price,
        liquidity_delta,
        true,
    )
    .unwrap();

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
    let user_amount_x = token_x.balance_of(&alice);
    let user_amount_y = token_y.balance_of(&alice);
    assert_eq!(
        user_amount_x,
        U256::max_value() - U256::from_dec_str(mint_amount).unwrap()
    );
    assert_eq!(user_amount_y, U256::max_value() - y.get());

    let contract_amount_x = token_x.balance_of(invariant.address());
    let contract_amount_y = token_y.balance_of(invariant.address());
    assert_eq!(contract_amount_x, U256::from_dec_str(mint_amount).unwrap());
    assert_eq!(contract_amount_y, y.get());
}

#[test]
fn test_deposit_limits_at_upper_limit() {
    let (mut invariant, mut token_x, mut token_y) =
        init(Percentage::from_scale(1, 2), U256::max_value());

    let mint_amount = "110427941548649020598956093796432407239217743554726184882600387580788736"; // 2^236
    token_x.approve(invariant.address(), &U256::max_value());
    token_y.approve(invariant.address(), &U256::max_value());

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();
    invariant.add_fee_tier(fee_tier).unwrap();

    let init_tick = get_max_tick(1);
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

    let pool = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();
    assert_eq!(pool.current_tick_index, init_tick);
    assert_eq!(pool.sqrt_price, calculate_sqrt_price(init_tick).unwrap());

    let position_amount = U256::from_dec_str(mint_amount).unwrap() - U256::from(1);

    let liquidity_delta = get_liquidity_by_y(
        TokenAmount::new(position_amount),
        0,
        MAX_TICK,
        pool.sqrt_price,
        false,
    )
    .unwrap()
    .l;

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;
    invariant
        .create_position(
            pool_key,
            0,
            MAX_TICK,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
        )
        .unwrap();
}

#[test]
fn test_limits_big_deposit_and_swaps() {
    let (mut invariant, mut token_x, mut token_y) =
        init(Percentage::from_scale(1, 2), U256::max_value());

    let mint_amount = "191561942608236107294793378393788647952342390272950272"; // 2^177
    token_x.approve(invariant.address(), &U256::max_value());
    token_y.approve(invariant.address(), &U256::max_value());

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();
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

    let pos_amount = U256::from_dec_str(mint_amount).unwrap() / 2;
    let lower_tick = -(fee_tier.tick_spacing as i32);
    let upper_tick = fee_tier.tick_spacing as i32;
    let pool = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    let liquidity_delta = get_liquidity_by_x(
        TokenAmount::new(pos_amount),
        lower_tick,
        upper_tick,
        pool.sqrt_price,
        false,
    )
    .unwrap()
    .l;

    let y = get_delta_y(
        calculate_sqrt_price(lower_tick).unwrap(),
        pool.sqrt_price,
        liquidity_delta,
        true,
    )
    .unwrap();

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
    let user_amount_x = token_x.balance_of(&alice);
    let user_amount_y = token_y.balance_of(&alice);
    assert_eq!(user_amount_x, U256::max_value() - pos_amount);
    assert_eq!(user_amount_y, U256::max_value() - y.get());

    let contract_amount_x = token_x.balance_of(invariant.address());
    let contract_amount_y = token_y.balance_of(invariant.address());
    assert_eq!(contract_amount_x, pos_amount);
    assert_eq!(contract_amount_y, y.get());

    let swap_amount = TokenAmount::new(U256::from_dec_str(mint_amount).unwrap() / 8);

    for i in 1..=4 {
        let (_, sqrt_price_limit) = if i % 2 == 0 {
            (true, SqrtPrice::new(U128::from(MIN_SQRT_PRICE)))
        } else {
            (false, SqrtPrice::new(U128::from(MAX_SQRT_PRICE)))
        };

        invariant
            .swap(pool_key, i % 2 == 0, swap_amount, true, sqrt_price_limit)
            .unwrap();
    }
}
