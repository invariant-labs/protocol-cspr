import {
  CLValue,
  CasperClient,
  CasperServiceByJsonRPC,
  ContractPackageJson,
  Contracts,
  GetDeployResult,
  Keys,
  RuntimeArgs
} from 'casper-js-sdk'
import fs from 'fs'
import { readFile } from 'fs/promises'
import path from 'path'
import { dynamicImport } from 'tsimportlib'
import type {
  FeeTier,
  Percentage,
  Pool,
  PoolKey,
  Position,
  Price,
  SqrtPrice,
  Tick,
  TokenAmount
} from '../wasm'
import { Algo, Network, WasmCallParams } from './schema'
import { isTokenX } from './wasm'

export const initCasperClient = (nodeUrl: string) => {
  return new CasperClient(nodeUrl)
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

export const loadWasm = async () => {
  return (await dynamicImport('invariant-cspr-wasm', module)) as typeof import('../wasm')
}

export const orderTokens = async (
  token0ContractPackage: string,
  token1ContractPackage: string,
  token0ContractHash: string,
  token1Contracthash: string
): Promise<[string, string]> => {
  return (await isTokenX(token0ContractPackage, token1ContractPackage))
    ? [token0ContractHash, token1Contracthash]
    : [token1Contracthash, token0ContractHash]
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
  return Boolean((v >> index) & BigInt(1))
}

export const calculateSqrtPriceAfterSlippage = async (
  sqrtPrice: SqrtPrice,
  slippage: Percentage,
  up: boolean
): Promise<SqrtPrice> => {
  const wasm = await loadWasm()
  const percentageDenominator = await callWasm(wasm.getPercentageDenominator)
  const multiplier = percentageDenominator + (up ? slippage.v : -slippage.v)
  const price = await sqrtPriceToPrice(sqrtPrice)
  const priceWithSlippage = price.v * multiplier * percentageDenominator
  const sqrtPriceWithSlippage =
    (await priceToSqrtPrice({ v: priceWithSlippage })).v / percentageDenominator

  return { v: sqrtPriceWithSlippage }
}

export const calculatePriceImpact = async (
  startingSqrtPrice: SqrtPrice,
  endingSqrtPrice: SqrtPrice
): Promise<Percentage> => {
  const wasm = await loadWasm()

  const startingPrice = startingSqrtPrice.v * startingSqrtPrice.v
  const endingPrice = endingSqrtPrice.v * endingSqrtPrice.v
  const diff = startingPrice - endingPrice

  const nominator = diff > 0n ? diff : -diff
  const denominator = startingPrice > endingPrice ? startingPrice : endingPrice

  return { v: (nominator * (await callWasm(wasm.getPercentageDenominator))) / denominator }
}

export const sqrtPriceToPrice = async (sqrtPrice: SqrtPrice): Promise<Price> => {
  const wasm = await loadWasm()
  const sqrtPriceDenominator = (await callWasm(wasm.getSqrtPriceDenominator)) as bigint
  return { v: (sqrtPrice.v * sqrtPrice.v) / sqrtPriceDenominator }
}

export const priceToSqrtPrice = async (price: Price): Promise<SqrtPrice> => {
  const wasm = await loadWasm()

  return { v: sqrt(price.v * (await callWasm(wasm.getSqrtPriceDenominator))) }
}

const sqrt = (value: bigint): bigint => {
  if (value < 0n) {
    throw 'square root of negative numbers is not supported'
  }

  if (value < 2n) {
    return value
  }

  return newtonIteration(value, 1n)
}

const newtonIteration = (n: bigint, x0: bigint): bigint => {
  const x1 = (n / x0 + x0) >> 1n
  if (x0 === x1 || x0 === x1 - 1n) {
    return x0
  }
  return newtonIteration(n, x1)
}

export const calculateFee = async (
  pool: Pool,
  position: Position,
  lowerTick: Tick,
  upperTick: Tick
): Promise<[TokenAmount, TokenAmount]> => {
  const wasm = await loadWasm()
  return await callWasm(
    wasm._calculateFee,
    lowerTick.index,
    lowerTick.feeGrowthOutsideX,
    lowerTick.feeGrowthOutsideY,
    upperTick.index,
    upperTick.feeGrowthOutsideX,
    upperTick.feeGrowthOutsideY,
    pool.currentTickIndex,
    pool.feeGrowthGlobalX,
    pool.feeGrowthGlobalY,
    position.feeGrowthInsideX,
    position.feeGrowthInsideY,
    position.liquidity
  )
}

export const findContractPackageHash = (account: any, name: string) => {
  const contractPackageHash = account.namedKeys.find((i: any) => i.name === name)?.key
  return contractPackageHash
}
export const extractContractHash = (contractPackageHash: string): string => {
  return contractPackageHash.replace('hash-', '')
}
export const extractContractPackageHash = (contractPackage: ContractPackageJson): string => {
  return contractPackage.versions[0].contractHash.replace('contract-', '')
}

export const getAccountHashFromKey = (key: Keys.AsymmetricKey): string => {
  return key.publicKey.toAccountHashStr().replace('account-hash-', '')
}

export const createFeeTier = async (fee: Percentage, tickSpacing: bigint): Promise<FeeTier> => {
  const wasm = await loadWasm()
  return (await callWasm(wasm.newFeeTier, fee, tickSpacing)) as FeeTier
}

export const createPoolKey = async (
  tokenXHash: string,
  tokenYHash: string,
  feeTier: FeeTier
): Promise<PoolKey> => {
  const wasm = await loadWasm()
  return (await callWasm(wasm.newPoolKey, tokenXHash, tokenYHash, feeTier)) as PoolKey
}
