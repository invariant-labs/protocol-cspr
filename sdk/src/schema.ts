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
