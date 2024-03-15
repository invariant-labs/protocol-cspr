import { Keys } from 'casper-js-sdk'
import { FUNDED_KEYS } from 'casper-node-launcher-js'
import { stringToUint8Array } from './parser'

export const LOCAL_NODE_URL = 'http://127.0.0.1:7777/rpc'
export const TESTNET_NODE_URL = 'http://195.201.174.222:7777'

export const [ALICE, BOB, CHARLIE] = FUNDED_KEYS.map(k => k.private).map(key =>
  Keys.getKeysFromHexPrivKey(key, Keys.SignatureAlgorithm.Ed25519)
)

export const DEFAULT_PAYMENT_AMOUNT = 100000000000n
export const TESTNET_DEPLOY_AMOUNT = 947800000000n

export const BALANCES = stringToUint8Array('balances')
export const ALLOWANCES = stringToUint8Array('allowances')

export const ERC20_CONTRACT_NAME = 'erc20'
export const INVARIANT_CONTRACT_NAME = 'invariant'
