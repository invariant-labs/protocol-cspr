/* tslint:disable */
/* eslint-disable */
/**
* @param {any} js_current_sqrt_price
* @param {any} js_target_sqrt_price
* @param {any} js_liquidity
* @param {any} js_amount
* @param {any} js_by_amount_in
* @param {any} js_fee
* @returns {any}
*/
export function computeSwapStep(js_current_sqrt_price: any, js_target_sqrt_price: any, js_liquidity: any, js_amount: any, js_by_amount_in: any, js_fee: any): any;
/**
* @param {any} js_sqrt_price_a
* @param {any} js_sqrt_price_b
* @param {any} js_liquidity
* @param {any} js_rounding_up
* @returns {any}
*/
export function getDeltaX(js_sqrt_price_a: any, js_sqrt_price_b: any, js_liquidity: any, js_rounding_up: any): any;
/**
* @param {any} js_sqrt_price_a
* @param {any} js_sqrt_price_b
* @param {any} js_liquidity
* @param {any} js_rounding_up
* @returns {any}
*/
export function getDeltaY(js_sqrt_price_a: any, js_sqrt_price_b: any, js_liquidity: any, js_rounding_up: any): any;
/**
* @param {any} js_starting_sqrt_price
* @param {any} js_liquidity
* @param {any} js_amount
* @param {any} js_x_to_y
* @returns {any}
*/
export function getNextSqrtPriceFromInput(js_starting_sqrt_price: any, js_liquidity: any, js_amount: any, js_x_to_y: any): any;
/**
* @param {any} js_starting_sqrt_price
* @param {any} js_liquidity
* @param {any} js_amount
* @param {any} js_x_to_y
* @returns {any}
*/
export function getNextSqrtPriceFromOutput(js_starting_sqrt_price: any, js_liquidity: any, js_amount: any, js_x_to_y: any): any;
/**
* @param {any} js_starting_sqrt_price
* @param {any} js_liquidity
* @param {any} js_x
* @param {any} js_add_x
* @returns {any}
*/
export function getNextSqrtPriceXUp(js_starting_sqrt_price: any, js_liquidity: any, js_x: any, js_add_x: any): any;
/**
* @param {any} js_starting_sqrt_price
* @param {any} js_liquidity
* @param {any} js_y
* @param {any} js_add_y
* @returns {any}
*/
export function getNextSqrtPriceYDown(js_starting_sqrt_price: any, js_liquidity: any, js_y: any, js_add_y: any): any;
/**
* @param {any} js_current_tick_index
* @param {any} js_current_sqrt_price
* @param {any} js_liquidity_delta
* @param {any} js_liquidity_sign
* @param {any} js_upper_tick
* @param {any} js_lower_tick
* @returns {any}
*/
export function calculateAmountDelta(js_current_tick_index: any, js_current_sqrt_price: any, js_liquidity_delta: any, js_liquidity_sign: any, js_upper_tick: any, js_lower_tick: any): any;
/**
* @param {any} js_amount
* @param {any} js_starting_sqrt_price
* @param {any} js_liquidity
* @param {any} js_fee
* @param {any} js_by_amount_in
* @param {any} js_x_to_y
* @returns {any}
*/
export function isEnoughAmountToChangePrice(js_amount: any, js_starting_sqrt_price: any, js_liquidity: any, js_fee: any, js_by_amount_in: any, js_x_to_y: any): any;
/**
* @param {any} js_tick_spacing
* @returns {bigint}
*/
export function calculateMaxLiquidityPerTick(js_tick_spacing: any): bigint;
/**
* @param {any} js_tick_lower
* @param {any} js_tick_upper
* @param {any} js_tick_spacing
* @returns {any}
*/
export function checkTicks(js_tick_lower: any, js_tick_upper: any, js_tick_spacing: any): any;
/**
* @param {any} js_tick_index
* @param {any} js_tick_spacing
* @returns {any}
*/
export function checkTick(js_tick_index: any, js_tick_spacing: any): any;
/**
* @param {any} js_expected_amount_out
* @param {any} js_slippage
* @returns {bigint}
*/
export function calculateMinAmountOut(js_expected_amount_out: any, js_slippage: any): bigint;
/**
* @returns {bigint}
*/
export function getTokenAmountScale(): bigint;
/**
* @returns {bigint}
*/
export function getTokenAmountDenominator(): bigint;
/**
* @param {any} js_val
* @param {any} js_scale
* @returns {bigint}
*/
export function toTokenAmount(js_val: any, js_scale: any): bigint;
/**
* @returns {bigint}
*/
export function getLiquidityScale(): bigint;
/**
* @returns {bigint}
*/
export function getLiquidityDenominator(): bigint;
/**
* @param {any} js_val
* @param {any} js_scale
* @returns {bigint}
*/
export function toLiquidity(js_val: any, js_scale: any): bigint;
/**
* @param {any} js_fee
* @param {any} js_tick_spacing
* @returns {any}
*/
export function _newFeeTier(js_fee: any, js_tick_spacing: any): any;
/**
* @returns {bigint}
*/
export function getPercentageScale(): bigint;
/**
* @returns {bigint}
*/
export function getPercentageDenominator(): bigint;
/**
* @param {any} js_val
* @param {any} js_scale
* @returns {bigint}
*/
export function toPercentage(js_val: any, js_scale: any): bigint;
/**
* @returns {bigint}
*/
export function getFixedPointScale(): bigint;
/**
* @returns {bigint}
*/
export function getFixedPointDenominator(): bigint;
/**
* @param {any} js_val
* @param {any} js_scale
* @returns {bigint}
*/
export function toFixedPoint(js_val: any, js_scale: any): bigint;
/**
* @returns {bigint}
*/
export function getSqrtPriceScale(): bigint;
/**
* @returns {bigint}
*/
export function getSqrtPriceDenominator(): bigint;
/**
* @param {any} js_val
* @param {any} js_scale
* @returns {bigint}
*/
export function toSqrtPrice(js_val: any, js_scale: any): bigint;
/**
* @returns {bigint}
*/
export function getGlobalMaxSqrtPrice(): bigint;
/**
* @returns {bigint}
*/
export function getGlobalMinSqrtPrice(): bigint;
/**
* @returns {bigint}
*/
export function getTickSearchRange(): bigint;
/**
* @param {any} js_tick_spacing
* @returns {bigint}
*/
export function getMaxChunk(js_tick_spacing: any): bigint;
/**
* @returns {bigint}
*/
export function getChunkSize(): bigint;
/**
* @param {any} js_lower_tick_index
* @param {any} js_lower_tick_fee_growth_outside_x
* @param {any} js_lower_tick_fee_growth_outside_y
* @param {any} js_upper_tick_index
* @param {any} js_upper_tick_fee_growth_outside_x
* @param {any} js_upper_tick_fee_growth_outside_y
* @param {any} js_pool_current_tick_index
* @param {any} js_pool_fee_growth_global_x
* @param {any} js_pool_fee_growth_global_y
* @param {any} js_position_fee_growth_inside_x
* @param {any} js_position_fee_growth_inside_y
* @param {any} js_position_liquidity
* @returns {any}
*/
export function _calculateFee(js_lower_tick_index: any, js_lower_tick_fee_growth_outside_x: any, js_lower_tick_fee_growth_outside_y: any, js_upper_tick_index: any, js_upper_tick_fee_growth_outside_x: any, js_upper_tick_fee_growth_outside_y: any, js_pool_current_tick_index: any, js_pool_fee_growth_global_x: any, js_pool_fee_growth_global_y: any, js_position_fee_growth_inside_x: any, js_position_fee_growth_inside_y: any, js_position_liquidity: any): any;
/**
* @param {any} js_token_candidate
* @param {any} js_token_to_compare
* @returns {any}
*/
export function isTokenX(js_token_candidate: any, js_token_to_compare: any): any;
/**
* @param {any} js_tick_index
* @param {any} js_tick_spacing
* @param {any} js_sqrt_price
* @returns {any}
*/
export function isValidTick(js_tick_index: any, js_tick_spacing: any, js_sqrt_price: any): any;
/**
* @param {any} js_sqrt_price
* @param {any} js_tick_spacing
* @returns {any}
*/
export function calculateTick(js_sqrt_price: any, js_tick_spacing: any): any;
/**
* @param {any} js_x
* @param {any} js_lower_tick
* @param {any} js_upper_tick
* @param {any} js_current_sqrt_price
* @param {any} js_rounding_up
* @returns {any}
*/
export function getLiquidityByX(js_x: any, js_lower_tick: any, js_upper_tick: any, js_current_sqrt_price: any, js_rounding_up: any): any;
/**
* @param {any} js_y
* @param {any} js_lower_tick
* @param {any} js_upper_tick
* @param {any} js_current_sqrt_price
* @param {any} js_rounding_up
* @returns {any}
*/
export function getLiquidityByY(js_y: any, js_lower_tick: any, js_upper_tick: any, js_current_sqrt_price: any, js_rounding_up: any): any;
/**
* @param {any} token_0
* @param {any} token_1
* @param {any} fee_tier
* @returns {any}
*/
export function _newPoolKey(token_0: any, token_1: any, fee_tier: any): any;
/**
* @returns {bigint}
*/
export function getFeeGrowthScale(): bigint;
/**
* @returns {bigint}
*/
export function getFeeGrowthDenominator(): bigint;
/**
* @param {any} js_val
* @param {any} js_scale
* @returns {bigint}
*/
export function toFeeGrowth(js_val: any, js_scale: any): bigint;
/**
* @returns {bigint}
*/
export function getSecondsPerLiquidityScale(): bigint;
/**
* @returns {bigint}
*/
export function getSecondsPerLiquidityDenominator(): bigint;
/**
* @param {any} js_val
* @param {any} js_scale
* @returns {bigint}
*/
export function toSecondsPerLiquidity(js_val: any, js_scale: any): bigint;
/**
*/
export enum InvariantError {
  NotAdmin = 0,
  NotFeeReceiver = 1,
  PoolAlreadyExist = 2,
  PoolNotFound = 3,
  TickAlreadyExist = 4,
  InvalidTickIndexOrTickSpacing = 5,
  PositionNotFound = 6,
  TickNotFound = 7,
  FeeTierNotFound = 8,
  PoolKeyNotFound = 9,
  AmountIsZero = 10,
  WrongLimit = 11,
  PriceLimitReached = 12,
  NoGainSwap = 13,
  InvalidTickSpacing = 14,
  FeeTierAlreadyExist = 15,
  PoolKeyAlreadyExist = 16,
  UnauthorizedFeeReceiver = 17,
  ZeroLiquidity = 18,
  TransferError = 19,
  TokensAreSame = 20,
  AmountUnderMinimumAmountOut = 21,
  InvalidFee = 22,
  NotEmptyTickDeinitialization = 23,
  InvalidInitTick = 24,
  InvalidInitSqrtPrice = 25,
}
export interface SwapResult {
    next_sqrt_price: SqrtPrice;
    amount_in: TokenAmount;
    amount_out: TokenAmount;
    fee_amount: TokenAmount;
}

export type calculateAmountDeltaResult = [TokenAmount, TokenAmount, boolean];

export interface SwapEvent {
    timestamp: bigint;
    address: string;
    pool: PoolKey;
    amountIn: TokenAmount;
    amountOut: TokenAmount;
    fee: TokenAmount;
    startSqrtPrice: SqrtPrice;
    targetSqrtPrice: SqrtPrice;
    xToY: boolean;
}

export interface CrossTickEvent {
    timestamp: bigint;
    address: string;
    pool: PoolKey;
    indexes: bigint[];
}

export interface RemovePositionEvent {
    timestamp: bigint;
    address: string;
    pool: PoolKey;
    liquidity: Liquidity;
    lowerTick: bigint;
    upperTick: bigint;
    currentSqrtPrice: SqrtPrice;
}

export interface CreatePositionEvent {
    timestamp: bigint;
    address: string;
    pool: PoolKey;
    liquidity: Liquidity;
    lowerTick: bigint;
    upperTick: bigint;
    currentSqrtPrice: SqrtPrice;
}

export interface TokenAmount {
    v: bigint;
}

export interface Liquidity {
    v: bigint;
}

export interface FeeTier {
    fee: Percentage;
    tickSpacing: bigint;
}

export interface InvariantConfig {
    admin: string;
    protocolFee: Percentage;
}

export interface Percentage {
    v: bigint;
}

export interface FixedPoint {
    v: bigint;
}

export interface SqrtPrice {
    v: bigint;
}

export interface SwapHop {
    poolKey: PoolKey;
    xToY: boolean;
}

export interface QuoteResult {
    amountIn: TokenAmount;
    amountOut: TokenAmount;
    targetSqrtPrice: SqrtPrice;
    ticks: Tick[];
}

export interface TokenAmounts {
    x: TokenAmount;
    y: TokenAmount;
}

export type _calculateFeeResult = [TokenAmount, TokenAmount];

export type InvariantError = "NotAdmin" | "NotFeeReceiver" | "PoolAlreadyExist" | "PoolNotFound" | "TickAlreadyExist" | "InvalidTickIndexOrTickSpacing" | "PositionNotFound" | "TickNotFound" | "FeeTierNotFound" | "PoolKeyNotFound" | "AmountIsZero" | "WrongLimit" | "PriceLimitReached" | "NoGainSwap" | "InvalidTickSpacing" | "FeeTierAlreadyExist" | "PoolKeyAlreadyExist" | "UnauthorizedFeeReceiver" | "ZeroLiquidity" | "TransferError" | "TokensAreSame" | "AmountUnderMinimumAmountOut" | "InvalidFee" | "NotEmptyTickDeinitialization" | "InvalidInitTick" | "InvalidInitSqrtPrice";

export interface Pool {
    liquidity: Liquidity;
    sqrtPrice: SqrtPrice;
    currentTickIndex: bigint;
    feeGrowthGlobalX: FeeGrowth;
    feeGrowthGlobalY: FeeGrowth;
    feeProtocolTokenX: TokenAmount;
    feeProtocolTokenY: TokenAmount;
    startTimestamp: bigint;
    lastTimestamp: bigint;
    feeReceiver: string;
    oracleInitialized: boolean;
}

export interface SingleTokenLiquidity {
    l: Liquidity;
    amount: TokenAmount;
}

export interface PoolKey {
    tokenX: string;
    tokenY: string;
    feeTier: FeeTier;
}

export interface Tick {
    index: bigint;
    sign: boolean;
    liquidityChange: Liquidity;
    liquidityGross: Liquidity;
    sqrtPrice: SqrtPrice;
    feeGrowthOutsideX: FeeGrowth;
    feeGrowthOutsideY: FeeGrowth;
    secondsOutside: bigint;
}

export interface Position {
    poolKey: PoolKey;
    liquidity: Liquidity;
    lowerTickIndex: bigint;
    upperTickIndex: bigint;
    feeGrowthInsideX: FeeGrowth;
    feeGrowthInsideY: FeeGrowth;
    lastBlockNumber: bigint;
    tokensOwedX: TokenAmount;
    tokensOwedY: TokenAmount;
}

export interface FeeGrowth {
    v: bigint;
}

export interface SecondsPerLiquidity {
    v: bigint;
}

