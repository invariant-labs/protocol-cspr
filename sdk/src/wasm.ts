import type {
  FeeGrowth,
  FeeTier,
  FixedPoint,
  Liquidity,
  Percentage,
  PoolKey,
  SecondsPerLiquidity,
  SqrtPrice,
  TokenAmount
} from '../wasm'
import { Decimal, Decimals } from './schema'
import { callWasm, loadWasm } from './utils'

let wasmLoaded = false
let wasm: typeof import('../wasm')

const loadWasmIfNotLoaded = async () => {
  if (!wasmLoaded) {
    wasm = await loadWasm()
    wasmLoaded = true
  }

  return wasm
}

export const getMaxTick = async (tickSpacing: bigint): Promise<bigint> => {
  const wasm = await loadWasmIfNotLoaded()
  return callWasm(wasm.getMaxTick, tickSpacing)
}

export const getMinTick = async (tickSpacing: bigint): Promise<bigint> => {
  const wasm = await loadWasmIfNotLoaded()
  return callWasm(wasm.getMinTick, tickSpacing)
}

export const getGlobalMaxSqrtPrice = async (): Promise<SqrtPrice> => {
  const wasm = await loadWasmIfNotLoaded()
  return { v: await callWasm(wasm.getGlobalMaxSqrtPrice) }
}

export const getGlobalMinSqrtPrice = async (): Promise<SqrtPrice> => {
  const wasm = await loadWasmIfNotLoaded()
  return { v: await callWasm(wasm.getGlobalMinSqrtPrice) }
}

export const getMaxSqrtPrice = async (tickSpacing: bigint): Promise<SqrtPrice> => {
  const wasm = await loadWasmIfNotLoaded()
  return { v: await callWasm(wasm.getMaxSqrtPrice, tickSpacing) }
}

export const getMinSqrtPrice = async (tickSpacing: bigint): Promise<SqrtPrice> => {
  const wasm = await loadWasmIfNotLoaded()
  return { v: await callWasm(wasm.getMinSqrtPrice, tickSpacing) }
}

export const isTokenX = async (token0: string, token1: string): Promise<boolean> => {
  const wasm = await loadWasmIfNotLoaded()
  return callWasm(wasm.isTokenX, token0, token1)
}

export const newFeeTier = async (fee: Percentage, tickSpacing: bigint): Promise<FeeTier> => {
  const wasm = await loadWasmIfNotLoaded()
  return callWasm(wasm.newFeeTier, fee, tickSpacing)
}

export const newPoolKey = async (
  token0: string,
  token1: string,
  feeTier: FeeTier
): Promise<PoolKey> => {
  const wasm = await loadWasmIfNotLoaded()
  return callWasm(wasm.newPoolKey, token0, token1, feeTier)
}

export const getLiquidityByX = async (
  x: TokenAmount,
  lowerTickIndex: bigint,
  upperTickIndex: bigint,
  currentSqrtPrice: SqrtPrice,
  roundingUp: boolean
): Promise<{ l: Liquidity; amount: TokenAmount }> => {
  const wasm = await loadWasmIfNotLoaded()
  return await callWasm(
    wasm.getLiquidityByX,
    x,
    lowerTickIndex,
    upperTickIndex,
    currentSqrtPrice,
    roundingUp
  )
}

export const getLiquidityByY = async (
  y: TokenAmount,
  lowerTickIndex: bigint,
  upperTickIndex: bigint,
  currentSqrtPrice: SqrtPrice,
  roundingUp: boolean
): Promise<{ l: Liquidity; amount: TokenAmount }> => {
  const wasm = await loadWasmIfNotLoaded()
  return await callWasm(
    wasm.getLiquidityByY,
    y,
    lowerTickIndex,
    upperTickIndex,
    currentSqrtPrice,
    roundingUp
  )
}

export const toDecimal = async (
  decimal: Decimal,
  value: bigint,
  scale: bigint
): Promise<Decimals> => {
  const wasm = await loadWasmIfNotLoaded()
  switch (decimal) {
    case Decimal.Liquidity:
      return { v: await callWasm(wasm.toLiquidity, value, scale) } as Liquidity
    case Decimal.SqrtPrice:
      return { v: await callWasm(wasm.toSqrtPrice, value, scale) } as SqrtPrice
    case Decimal.TokenAmount:
      return { v: await callWasm(wasm.toTokenAmount, value, scale) } as TokenAmount
    case Decimal.FixedPoint:
      return { v: await callWasm(wasm.toFixedPoint, value, scale) } as FixedPoint
    case Decimal.Percentage:
      return { v: await callWasm(wasm.toPercentage, value, scale) } as Percentage
    case Decimal.SecondsPerLiquidity:
      return { v: await callWasm(wasm.toSecondsPerLiquidity, value, scale) } as SecondsPerLiquidity
    case Decimal.FeeGrowth:
      return { v: await callWasm(wasm.toFeeGrowth, value, scale) } as FeeGrowth
    default:
      throw new Error('Invalid decimal')
  }
}
