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
import {
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
} from 'invariant-cspr-wasm'
import { Decimals } from './schema'

const i32Parser = new CLI32BytesParser()
const u32Parser = new CLU32BytesParser()
const u64Parser = new CLU64BytesParser()
const u128Parser = new CLU128BytesParser()
const u256Parser = new CLU256BytesParser()
const accountHashParser = new CLAccountHashBytesParser()
const stringParser = new CLStringBytesParser()
const boolParser = new CLBoolBytesParser()
const optionParser = new CLOptionBytesParser()
const expectedOptionType = new CLOptionType(CLTypeBuilder.string())

export const unwrap = (value: Result<CLValue, CLErrorCodes>, err?: string) => {
  if (value.err) {
    throw new Error(err || 'Couldnt unwrap result')
  }
  return (value.val as any).data
}

export const lowerCaseFirstLetter = (v: string): string => v.charAt(0).toLowerCase() + v.slice(1)

const decodeDecimal = (
  parser: CLValueBytesParsers,
  bytes: Uint8Array,
  errorMessage?: string
): [Decimals, Uint8Array] => {
  const { remainder: valueRemainder } = stringParser.fromBytesWithRemainder(bytes)
  const { result, remainder } = parser.fromBytesWithRemainder(valueRemainder!)
  const value = BigInt(unwrap(result, errorMessage))
  return [{ v: value }, remainder!]
}

const decodeBigint = (
  parser: CLValueBytesParsers,
  bytes: Uint8Array,
  errorMessage?: string
): [bigint, Uint8Array] => {
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = BigInt(unwrap(result, errorMessage))
  return [value, remainder!]
}

export const decodeAddress = (bytes: Uint8Array): [string, Uint8Array] => {
  // One additional byte is left on the beggining of the bytes remainder
  const slicedBytes = bytes.slice(1, bytes.length)
  const { result, remainder } = accountHashParser.fromBytesWithRemainder(slicedBytes)
  const address = bytesToHex(unwrap(result, 'Couldnt parse address'))
  return [address, remainder!]
}
export const decodeString = (bytes: Uint8Array): [string, Uint8Array] => {
  const { result, remainder } = stringParser.fromBytesWithRemainder(bytes)
  const value = lowerCaseFirstLetter(unwrap(result, 'Couldnt parse string'))
  return [value, remainder!]
}

export const decodeOption = (bytes: Uint8Array): Uint8Array => {
  const { remainder } = optionParser.fromBytesWithRemainder(bytes, expectedOptionType)
  return remainder!
}
export const decodeBool = (bytes: Uint8Array): [boolean, Uint8Array] => {
  const { result, remainder } = boolParser.fromBytesWithRemainder(bytes)
  const value = unwrap(result, 'Couldnt parse bool')
  return [value, remainder!]
}

export const decodeInvariantConfig = (rawBytes: string) => {
  const bytes = parseBytes(rawBytes)
  const structNameRemainder = decodeString(bytes)[1]
  const [admin, adminRemainder] = decodeAddress(structNameRemainder)
  const [protocolFee, remainder]: [Percentage, Uint8Array] = decodeDecimal(
    u128Parser,
    adminRemainder,
    'Couldnt parse protocol fee'
  )

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

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
    const [tokenX, tokenXRemainder] = decodeAddress(remainingBytes)
    remainingBytes = tokenXRemainder
    const [tokenY, tokenYRemainder] = decodeAddress(remainingBytes)
    remainingBytes = tokenYRemainder
    remainingBytes = decodeString(remainingBytes)[1]
    const [fee, feeRemainder]: [Percentage, Uint8Array] = decodeDecimal(
      u128Parser,
      remainingBytes,
      'Couldnt parse pool key fee'
    )
    remainingBytes = feeRemainder
    const [tickSpacing, remainder] = decodeBigint(
      u32Parser,
      remainingBytes,
      'Couldnt parse pool key tickspacing'
    )
    remainingBytes = remainder
    poolKeys.push({
      tokenX,
      tokenY,
      feeTier: { fee, tickSpacing }
    })
  }

  if (remainingBytes!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

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
      'Couldnt parse fee tier fee'
    )
    remainingBytes = feeRemainder
    const [tickSpacing, remainder] = decodeBigint(
      u32Parser,
      remainingBytes,
      'Couldnt parse fee tier tickspacing'
    )
    remainingBytes = remainder
    const feeTier = { fee, tickSpacing }
    feeTiers.push(feeTier)
  }

  if (remainingBytes!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

  return feeTiers
}
export const decodePool = (rawBytes: string): Pool => {
  const bytes = parseBytes(rawBytes)
  const remainingBytes = decodeOption(bytes)
  const [liquidity, liquidityRemainder]: [Liquidity, Uint8Array] = decodeDecimal(
    u256Parser,
    remainingBytes,
    'Couldnt parse liquidity'
  )
  const [sqrtPrice, sqrtPriceRemainder]: [SqrtPrice, Uint8Array] = decodeDecimal(
    u128Parser,
    liquidityRemainder,
    'Couldnt parse sqrt price'
  )
  const [currentTickIndex, currentTickRemainder] = decodeBigint(
    i32Parser,
    sqrtPriceRemainder,
    'Couldnt parse current tick index'
  )
  const [feeGrowthGlobalX, feeGrowthGlobalXRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    currentTickRemainder,
    'Couldnt parse fee growth global x'
  )
  const [feeGrowthGlobalY, feeGrowthGlobalYRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    feeGrowthGlobalXRemainder,
    'Couldnt parse fee growth global y'
  )
  const [feeProtocolTokenX, feeProtocolTokenXRemainder]: [TokenAmount, Uint8Array] = decodeDecimal(
    u256Parser,
    feeGrowthGlobalYRemainder,
    'Couldnt parse fee protocol token x'
  )
  const [feeProtocolTokenY, feeProtocolTokenYRemainder]: [TokenAmount, Uint8Array] = decodeDecimal(
    u256Parser,
    feeProtocolTokenXRemainder,
    'Couldnt parse fee protocol token y'
  )
  const [startTimestamp, startTimestampRemainder] = decodeBigint(
    u64Parser,
    feeProtocolTokenYRemainder,
    'Couldnt parse start timestamp'
  )
  const [lastTimestamp, lastTimestampRemainder] = decodeBigint(
    u64Parser,
    startTimestampRemainder,
    'Couldnt parse last timestamp'
  )
  const [feeReceiver, remainder] = decodeAddress(lastTimestampRemainder)

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

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
  const [tokenX, tokenXRemainder] = decodeAddress(poolKeyRemainder)
  const [tokenY, tokenYRemainder] = decodeAddress(tokenXRemainder)
  const feeTierRemainder = decodeString(tokenYRemainder)[1]
  const [fee, feeRemainder]: [Percentage, Uint8Array] = decodeDecimal(
    u128Parser,
    feeTierRemainder,
    'Couldnt parse position fee'
  )
  const [tickSpacing, tickSpacingRemainder] = decodeBigint(
    u32Parser,
    feeRemainder,
    'Couldnt parse position  tickspacing'
  )
  const [liquidity, liquidityRemainder]: [Liquidity, Uint8Array] = decodeDecimal(
    u256Parser,
    tickSpacingRemainder,
    'Couldnt parse position liquidity'
  )
  const [lowerTickIndex, lowerTickIndexRemainder] = decodeBigint(
    i32Parser,
    liquidityRemainder,
    'Couldnt parse position lower tick index'
  )
  const [upperTickIndex, upperTickIndexRemainder] = decodeBigint(
    i32Parser,
    lowerTickIndexRemainder,
    'Couldnt parse position upper tick index'
  )
  const [feeGrowthInsideX, feeGrowthInsideXRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    upperTickIndexRemainder,
    'Couldnt parse position fee growth inside x'
  )
  const [feeGrowthInsideY, feeGrowthInsideYRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    feeGrowthInsideXRemainder,
    'Couldnt parse position fee growth inside y'
  )
  const [lastBlockNumber, lastBlockNumberRemainder] = decodeBigint(
    u64Parser,
    feeGrowthInsideYRemainder,
    'Couldnt parse position last block number'
  )

  const [tokensOwedX, tokenOwedXRemainder]: [TokenAmount, Uint8Array] = decodeDecimal(
    u256Parser,
    lastBlockNumberRemainder,
    'Couldnt parse position tokens owed x'
  )
  const [tokensOwedY, remainder]: [TokenAmount, Uint8Array] = decodeDecimal(
    u256Parser,
    tokenOwedXRemainder,
    'Couldnt parse position tokens owed y'
  )

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

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
  const [index, indexRemainder] = decodeBigint(
    i32Parser,
    remainingBytes,
    'Couldnt parse tick index'
  )
  const [sign, signRemainder] = decodeBool(indexRemainder)
  const [liquidityChange, liquidtyChangeRemainder]: [Liquidity, Uint8Array] = decodeDecimal(
    u256Parser,
    signRemainder,
    'Couldnt parse tick liquidity change'
  )
  const [liquidityGross, liquidityGrossRemainder]: [Liquidity, Uint8Array] = decodeDecimal(
    u256Parser,
    liquidtyChangeRemainder,
    'Couldnt parse tick liquidity gross'
  )
  const [sqrtPrice, sqrtPriceRemainder]: [SqrtPrice, Uint8Array] = decodeDecimal(
    u128Parser,
    liquidityGrossRemainder,
    'Couldnt parse tick sqrt price'
  )
  const [feeGrowthOutsideX, feeGrowthOutsideXRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    sqrtPriceRemainder,
    'Couldnt parse tick sqrt fee growth outside x'
  )
  const [feeGrowthOutsideY, feeGrowthOutsideYRemainder]: [FeeGrowth, Uint8Array] = decodeDecimal(
    u256Parser,
    feeGrowthOutsideXRemainder,
    'Couldnt parse tick fee growth outside y'
  )
  const [secondsOutside, remainder] = decodeBigint(
    u64Parser,
    feeGrowthOutsideYRemainder,
    'Couldnt parse tick seconds outside'
  )

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

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
  const [chunk, remainder] = decodeBigint(u64Parser, bytes, 'Couldnt parse chunk')

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

  return chunk
}

export const decodePositionLength = (rawBytes: string): bigint => {
  const bytes = parseBytes(rawBytes)
  const [length, remainder] = decodeBigint(u32Parser, bytes, 'Couldnt parse position length')

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

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
