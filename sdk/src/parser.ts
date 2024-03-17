import { blake2bHex } from 'blakejs'
import type { PoolKey } from 'wasm'
import { Key } from './schema'
import { integerSafeCast } from './utils'

export const poolKeyPrefixBytes = [7, 0, 0, 0]
export const feeTierPrefixBytes = [7, 0, 0, 0]
export const percentagePrefixBytes = [10, 0, 0, 0]

export const encodePoolKey = (poolKey: PoolKey): number[] => {
  const buffor: number[] = []
  const poolKeyStructBytes = encodeString('PoolKey')
  const tokenXBytes = hexToBytes(poolKey.tokenX)
  const tokenYBytes = hexToBytes(poolKey.tokenY)
  const feeTierStructBytes = encodeString('FeeTier')
  const percentageSturctBytes = encodeString('Percentage')
  const feeBytes = bigintToByteArray(poolKey.feeTier.fee.v)

  buffor.push(...poolKeyPrefixBytes)
  buffor.push(...poolKeyStructBytes)
  buffor.push(...[Key.Hash])
  buffor.push(...tokenXBytes)
  buffor.push(...[Key.Hash])
  buffor.push(...tokenYBytes)
  buffor.push(...feeTierPrefixBytes)
  buffor.push(...feeTierStructBytes)
  buffor.push(...percentagePrefixBytes)
  buffor.push(...percentageSturctBytes)
  if (poolKey.feeTier.fee.v > 0) {
    buffor.push(feeBytes.length)
  }
  buffor.push(...feeBytes)
  buffor.push(...[integerSafeCast(poolKey.feeTier.tickSpacing), 0, 0, 0])

  return buffor
}

export const bigintToByteArray = (bigintValue: bigint): number[] => {
  const byteArray: number[] = []

  const isNegative = bigintValue < 0n

  if (isNegative) {
    bigintValue = -bigintValue
  }

  while (bigintValue > 0n) {
    byteArray.unshift(integerSafeCast(bigintValue & 0xffn))
    bigintValue >>= 8n
  }

  if (byteArray.length === 0) {
    byteArray.push(0)
  }

  if (isNegative) {
    const reversed = byteArray.reverse()
    const flipped = reversed.map(byte => 256 - byte)
    return flipped
  } else {
    return byteArray.reverse()
  }
}
export const encodeString = (str: string): number[] => {
  const bytes = str.split('').map(c => c.charCodeAt(0))
  return bytes
}

export const stringToUint8Array = (str: string) => {
  return new TextEncoder().encode(str)
}

export const hexToBytes = (hex: string) => {
  return new Uint8Array(hex.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || [])
}

export const hash = (input: string | Uint8Array) => {
  return blake2bHex(input, undefined, 32)
}
