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
import { Network } from './network'
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
    sqrtPrice,
    currentTickIndex,
    feeGrowthGlobalX,
    feeGrowthGlobalY,
    feeProtocolTokenX,
    feeProtocolTokenY,
    startTimestamp,
    lastTimestamp,
    feeReceiver,
    oracle
  }
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
  const feeBytes = bigintToByteArray(poolKey.feeTier.fee)

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
  buffor.push(feeBytes.length)
  buffor.push(...feeBytes)
  buffor.push(...[Number(poolKey.feeTier.tickSpacing), 0, 0, 0])

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
  const preparedParams = params.map(param => {
    if (typeof param === 'object') {
      return { v: param.v.toString() }
    }
    return param
  })

  const callResult = await fn(...preparedParams)

  if (typeof callResult === 'object') {
    return { v: BigInt(callResult.v) }
  }

  return callResult
}

export const integerSafeCast = (value: bigint): number => {
  if (value > BigInt(Number.MAX_SAFE_INTEGER) || value < BigInt(Number.MIN_SAFE_INTEGER)) {
    throw new Error('Integer value is outside the safe range for Numbers')
  }
  return Number(value)
}
