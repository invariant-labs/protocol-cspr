import type {
  FeeGrowth,
  FeeTier,
  FixedPoint,
  Liquidity,
  Percentage,
  SecondsPerLiquidity,
  SqrtPrice,
  TokenAmount
} from 'wasm'

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

export enum Network {
  Local = 'casper-net-1',
  Testnet = 'casper-test',
  Mainnet = 'casper'
}

export enum Key {
  Account = 0,
  Hash = 1
}

export enum DecodeError {
  DecodingI32Failed = 0,
  DecodingU32Failed = 1,
  DecodingU64Failed = 2,
  DecodingU128Failed = 3,
  DecodingU256Failed = 4,
  DecodingBoolFailed = 5,
  DecodingStringFailed = 6,
  DecodingOptionFailed = 7,
  DecodingDecimalFailed = 8,
  DecodingAddressFailed = 9,
  UnwrapFailed = 10
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
