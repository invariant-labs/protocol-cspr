import { Keys } from 'casper-js-sdk'
import { FUNDED_KEYS } from 'casper-node-launcher-js'
import { Algo } from './schema'
import { parseAccountKeys, stringToUint8Array } from './utils'

export const LOCAL_NODE_URL = 'http://127.0.0.1:7777/rpc'
export const TESTNET_NODE_URL = 'http://195.201.174.222:7777'
export const KEYS_PATH = './casper_keys'

export const [ALICE, BOB, CHARLIE] = FUNDED_KEYS.map(k => k.private).map(key =>
  Keys.getKeysFromHexPrivKey(key, Keys.SignatureAlgorithm.Ed25519)
)
export const TEST = parseAccountKeys(KEYS_PATH, Algo.ed25519)

export const DEFAULT_PAYMENT_AMOUNT = 10000000000n

export const BALANCES = stringToUint8Array('balances')
