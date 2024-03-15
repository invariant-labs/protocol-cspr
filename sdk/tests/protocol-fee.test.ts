import type { FeeTier, Percentage, PoolKey } from 'wasm'
import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import { Key, Network } from '../src/schema'
import {
  DeployedContractsHashes,
  assertThrowsAsync,
  deployInvariantAndTokens,
  loadChai
} from '../src/testUtils'
import { getAccountHashFromKey, initCasperClient, loadWasm } from '../src/utils'
import { newFeeTier, newPoolKey } from '../src/wasm'

let chai: typeof import('chai')

const client = initCasperClient(LOCAL_NODE_URL)
let erc20: Erc20
let invariant: Invariant

let hashes: DeployedContractsHashes

const deployer = ALICE
const deployerAddress = getAccountHashFromKey(deployer)

const fee: Percentage = { v: 1000000000n }
const tickSpacing = 1n

let feeTier: FeeTier
let poolKey: PoolKey

describe('protocol fee', () => {
  before(async () => {
    chai = await loadChai()
  })

  beforeEach(async () => {
    hashes = await deployInvariantAndTokens(client, deployer, fee)

    erc20 = await Erc20.load(client, Network.Local, hashes.tokenX.loadHash)
    await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, 1000000000n)
    erc20.setContractHash(hashes.tokenY.loadHash)
    await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, 1000000000n)

    invariant = await Invariant.load(client, hashes.invariant.loadHash, Network.Local)

    feeTier = await newFeeTier(fee, tickSpacing)
    poolKey = await newPoolKey(hashes.tokenX.packageHash, hashes.tokenY.packageHash, feeTier)

    await invariant.addFeeTier(deployer, feeTier)

    await invariant.createPool(deployer, poolKey, { v: 1000000000000000000000000n })

    await invariant.createPosition(
      deployer,
      poolKey,
      -10n,
      10n,
      { v: 10000000000000n },
      { v: 1000000000000000000000000n },
      { v: 1000000000000000000000000n }
    )

    await invariant.swap(deployer, poolKey, true, { v: 4999n }, true, {
      v: 999505344804856076727628n
    })
  })

  it('should withdraw protocol fee', async () => {
    feeTier = await newFeeTier(fee, tickSpacing)
    poolKey = await newPoolKey(hashes.tokenX.packageHash, hashes.tokenY.packageHash, feeTier)

    erc20.setContractHash(hashes.tokenX.loadHash)
    const tokenXBefore = await erc20.getBalanceOf(Key.Account, deployerAddress)
    erc20.setContractHash(hashes.tokenY.loadHash)
    const tokenYBefore = await erc20.getBalanceOf(Key.Account, deployerAddress)

    const poolBefore = await invariant.getPool(poolKey)
    chai.assert.deepEqual(poolBefore.feeProtocolTokenX, { v: 1n })
    chai.assert.deepEqual(poolBefore.feeProtocolTokenY, { v: 0n })

    await invariant.withdrawProtocolFee(deployer, poolKey)

    const poolAfter = await invariant.getPool(poolKey)
    chai.assert.deepEqual(poolAfter.feeProtocolTokenX, { v: 0n })
    chai.assert.deepEqual(poolAfter.feeProtocolTokenY, { v: 0n })

    erc20.setContractHash(hashes.tokenX.loadHash)
    const tokenXAfter = await erc20.getBalanceOf(Key.Account, deployerAddress)
    erc20.setContractHash(hashes.tokenY.loadHash)
    const tokenYAfter = await erc20.getBalanceOf(Key.Account, deployerAddress)

    chai.assert.equal(tokenXAfter, tokenXBefore + 1n)
    chai.assert.equal(tokenYAfter, tokenYBefore)
  })

  it('should change fee receiver', async () => {
    const wasm = await loadWasm()
    const newFeeReceiver = BOB
    const newFeeReceiverAddress = getAccountHashFromKey(newFeeReceiver)
    await invariant.changeFeeReceiver(deployer, poolKey, Key.Account, newFeeReceiverAddress)

    erc20.setContractHash(hashes.tokenX.loadHash)
    const tokenXBefore = await erc20.getBalanceOf(Key.Account, newFeeReceiverAddress)
    erc20.setContractHash(hashes.tokenY.loadHash)
    const tokenYBefore = await erc20.getBalanceOf(Key.Account, newFeeReceiverAddress)

    const poolBefore = await invariant.getPool(poolKey)
    chai.assert.deepEqual(poolBefore.feeProtocolTokenX, { v: 1n })
    chai.assert.deepEqual(poolBefore.feeProtocolTokenY, { v: 0n })

    assertThrowsAsync(
      invariant.withdrawProtocolFee(deployer, poolKey),
      wasm.InvariantError.NotFeeReceiver
    )

    await invariant.withdrawProtocolFee(newFeeReceiver, poolKey)

    const poolAfter = await invariant.getPool(poolKey)
    chai.assert.deepEqual(poolAfter.feeProtocolTokenX, { v: 0n })
    chai.assert.deepEqual(poolAfter.feeProtocolTokenY, { v: 0n })

    erc20.setContractHash(hashes.tokenX.loadHash)
    const tokenXAfter = await erc20.getBalanceOf(Key.Account, newFeeReceiverAddress)
    erc20.setContractHash(hashes.tokenY.loadHash)
    const tokenYAfter = await erc20.getBalanceOf(Key.Account, newFeeReceiverAddress)

    chai.assert.equal(tokenXAfter, tokenXBefore + 1n)
    chai.assert.equal(tokenYAfter, tokenYBefore)
  })
})
