import {
  CLAccountHashBytesParser,
  CLBoolBytesParser,
  CLErrorCodes,
  CLI32BytesParser,
  CLOptionBytesParser,
  CLOptionType,
  CLStringBytesParser,
  CLTypeBuilder,
  CLU128BytesParser,
  CLU256BytesParser,
  CLU32BytesParser,
  CLU64BytesParser,
  CLValue,
  CLValueBytesParsers,
  Result
} from 'casper-js-sdk'
import type {
  FeeGrowth,
  FeeTier,
  Liquidity,
  Percentage,
  Pool,
  PoolKey,
  Position,
  SqrtPrice,
  Tick,
  TokenAmount
} from '../wasm'
import { Decimals, DecodeError } from './schema'

export const i32Parser = new CLI32BytesParser()
export const u32Parser = new CLU32BytesParser()
export const u64Parser = new CLU64BytesParser()
export const u128Parser = new CLU128BytesParser()
export const u256Parser = new CLU256BytesParser()
export const accountHashParser = new CLAccountHashBytesParser()
export const stringParser = new CLStringBytesParser()
export const boolParser = new CLBoolBytesParser()
export const optionParser = new CLOptionBytesParser()
export const expectedOptionType = new CLOptionType(CLTypeBuilder.string())

export const unwrap = (value: Result<CLValue, CLErrorCodes>, err?: DecodeError) => {
  if (value.err) {
    throw new Error((err || DecodeError.UnwrapFailed).toString())
  }
  return (value.val as any).data
}

export const lowerCaseFirstLetter = (v: string): string => v.charAt(0).toLowerCase() + v.slice(1)

const decodeDecimal = (
  parser: CLValueBytesParsers,
  bytes: Uint8Array,
  errorMessage?: DecodeError
): [Decimals, Uint8Array] => {
  const { remainder: valueRemainder } = stringParser.fromBytesWithRemainder(bytes)
  const { result, remainder } = parser.fromBytesWithRemainder(valueRemainder!)
  const value = BigInt(unwrap(result, errorMessage))
  return [{ v: value }, remainder!]
}

const decodeBigint = (
  parser: CLValueBytesParsers,
  bytes: Uint8Array,
  errorMessage?: DecodeError
): [bigint, Uint8Array] => {
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = BigInt(unwrap(result, errorMessage))
  return [value, remainder!]
}

export const decodeAddress = (bytes: Uint8Array): [string, Uint8Array] => {
  // One additional byte is indicating if it is an Account or Contract
  const slicedBytes = bytes.slice(1, bytes.length)
  const { result, remainder } = accountHashParser.fromBytesWithRemainder(slicedBytes)
  const address = bytesToHex(unwrap(result, DecodeError.DecodingAddressFailed))
  return [address, remainder!]
}
export const decodeString = (bytes: Uint8Array): [string, Uint8Array] => {
  const { result, remainder } = stringParser.fromBytesWithRemainder(bytes)
  const value = lowerCaseFirstLetter(unwrap(result, DecodeError.DecodingStringFailed))
  return [value, remainder!]
}

export const decodeOption = (bytes: Uint8Array): Uint8Array => {
  const { remainder } = optionParser.fromBytesWithRemainder(bytes, expectedOptionType)
  return remainder!
}
export const decodeBool = (bytes: Uint8Array): [boolean, Uint8Array] => {
  const { result, remainder } = boolParser.fromBytesWithRemainder(bytes)
  const value = unwrap(result, DecodeError.DecodingBoolFailed)
  return [value, remainder!]
}

export const decodeInvariantConfig = (rawBytes: string) => {
  const bytes = parseBytes(rawBytes)
  const structNameRemainder = decodeString(bytes)[1]
  const [admin, adminRemainder]: [string, Uint8Array] = decodeAddress(structNameRemainder)
  const [protocolFee, remainder]: [Percentage, Uint8Array] = decodeDecimal(
    u128Parser,
    adminRemainder,
    DecodeError.DecodingDecimalFailed
  )

  assertBytes(remainder)

  return {
    admin,
    protocolFee: protocolFee
  }
}

export const decodePoolKeys = (rawBytes: string): PoolKey[] => {
  const bytes = parseBytes(rawBytes)
  const stringRemainder = decodeString(bytes)[1]

  const result = decodeBigint(u32Parser, stringRemainder)
  const poolKeyCount = result[0]
  let remainingBytes = result[1]
  const poolKeys = []
  for (let i = 0; i < poolKeyCount; i++) {
    remainingBytes = decodeString(remainingBytes)[1]
    const [tokenX, tokenXRemainder]: [string, Uint8Array] = decodeAddress(remainingBytes)
    remainingBytes = tokenXRemainder
    const [tokenY, tokenYRemainder]: [string, Uint8Array] = decodeAddress(remainingBytes)
    remainingBytes = tokenYRemainder
    remainingBytes = decodeString(remainingBytes)[1]
    const [fee, feeRemainder]: [Percentage, Uint8Array] = decodeDecimal(
      u128Parser,
      remainingBytes,
      DecodeError.DecodingDecimalFailed
    )
    remainingBytes = feeRemainder
    const [tickSpacing, remainder]: [bigint, Uint8Array] = decodeBigint(
      u32Parser,
      remainingBytes,
      DecodeError.DecodingU32Failed
    )
    remainingBytes = remainder
    poolKeys.push({
      tokenX,
      tokenY,
      feeTier: { fee, tickSpacing }
    })
  }

  assertBytes(remainingBytes)

  return poolKeys
}
export const decodeFeeTiers = (rawBytes: string): FeeTier[] => {
  const bytes = parseBytes(rawBytes)
  const stringRemainder = decodeString(bytes)[1]

  const result = decodeBigint(u32Parser, stringRemainder)
  const feeTierCount = result[0]
  let remainingBytes = result[1]
  const feeTiers: FeeTier[] = []

  for (let i = 0; i < feeTierCount; i++) {
    remainingBytes = decodeString(remainingBytes)[1]
    const [fee, feeRemainder]: [Percentage, Uint8Array] = decodeDecimal(
      u128Parser,
      remainingBytes,
      DecodeError.DecodingDecimalFailed
    )
    remainingBytes = feeRemainder
    const [tickSpacing, remainder]: [bigint, Uint8Array] = decodeBigint(
      u32Parser,
      remainingBytes,
      DecodeError.DecodingU32Failed
    )
    remainingBytes = remainder
    const feeTier = { fee, tickSpacing }
    feeTiers.push(feeTier)
  }

  assertBytes(remainingBytes)

  return feeTiers
}
export const decodePool = (rawBytes: string): Pool => {
  const bytes = parseBytes(rawBytes)
  const remainingBytes = decodeOption(bytes)
  const [liquidity, liquidityRemainder]: [Liquidity, Uint8Array] = decodeDecimal(
    u256Parser,
    remainingBytes,
    DecodeError.DecodingDecimalFailed
  )
  const [sqrtPrice, sqrtPriceRemainder]: [SqrtPrice, Uint8Array] = decodeDecimal(
    u128Parser,
    liquidityRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [currentTickIndex, currentTickRemainder]: [bigint, Uint8Array] = decodeBigint(
    i32Parser,
    sqrtPriceRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [feeGrowthGlobalX, feeGrowthGlobalXRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    currentTickRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [feeGrowthGlobalY, feeGrowthGlobalYRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    feeGrowthGlobalXRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [feeProtocolTokenX, feeProtocolTokenXRemainder]: [TokenAmount, Uint8Array] = decodeDecimal(
    u256Parser,
    feeGrowthGlobalYRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [feeProtocolTokenY, feeProtocolTokenYRemainder]: [TokenAmount, Uint8Array] = decodeDecimal(
    u256Parser,
    feeProtocolTokenXRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [startTimestamp, startTimestampRemainder]: [bigint, Uint8Array] = decodeBigint(
    u64Parser,
    feeProtocolTokenYRemainder,
    DecodeError.DecodingU64Failed
  )
  const [lastTimestamp, lastTimestampRemainder]: [bigint, Uint8Array] = decodeBigint(
    u64Parser,
    startTimestampRemainder,
    DecodeError.DecodingU64Failed
  )
  const [feeReceiver, remainder] = decodeAddress(lastTimestampRemainder)
  assertBytes(remainder)

  return {
    liquidity,
    sqrtPrice,
    currentTickIndex,
    feeGrowthGlobalX,
    feeGrowthGlobalY,
    feeProtocolTokenX,
    feeProtocolTokenY,
    startTimestamp,
    lastTimestamp,
    feeReceiver
  }
}

export const decodePosition = (rawBytes: string): Position => {
  const bytes = parseBytes(rawBytes)
  const remainingBytes = decodeOption(bytes)
  const poolKeyRemainder = decodeString(remainingBytes)[1]
  const [tokenX, tokenXRemainder]: [string, Uint8Array] = decodeAddress(poolKeyRemainder)
  const [tokenY, tokenYRemainder]: [string, Uint8Array] = decodeAddress(tokenXRemainder)
  const feeTierRemainder = decodeString(tokenYRemainder)[1]
  const [fee, feeRemainder]: [Percentage, Uint8Array] = decodeDecimal(
    u128Parser,
    feeTierRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [tickSpacing, tickSpacingRemainder]: [bigint, Uint8Array] = decodeBigint(
    u32Parser,
    feeRemainder,
    DecodeError.DecodingU64Failed
  )
  const [liquidity, liquidityRemainder]: [Liquidity, Uint8Array] = decodeDecimal(
    u256Parser,
    tickSpacingRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [lowerTickIndex, lowerTickIndexRemainder]: [bigint, Uint8Array] = decodeBigint(
    i32Parser,
    liquidityRemainder,
    DecodeError.DecodingI32Failed
  )
  const [upperTickIndex, upperTickIndexRemainder]: [bigint, Uint8Array] = decodeBigint(
    i32Parser,
    lowerTickIndexRemainder,
    DecodeError.DecodingI32Failed
  )
  const [feeGrowthInsideX, feeGrowthInsideXRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    upperTickIndexRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [feeGrowthInsideY, feeGrowthInsideYRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    feeGrowthInsideXRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [lastBlockNumber, lastBlockNumberRemainder]: [bigint, Uint8Array] = decodeBigint(
    u64Parser,
    feeGrowthInsideYRemainder,
    DecodeError.DecodingU64Failed
  )

  const [tokensOwedX, tokenOwedXRemainder]: [TokenAmount, Uint8Array] = decodeDecimal(
    u256Parser,
    lastBlockNumberRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [tokensOwedY, remainder]: [TokenAmount, Uint8Array] = decodeDecimal(
    u256Parser,
    tokenOwedXRemainder,
    DecodeError.DecodingDecimalFailed
  )

  assertBytes(remainder)

  return {
    poolKey: {
      tokenX,
      tokenY,
      feeTier: {
        fee,
        tickSpacing
      }
    },
    liquidity,
    lowerTickIndex,
    upperTickIndex,
    feeGrowthInsideX,
    feeGrowthInsideY,
    lastBlockNumber,
    tokensOwedX,
    tokensOwedY
  }
}

export const decodeTick = (rawBytes: string): Tick => {
  const bytes = parseBytes(rawBytes)
  const remainingBytes = decodeOption(bytes)
  const [index, indexRemainder]: [bigint, Uint8Array] = decodeBigint(
    i32Parser,
    remainingBytes,
    DecodeError.DecodingI32Failed
  )
  const [sign, signRemainder]: [boolean, Uint8Array] = decodeBool(indexRemainder)
  const [liquidityChange, liquidtyChangeRemainder]: [Liquidity, Uint8Array] = decodeDecimal(
    u256Parser,
    signRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [liquidityGross, liquidityGrossRemainder]: [Liquidity, Uint8Array] = decodeDecimal(
    u256Parser,
    liquidtyChangeRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [sqrtPrice, sqrtPriceRemainder]: [SqrtPrice, Uint8Array] = decodeDecimal(
    u128Parser,
    liquidityGrossRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [feeGrowthOutsideX, feeGrowthOutsideXRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    sqrtPriceRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [feeGrowthOutsideY, feeGrowthOutsideYRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    feeGrowthOutsideXRemainder,
    DecodeError.DecodingDecimalFailed
  )
  const [secondsOutside, remainder]: [bigint, Uint8Array] = decodeBigint(
    u64Parser,
    feeGrowthOutsideYRemainder,
    DecodeError.DecodingU64Failed
  )

  assertBytes(remainder)

  return {
    index,
    sign,
    liquidityChange,
    liquidityGross,
    sqrtPrice,
    feeGrowthOutsideX,
    feeGrowthOutsideY,
    secondsOutside
  }
}

export const decodeChunk = (rawBytes: string): bigint => {
  const bytes = parseBytes(rawBytes)
  const [chunk, remainder]: [bigint, Uint8Array] = decodeBigint(
    u64Parser,
    bytes,
    DecodeError.DecodingU64Failed
  )

  assertBytes(remainder)

  return chunk
}

export const decodePositionLength = (rawBytes: string): bigint => {
  const bytes = parseBytes(rawBytes)
  const [length, remainder]: [bigint, Uint8Array] = decodeBigint(
    u32Parser,
    bytes,
    DecodeError.DecodingU32Failed
  )

  assertBytes(remainder)

  return length
}
export const parseBytes = (rawBytes: string): Uint8Array => {
  return new Uint8Array(
    String(rawBytes)
      .match(/.{1,2}/g)!
      .map((byte: string) => parseInt(byte, 16))
  )
}

export const bytesToHex = (bytes: Uint8Array) => {
  return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('')
}
export const uint8ArrayToString = (uintArray: Uint8Array) => {
  return new TextDecoder().decode(uintArray)
}

const assertBytes = (bytes: Uint8Array) => {
  if (bytes.length !== 0) {
    throw new Error('There are remaing bytes left')
  }
}
