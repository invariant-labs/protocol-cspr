import { blake2bHex } from 'blakejs'
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
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  GetDeployResult,
  Keys,
  Result,
  RuntimeArgs
} from 'casper-js-sdk'
import fs from 'fs'
import { readFile } from 'fs/promises'
import path from 'path'
import { dynamicImport } from 'tsimportlib'
import { Network } from './enums'
import { Algo, WasmCallParams } from './schema'

export const initCasperClientAndService = (nodeUrl: string) => {
  const client = new CasperClient(nodeUrl)
  const service = new CasperServiceByJsonRPC(nodeUrl)
  return { client, service }
}

export const sendTx = async (
  contract: Contracts.Contract,
  service: CasperServiceByJsonRPC,
  paymentAmount: bigint,
  account: Keys.AsymmetricKey,
  network: Network,
  entrypoint: string,
  args: Record<string, CLValue>
): Promise<GetDeployResult> => {
  const txArgs = RuntimeArgs.fromMap(args)

  const deploy = contract.callEntrypoint(
    entrypoint,
    txArgs,
    account.publicKey,
    network,
    paymentAmount.toString(),
    [account]
  )

  await service.deploy(deploy)
  return await service.waitForDeploy(deploy, 100000)
}

export const getDeploymentData = async (contractName: string): Promise<Buffer> => {
  try {
    const wasm = await readFile(`./contracts/${contractName}.wasm`)

    return wasm
  } catch (error) {
    throw new Error(`${contractName}.wasm not found.`)
  }
}

export const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

export const parseAccountKeys = (keysPath: string, algo: Algo): Keys.AsymmetricKey => {
  let accountKeys

  if (algo == Algo.ed25519) {
    accountKeys = Keys.Ed25519.loadKeyPairFromPrivateFile(`${keysPath}/private_key.pem`)
  } else if (algo == Algo.secp256K1) {
    accountKeys = Keys.Secp256K1.loadKeyPairFromPrivateFile(`${keysPath}/private_key.pem`)
  } else {
    throw new Error(`${algo} is invalid algorithm`)
  }

  return accountKeys
}

export const createAccountKeys = () => {
  const edKeyPair = Keys.Ed25519.new()
  const { publicKey } = edKeyPair

  const accountAddress = publicKey.toHex()

  const publicKeyInPem = edKeyPair.exportPublicKeyInPem()
  const privateKeyInPem = edKeyPair.exportPrivateKeyInPem()

  const folder = path.join('./', 'casper_keys')

  fs.writeFileSync(folder + '/public_key.pem', publicKeyInPem)
  fs.writeFileSync(folder + '/private_key.pem', privateKeyInPem)

  return accountAddress
}

export const hash = (input: string | Uint8Array) => {
  return blake2bHex(input, undefined, 32)
}

export const stringToUint8Array = (str: string) => {
  return new TextEncoder().encode(str)
}

export const uint8ArrayToString = (uintArray: Uint8Array) => {
  return new TextDecoder().decode(uintArray)
}

export const hexToBytes = (hex: string) => {
  return new Uint8Array(hex.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || [])
}

export const bytesToHex = (bytes: Uint8Array) => {
  return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('')
}

export const unwrap = (value: Result<CLValue, CLErrorCodes>, err?: string) => {
  if (value.err) {
    throw new Error(err || 'Couldnt unwrap result')
  }
  return (value.val as any).data
}

export const lowerCaseFirstLetter = (v: string): string => v.charAt(0).toLowerCase() + v.slice(1)

export const decodeI32 = (bytes: Uint8Array): [bigint, Uint8Array] => {
  const parser = new CLI32BytesParser()
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = BigInt(unwrap(result, 'Couldnt parse i32'))
  return [value, remainder!]
}
export const decodeU32 = (bytes: Uint8Array): [bigint, Uint8Array] => {
  const parser = new CLU32BytesParser()
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = BigInt(unwrap(result, 'Couldnt parse u32'))
  return [value, remainder!]
}
export const decodeU64 = (bytes: Uint8Array): [bigint, Uint8Array] => {
  const parser = new CLU64BytesParser()
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = BigInt(unwrap(result, 'Couldnt parse u64'))
  return [value, remainder!]
}
export const decodeU128 = (bytes: Uint8Array): [bigint, Uint8Array] => {
  const parser = new CLU128BytesParser()
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = BigInt(unwrap(result, 'Couldnt parse u128'))
  return [value, remainder!]
}
export const decodeU256 = (bytes: Uint8Array): [bigint, Uint8Array] => {
  const parser = new CLU256BytesParser()
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = BigInt(unwrap(result, 'Couldnt parse u256'))
  return [value, remainder!]
}
export const decodeAddress = (bytes: Uint8Array): [string, Uint8Array] => {
  // One additional byte is left on the beggining of the bytes remainder
  const parser = new CLAccountHashBytesParser()
  const slicedBytes = bytes.slice(1, bytes.length)
  const { result, remainder } = parser.fromBytesWithRemainder(slicedBytes)
  const address = bytesToHex(unwrap(result, 'Couldnt parse address'))
  return [address, remainder!]
}
export const decodeString = (bytes: Uint8Array): [string, Uint8Array] => {
  const parser = new CLStringBytesParser()
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = lowerCaseFirstLetter(unwrap(result, 'Couldnt parse string'))
  return [value, remainder!]
}
export const decodeOption = (bytes: Uint8Array): Uint8Array => {
  const expectedType = new CLOptionType(CLTypeBuilder.string())
  const parser = new CLOptionBytesParser()
  const { remainder } = parser.fromBytesWithRemainder(bytes, expectedType)
  return remainder!
}
export const decodeBool = (bytes: Uint8Array): [boolean, Uint8Array] => {
  const parser = new CLBoolBytesParser()
  const { result, remainder } = parser.fromBytesWithRemainder(bytes)
  const value = unwrap(result, 'Couldnt parse bool')
  return [value, remainder!]
}

export const decodeInvariantConfig = (rawBytes: any) => {
  const bytes = parseBytes(rawBytes)
  const structNameRemainder = decodeString(bytes)[1]
  const [admin, adminRemainder] = decodeAddress(structNameRemainder)
  const percentageStructNameRemainder = decodeString(adminRemainder)[1]
  const [protocolFee, remainder] = decodeU128(percentageStructNameRemainder)

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

  return {
    admin,
    protocolFee
  }
}

export const decodePoolKeys = (rawBytes: any) => {
  const bytes = parseBytes(rawBytes)
  const stringRemainder = decodeString(bytes)[1]

  const result = decodeU32(stringRemainder)
  const poolKeyCount = result[0]
  let remainingBytes = result[1]
  const poolKeys = []
  console.log(poolKeyCount)
  for (let i = 0; i < poolKeyCount; i++) {
    remainingBytes = decodeString(remainingBytes)[1]
    const [tokenX, tokenXRemainder] = decodeAddress(remainingBytes)
    remainingBytes = tokenXRemainder
    const [tokenY, tokenYRemainder] = decodeAddress(remainingBytes)
    remainingBytes = tokenYRemainder
    remainingBytes = decodeString(remainingBytes)[1]
    remainingBytes = decodeString(remainingBytes)[1]
    const [fee, feeRemainder] = decodeU128(remainingBytes)
    remainingBytes = feeRemainder
    const [tickSpacing, remainder] = decodeU32(remainingBytes)
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
export const decodeFeeTiers = (rawBytes: any) => {
  const bytes = parseBytes(rawBytes)
  const stringRemainder = decodeString(bytes)[1]

  const result = decodeU32(stringRemainder)
  const feeTierCount = result[0]
  let remainingBytes = result[1]
  const feeTiers = []

  for (let i = 0; i < feeTierCount; i++) {
    remainingBytes = decodeString(remainingBytes)[1]
    const [feeType, feeTypeRemainder] = decodeString(remainingBytes)
    remainingBytes = feeTypeRemainder
    const [fee, feeRemainder] = decodeU128(remainingBytes)
    remainingBytes = feeRemainder
    const [tickSpacing, remainder] = decodeU32(remainingBytes)
    remainingBytes = remainder

    const feeTier = { [feeType]: fee, tickSpacing }
    feeTiers.push(feeTier)
  }

  if (remainingBytes!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

  return feeTiers
}
export const decodePool = (rawBytes: any) => {
  const bytes = parseBytes(rawBytes)
  const remainingBytes = decodeOption(bytes)
  const liquidityTypeRemainder = decodeString(remainingBytes)[1]
  const [liquidity, liquidityRemainder] = decodeU256(liquidityTypeRemainder)
  const sqrtPriceTypeRemainder = decodeString(liquidityRemainder)[1]
  const [sqrtPrice, sqrtPriceRemainder] = decodeU256(sqrtPriceTypeRemainder)
  const [currentTickIndex, currentTickRemainder] = decodeI32(sqrtPriceRemainder)
  const feeGrowthGlobalXTypeRemainder = decodeString(currentTickRemainder)[1]
  const [feeGrowthGlobalX, feeGrowthGlobalXRemainder] = decodeU256(feeGrowthGlobalXTypeRemainder)
  const feeGrowthGlobalYTypeRemainder = decodeString(feeGrowthGlobalXRemainder)[1]
  const [feeGrowthGlobalY, feeGrowthGlobalYRemainder] = decodeU256(feeGrowthGlobalYTypeRemainder)
  const feeProtocolTokenXTypeRemainder = decodeString(feeGrowthGlobalYRemainder)[1]
  const [feeProtocolTokenX, feeProtocolTokenXRemainder] = decodeU256(feeProtocolTokenXTypeRemainder)
  const feeProtocolTokenYTypeRemainder = decodeString(feeProtocolTokenXRemainder)[1]
  const [feeProtocolTokenY, feeProtocolTokenYRemainder] = decodeU256(feeProtocolTokenYTypeRemainder)
  const [startTimestamp, startTimestampRemainder] = decodeU64(feeProtocolTokenYRemainder)
  const [lastTimestamp, lastTimestampRemainder] = decodeU64(startTimestampRemainder)
  const [feeReceiver, feeReceiverRemainder] = decodeAddress(lastTimestampRemainder)
  const [oracle, remainder] = decodeBool(feeReceiverRemainder)

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

  return {
    liquidity,
    sqrtPrice: { v: sqrtPrice },
    currentTickIndex,
    feeGrowthGlobalX: { v: feeGrowthGlobalX },
    feeGrowthGlobalY: { v: feeGrowthGlobalY },
    feeProtocolTokenX: { v: feeProtocolTokenX },
    feeProtocolTokenY: { v: feeProtocolTokenY },
    startTimestamp,
    lastTimestamp,
    feeReceiver,
    oracle
  }
}

export const decodePosition = (rawBytes: any) => {
  const bytes = parseBytes(rawBytes)
  const remainingBytes = decodeOption(bytes)
  const poolKeyRemainder = decodeString(remainingBytes)[1]
  const [tokenX, tokenXRemainder] = decodeAddress(poolKeyRemainder)
  const [tokenY, tokenYRemainder] = decodeAddress(tokenXRemainder)
  const feeTierRemainder = decodeString(tokenYRemainder)[1]
  const percentageRemainder = decodeString(feeTierRemainder)[1]
  const [fee, feeRemainder] = decodeU128(percentageRemainder)
  const [tickSpacing, tickSpacingRemainder] = decodeU32(feeRemainder)
  const liquidityTypeRemainder = decodeString(tickSpacingRemainder)[1]
  const [liquidity, liquidityRemainder] = decodeU256(liquidityTypeRemainder)
  const [lowerTickIndex, lowerTickIndexRemainder] = decodeI32(liquidityRemainder)
  const [upperTickIndex, upperTickIndexRemainder] = decodeI32(lowerTickIndexRemainder)
  const feeGrowthXTypeRemainder = decodeString(upperTickIndexRemainder)[1]
  const [feeGrowthInsideX, feeGrowthInsideXRemainder] = decodeU256(feeGrowthXTypeRemainder)
  const feeGrowthYTypeRemainder = decodeString(feeGrowthInsideXRemainder)[1]
  const [feeGrowthInsideY, feeGrowthInsideYRemainder] = decodeU256(feeGrowthYTypeRemainder)
  const [lastBlockNumber, lastBlockNumberRemainder] = decodeU64(feeGrowthInsideYRemainder)
  const tokensOwedXTypeRemainder = decodeString(lastBlockNumberRemainder)[1]
  const [tokensOwedX, tokenOwedXRemainder] = decodeU256(tokensOwedXTypeRemainder)
  const tokensOwedYTypeRemainder = decodeString(tokenOwedXRemainder)[1]
  const [tokensOwedY, remainder] = decodeU256(tokensOwedYTypeRemainder)

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

  return {
    poolKey: {
      tokenX,
      tokenY,
      feeTier: {
        fee: { v: fee },
        tickSpacing
      }
    },
    liquidity: { v: liquidity },
    lowerTickIndex,
    upperTickIndex,
    feeGrowthInsideX: { v: feeGrowthInsideX },
    feeGrowthInsideY: { v: feeGrowthInsideY },
    lastBlockNumber,
    tokensOwedX: { v: tokensOwedX },
    tokensOwedY: { v: tokensOwedY }
  }
}

export const decodeTick = (rawBytes: any) => {
  const bytes = parseBytes(rawBytes)
  const remainingBytes = decodeOption(bytes)
  const [index, indexRemainder] = decodeI32(remainingBytes)
  const [sign, signRemainder] = decodeBool(indexRemainder)
  const liquidityChangeTypeRemainder = decodeString(signRemainder)[1]
  const [liquidityChange, liquidtyChangeRemainder] = decodeU128(liquidityChangeTypeRemainder)
  const liquidityGrossTypeRemainder = decodeString(liquidtyChangeRemainder)[1]
  const [liquidityGross, liquidityGrossRemainder] = decodeU128(liquidityGrossTypeRemainder)
  const sqrtPriceTypeRemainder = decodeString(liquidityGrossRemainder)[1]
  const [sqrtPrice, sqrtPriceRemainder] = decodeU256(sqrtPriceTypeRemainder)
  const feeGrowthOutsideXTypeRemainder = decodeString(sqrtPriceRemainder)[1]
  const [feeGrowthOutsideX, feeGrowthOutsideXRemainder] = decodeU256(feeGrowthOutsideXTypeRemainder)
  const feeGrowthOutsideYTypeRemainder = decodeString(feeGrowthOutsideXRemainder)[1]
  const [feeGrowthOutsideY, feeGrowthOutsideYRemainder] = decodeU256(feeGrowthOutsideYTypeRemainder)
  const [secondsOutside, remainder] = decodeU64(feeGrowthOutsideYRemainder)

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

export const decodeChunk = (rawBytes: any) => {
  const bytes = parseBytes(rawBytes)
  const [chunk, remainder] = decodeU64(bytes)

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

  return chunk
}

export const decodePositionLength = (rawBytes: any) => {
  const bytes = parseBytes(rawBytes)
  const [length, remainder] = decodeU32(bytes)

  if (remainder!.length != 0) {
    throw new Error('There are remaining bytes left')
  }

  return length
}
export const parseBytes = (rawBytes: any): Uint8Array => {
  return new Uint8Array(
    String(rawBytes)
      .match(/.{1,2}/g)!
      .map((byte: string) => parseInt(byte, 16))
  )
}

export const encodePoolKey = (poolKey: any): number[] => {
  const buffor: number[] = []
  const poolKeyStructBytes = 'PoolKey'.split('').map(c => c.charCodeAt(0))
  const tokenXBytes = hexToBytes(poolKey.tokenX)
  const tokenYBytes = hexToBytes(poolKey.tokenY)
  const feeTierStructBytes = 'FeeTier'.split('').map(c => c.charCodeAt(0))
  const percentageSturctBytes = 'Percentage'.split('').map(c => c.charCodeAt(0))
  const feeBytes = bigintToByteArray(poolKey.feeTier.fee.v)

  buffor.push(7, 0, 0, 0)
  buffor.push(...poolKeyStructBytes)
  buffor.push(1)
  buffor.push(...tokenXBytes)
  buffor.push(1)
  buffor.push(...tokenYBytes)
  buffor.push(7, 0, 0, 0)
  buffor.push(...feeTierStructBytes)
  buffor.push(10, 0, 0, 0)
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

  while (bigintValue > 0n) {
    byteArray.unshift(Number(bigintValue & 0xffn))
    bigintValue >>= 8n
  }

  if (byteArray.length === 0) {
    byteArray.push(0)
  }

  return byteArray.reverse()
}

export const loadWasm = async () => {
  return (await dynamicImport(
    'invariant-cspr-wasm',
    module
  )) as typeof import('invariant-cspr-wasm')
}

export const callWasm = async (
  fn: Promise<any> | any,
  ...params: WasmCallParams[]
): Promise<any> => {
  const preparedParams = params.map(param => prepareWasmParms(param))
  const callResult = await fn(...preparedParams)
  return parse(callResult)
}

export const prepareWasmParms = (
  value: any,
  stringify: boolean = false,
  numberize: boolean = false
) => {
  if (isArray(value)) {
    return value.map((element: any) => prepareWasmParms(element))
  }

  if (isObject(value)) {
    const newValue: { [key: string]: any } = {}

    Object.entries(value as { [key: string]: any }).forEach(([key, value]) => {
      if (key === 'v') {
        newValue[key] = prepareWasmParms(value, true)
      } else {
        newValue[key] = prepareWasmParms(value, false, true)
      }
    })

    return newValue
  }

  if (isBoolean(value)) {
    return value
  }

  try {
    if (stringify) {
      return value.toString()
    } else if (numberize) {
      return integerSafeCast(value)
    } else {
      return value
    }
  } catch (e) {
    return value
  }
}

export const parse = (value: any) => {
  if (isArray(value)) {
    return value.map((element: any) => parse(element))
  }

  if (isObject(value)) {
    const newValue: { [key: string]: any } = {}

    Object.entries(value as { [key: string]: any }).forEach(([key, value]) => {
      newValue[key] = parse(value)
    })

    return newValue
  }

  if (isBoolean(value)) {
    return value
  }

  try {
    return BigInt(value)
  } catch (e) {
    return value
  }
}

const isBoolean = (value: any): boolean => {
  return typeof value === 'boolean'
}

const isArray = (value: any): boolean => {
  return Array.isArray(value)
}

const isObject = (value: any): boolean => {
  return typeof value === 'object' && value !== null
}

export const integerSafeCast = (value: bigint): number => {
  if (value > BigInt(Number.MAX_SAFE_INTEGER) || value < BigInt(Number.MIN_SAFE_INTEGER)) {
    throw new Error('Integer value is outside the safe range for Numbers')
  }
  return Number(value)
}

export const getBitAtIndex = (v: bigint, index: bigint): boolean => {
  const binary = v.toString(2)
  const reversedBinaryString = binary.split('').reverse().join('')
  const bitAtIndex = reversedBinaryString[integerSafeCast(index)]
  return bitAtIndex === '1'
}
