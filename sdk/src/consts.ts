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
export const TESTNET_DEPLOY_AMOUNT = 947800000000n
export const TESTNET_INVARIANT_HASH =
  '17da891dccd576ddaf93b942b4cf06855fcbb70e95cbf8276adb815f9e1cf0d9'
// fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810
export const BALANCES = stringToUint8Array('balances')
export const QUERY_SERVICE_URL = 'http://localhost:8080'
