import {
  FeeTier,
  Liquidity,
  Percentage,
  PoolKey,
  SqrtPrice,
  TokenAmount
} from 'invariant-cspr-wasm'
import { callWasm, loadWasm } from './utils'

let wasmLoaded = false
let wasm: typeof import('invariant-cspr-wasm')

const loadWasmIfNotLoaded = async () => {
  if (!wasmLoaded) {
    wasm = await loadWasm()
    wasmLoaded = true
  }

  return wasm
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
  const wasm = await loadWasm()
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
  const wasm = await loadWasm()
  return await callWasm(
    wasm.getLiquidityByY,
    y,
    lowerTickIndex,
    upperTickIndex,
    currentSqrtPrice,
    roundingUp
  )
}
