import { Keys } from 'casper-js-sdk'
import { FUNDED_KEYS } from 'casper-node-launcher-js'

export const NETWORK_URL = 'http://127.0.0.1:7777/rpc'
export const NETWORK_NAME = 'casper-net-1'
export const KEYS_PATH = './casper_keys'
export const KEYS_ALGO = 'ed25519'

export const [ALICE, BOB, CHARLIE] = FUNDED_KEYS.map(k => k.private).map(key =>
  Keys.getKeysFromHexPrivKey(key, Keys.SignatureAlgorithm.Ed25519)
)
