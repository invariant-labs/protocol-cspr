import { blake2bHex } from 'blakejs'
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
import { Network } from './network'

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

export const parseAccountKeys = (keysPath: string, algo: string): Keys.AsymmetricKey => {
  let accountKeys

  if (algo == 'ed25519') {
    accountKeys = Keys.Ed25519.loadKeyPairFromPrivateFile(`${keysPath}/private_key.pem`)
  } else if (algo == 'secp256K1') {
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
  const bytes = []

  for (let c = 0; c < hex.length; c += 2) {
    bytes.push(parseInt(hex.substr(c, 2), 16))
  }

  return bytes
}

export const bytesToHex = (bytes: Uint8Array) => {
  const hex = []

  for (let i = 0; i < bytes.length; i++) {
    const current = bytes[i] < 0 ? bytes[i] + 256 : bytes[i]
    hex.push((current >>> 4).toString(16))
    hex.push((current & 0xf).toString(16))
  }

  return hex.join('')
}
