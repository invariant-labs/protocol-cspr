use crate::{
    contracts::{FeeTier, InvariantError, PoolKey},
    math::{
        liquidity::Liquidity,
        percentage::Percentage,
        sqrt_price::{calculate_sqrt_price, SqrtPrice},
    },
    Erc20Deployer, InvariantDeployer,
};
use alloc::string::String;
use decimal::{Decimal, Factories};
use odra::{
    test_env,
    types::{U128, U256},
};

#[test]
fn test_position_slippage_zero_slippage_and_inside_range() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);

    let mint_amount = 10u128.pow(23);

    let mut token_x = Erc20Deployer::init(
        String::from(""),
        String::from(""),
        0,
        &Some(U256::from(mint_amount)),
    );
    let mut token_y = Erc20Deployer::init(
        String::from(""),
        String::from(""),
        0,
        &Some(U256::from(mint_amount)),
    );
    let mut invariant = InvariantDeployer::init(U128::from(0));

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    invariant
        .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
        .unwrap();

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

    let mint_amount = 10u128.pow(10);
    token_x.approve(invariant.address(), &U256::from(mint_amount));
    token_y.approve(invariant.address(), &U256::from(mint_amount));

    let lower_tick = -1000;
    let upper_tick = 1000;
    let liquidity = Liquidity::from_integer(10_000_000_000u64);

    let pool_before = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();
    let slippage_limit_lower = pool_before.sqrt_price;
    let slippage_limit_upper = pool_before.sqrt_price;
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

    let pool_after = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();

    assert_eq!(pool_after.liquidity, liquidity);

    let pool = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();

    // zero slippage
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let known_price = pool.sqrt_price;
        let tick = pool_key.fee_tier.tick_spacing as i32;
        invariant
            .create_position(
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                -tick,
                tick,
                liquidity_delta.get(),
                known_price.get(),
                known_price.get(),
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
                pool_key.token_x,
                pool_key.token_y,
                fee_tier.fee.get(),
                fee_tier.tick_spacing,
                -tick,
                tick,
                liquidity_delta.get(),
                limit_lower.get(),
                limit_upper.get(),
            )
            .unwrap();
    }
}

#[test]
#[should_panic]
fn test_position_slippage_below_range() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);

    let mint_amount = 10u128.pow(23);

    let mut token_x = Erc20Deployer::init(
        String::from(""),
        String::from(""),
        0,
        &Some(U256::from(mint_amount)),
    );
    let mut token_y = Erc20Deployer::init(
        String::from(""),
        String::from(""),
        0,
        &Some(U256::from(mint_amount)),
    );
    let mut invariant = InvariantDeployer::init(U128::from(0));

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };

    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    invariant
        .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
        .unwrap();

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

    let mint_amount = 10u128.pow(10);
    token_x.approve(invariant.address(), &U256::from(mint_amount));
    token_y.approve(invariant.address(), &U256::from(mint_amount));

    let lower_tick = -1000;
    let upper_tick = 1000;
    let liquidity = Liquidity::from_integer(10_000_000_000u64);

    let pool_before = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();
    let slippage_limit_lower = pool_before.sqrt_price;
    let slippage_limit_upper = pool_before.sqrt_price;
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

    let pool_after = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();

    assert_eq!(pool_after.liquidity, liquidity);

    invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(U128::from(1014432353584998786339859u128));
    let limit_upper = SqrtPrice::new(U128::from(1045335831204498605270797u128));
    let tick = pool_key.fee_tier.tick_spacing as i32;
    let result = invariant.create_position(
        pool_key.token_x,
        pool_key.token_y,
        fee_tier.fee.get(),
        fee_tier.tick_spacing,
        -tick,
        tick,
        liquidity_delta.get(),
        limit_lower.get(),
        limit_upper.get(),
    );

    assert_eq!(result, Err(InvariantError::PriceLimitReached));
}

#[test]
#[should_panic]
fn test_position_slippage_above_range() {
    let deployer = test_env::get_account(0);
    test_env::set_caller(deployer);

    let mint_amount = 10u128.pow(23);

    let mut token_x = Erc20Deployer::init(
        String::from(""),
        String::from(""),
        0,
        &Some(U256::from(mint_amount)),
    );
    let mut token_y = Erc20Deployer::init(
        String::from(""),
        String::from(""),
        0,
        &Some(U256::from(mint_amount)),
    );
    let mut invariant = InvariantDeployer::init(U128::from(0));

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };
    let pool_key = PoolKey::new(*token_x.address(), *token_y.address(), fee_tier).unwrap();
    invariant
        .add_fee_tier(fee_tier.fee.get(), fee_tier.tick_spacing)
        .unwrap();

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

    let mint_amount = 10u128.pow(10);
    token_x.approve(invariant.address(), &U256::from(mint_amount));
    token_y.approve(invariant.address(), &U256::from(mint_amount));

    let lower_tick = -1000;
    let upper_tick = 1000;
    let liquidity = Liquidity::from_integer(10_000_000_000u64);

    let pool_before = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();
    let slippage_limit_lower = pool_before.sqrt_price;
    let slippage_limit_upper = pool_before.sqrt_price;
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

    let pool_after = invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();

    assert_eq!(pool_after.liquidity, liquidity);

    invariant
        .get_pool(
            pool_key.token_x,
            pool_key.token_y,
            fee_tier.fee.get(),
            fee_tier.tick_spacing,
        )
        .unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(U128::from(955339206774222158009382u128));
    let limit_upper = SqrtPrice::new(U128::from(984442481813945288458906u128));
    let tick = pool_key.fee_tier.tick_spacing as i32;
    let result = invariant.create_position(
        pool_key.token_x,
        pool_key.token_y,
        fee_tier.fee.get(),
        fee_tier.tick_spacing,
        -tick,
        tick,
        liquidity_delta.get(),
        limit_lower.get(),
        limit_upper.get(),
    );

    assert_eq!(result, Err(InvariantError::PriceLimitReached));
}
