use crate::{
    contracts::{FeeTier, InvariantError, PoolKey},
    math::{
        liquidity::Liquidity,
        percentage::Percentage,
        sqrt_price::{calculate_sqrt_price, SqrtPrice},
    },
    token::{TokenDeployer, TokenRef},
    InvariantDeployer, InvariantRef,
};
use alloc::string::String;
use decimal::{Decimal, Factories};
use odra::{
    test_env,
    types::{U128, U256},
};

fn init_slippage_dex_and_tokens() -> (InvariantRef, TokenRef, TokenRef) {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let mint_amount = 10u128.pow(23);

    let token_x = TokenDeployer::init(
        String::from(""),
        String::from(""),
        0,
        &U256::from(mint_amount),
    );
    let token_y = TokenDeployer::init(
        String::from(""),
        String::from(""),
        0,
        &U256::from(mint_amount),
    );
    let invariant = InvariantDeployer::init(Percentage::new(U128::from(0)));

    (invariant, token_x, token_y)
}

fn init_slippage_pool_with_liquidity(
    mut invariant: InvariantRef,
    mut token_x: TokenRef,
    mut token_y: TokenRef,
) -> (InvariantRef, TokenRef, TokenRef, PoolKey) {
    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
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
    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };

    let mint_amount = 10u128.pow(10);
    token_x.approve(invariant.address(), &U256::from(mint_amount));
    token_y.approve(invariant.address(), &U256::from(mint_amount));

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    let lower_tick = -1000;
    let upper_tick = 1000;
    let liquidity = Liquidity::from_integer(10_000_000_000u64);

    let pool_before = invariant
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
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
        .get_pool(*token_x.address(), *token_y.address(), fee_tier)
        .unwrap();

    assert_eq!(pool_after.liquidity, liquidity);

    (invariant, token_x, token_y, pool_key)
}

#[test]
fn test_position_slippage_zero_slippage_and_inside_range() {
    let alice = test_env::get_account(0);
    test_env::set_caller(alice);

    let (invariant, token_x, token_y) = init_slippage_dex_and_tokens();
    let (mut invariant, token_x, token_y, pool_key) =
        init_slippage_pool_with_liquidity(invariant, token_x, token_y);

    let pool = invariant
        .get_pool(*token_x.address(), *token_y.address(), pool_key.fee_tier)
        .unwrap();

    // zero slippage
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let known_price = pool.sqrt_price;
        let tick = pool_key.fee_tier.tick_spacing as i32;
        invariant
            .create_position(
                pool_key,
                -tick,
                tick,
                liquidity_delta,
                known_price,
                known_price,
            )
            .unwrap();
    }
    // inside range
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let limit_lower = SqrtPrice::new(U128::from(994734637981406576896367u128));
        let limit_upper = SqrtPrice::new(U128::from(1025038048074314166333500u128));

        let tick = pool_key.fee_tier.tick_spacing as i32;

        invariant
            .create_position(
                pool_key,
                -tick,
                tick,
                liquidity_delta,
                limit_lower,
                limit_upper,
            )
            .unwrap();
    }
}

#[test]
fn test_position_slippage_below_range() {
    let (invariant, token_x, token_y) = init_slippage_dex_and_tokens();
    let (mut invariant, token_x, token_y, pool_key) =
        init_slippage_pool_with_liquidity(invariant, token_x, token_y);

    invariant
        .get_pool(*token_x.address(), *token_y.address(), pool_key.fee_tier)
        .unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(U128::from(1014432353584998786339859u128));
    let limit_upper = SqrtPrice::new(U128::from(1045335831204498605270797u128));
    let tick = pool_key.fee_tier.tick_spacing as i32;
    let result = invariant.create_position(
        pool_key,
        -tick,
        tick,
        liquidity_delta,
        limit_lower,
        limit_upper,
    );

    assert_eq!(result, Err(InvariantError::PriceLimitReached));
}
