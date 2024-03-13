import {
  CLAccountHashBytesParser,
  CLBoolBytesParser,
  CLI32BytesParser,
  CLOptionBytesParser,
  CLOptionType,
  CLStringBytesParser,
  CLTypeBuilder,
  CLU128BytesParser,
  CLU256BytesParser,
  CLU32BytesParser,
  CLU64BytesParser,
  Keys
} from 'casper-js-sdk'
import { FUNDED_KEYS } from 'casper-node-launcher-js'
import { stringToUint8Array } from './parser'
import { Algo } from './schema'
import { parseAccountKeys } from './utils'

export const LOCAL_NODE_URL = 'http://127.0.0.1:7777/rpc'
export const TESTNET_NODE_URL = 'http://195.201.174.222:7777'
export const KEYS_PATH = './casper_keys'

export const [ALICE, BOB, CHARLIE] = FUNDED_KEYS.map(k => k.private).map(key =>
  Keys.getKeysFromHexPrivKey(key, Keys.SignatureAlgorithm.Ed25519)
)
export const TEST = parseAccountKeys(KEYS_PATH, Algo.ed25519)

export const DEFAULT_PAYMENT_AMOUNT = 100000000000n
export const TESTNET_DEPLOY_AMOUNT = 947800000000n

export const BALANCES = stringToUint8Array('balances')
export const ALLOWANCES = stringToUint8Array('allowances')

export const i32Parser = new CLI32BytesParser()
export const u32Parser = new CLU32BytesParser()
export const u64Parser = new CLU64BytesParser()
export const u128Parser = new CLU128BytesParser()
export const u256Parser = new CLU256BytesParser()
export const accountHashParser = new CLAccountHashBytesParser()
export const stringParser = new CLStringBytesParser()
export const boolParser = new CLBoolBytesParser()
export const optionParser = new CLOptionBytesParser()
export const expectedOptionType = new CLOptionType(CLTypeBuilder.string())

export const poolKeyPrefixBytes = [7, 0, 0, 0]
export const contractAddressPrefixBytes = [1]
export const feeTierPrefixBytes = [7, 0, 0, 0]
export const percentagePrefixBytes = [10, 0, 0, 0]
