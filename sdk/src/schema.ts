import type {
  FeeGrowth,
  FeeTier,
  FixedPoint,
  Liquidity,
  Percentage,
  SecondsPerLiquidity,
  SqrtPrice,
  TokenAmount
} from 'invariant-cspr-wasm'

export enum Algo {
  ed25519 = 'ed25519',
  secp256K1 = 'secp256K1'
}

export enum Decimal {
  SqrtPrice = 0,
  Liquidity = 1,
  TokenAmount = 2,
  FixedPoint = 3,
  Percentage = 4,
  SecondsPerLiquidity = 5,
  FeeGrowth = 6
}

export type Decimals =
  | Liquidity
  | SqrtPrice
  | TokenAmount
  | FixedPoint
  | Percentage
  | SecondsPerLiquidity
  | FeeGrowth

export type WasmCallParams =
  | bigint
  | boolean
  | string
  | Liquidity
  | SqrtPrice
  | TokenAmount
  | FixedPoint
  | Percentage
  | SecondsPerLiquidity
  | FeeGrowth
  | FeeTier
