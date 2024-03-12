import {
  CLValue,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  GetDeployResult,
  Keys,
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
