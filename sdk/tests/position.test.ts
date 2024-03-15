import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import { Key, Network } from '../src/schema'
import {
  DeployedContractsHashes,
  assertThrowsAsync,
  deployInvariantAndTokens,
  loadChai,
  positionEquals
} from '../src/testUtils'
import { getAccountHashFromKey, initCasperClient } from '../src/utils'
import { newFeeTier, newPoolKey } from '../src/wasm'
import type { FeeTier, PoolKey, Position } from '../wasm'

let chai: typeof import('chai')

let hashes: DeployedContractsHashes

const client = initCasperClient(LOCAL_NODE_URL)
let erc20: Erc20
let invariant: Invariant
const deployer = ALICE
const deployerAddress = getAccountHashFromKey(deployer)

const fee = { v: 6000000000n }
const tickSpacing = 1n
const lowerTickIndex = -20n
const upperTickIndex = 10n

let feeTier: FeeTier
let poolKey: PoolKey

describe('position', () => {
  before(async () => {
    chai = await loadChai()
  })

  beforeEach(async () => {
    hashes = await deployInvariantAndTokens(client, deployer, fee)

    erc20 = await Erc20.load(client, Network.Local, hashes.tokenX.loadHash)
    await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, 6000000000n)
    erc20.setContractHash(hashes.tokenY.loadHash)
    await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, 6000000000n)

    invariant = await Invariant.load(client, hashes.invariant.loadHash, Network.Local)

    feeTier = await newFeeTier(fee, tickSpacing)
    poolKey = await newPoolKey(hashes.tokenX.packageHash, hashes.tokenY.packageHash, feeTier)

    await invariant.addFeeTier(deployer, feeTier)

    await invariant.createPool(deployer, poolKey, { v: 1000000000000000000000000n })

    await invariant.createPosition(
      deployer,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      { v: 100000000000n },
      { v: 1000000000000000000000000n },
      { v: 1000000000000000000000000n }
    )
  })

  it('create position', async () => {
    const position = await invariant.getPosition(deployer, 0n)

    chai.assert.deepEqual(position.liquidity, { v: 100000000000n })
    chai.assert.deepEqual(position.lowerTickIndex, lowerTickIndex)
    chai.assert.deepEqual(position.upperTickIndex, upperTickIndex)
    chai.assert.deepEqual(position.feeGrowthInsideX, { v: 0n })
    chai.assert.deepEqual(position.feeGrowthInsideY, { v: 0n })
    chai.assert.deepEqual(position.tokensOwedX, { v: 0n })
    chai.assert.deepEqual(position.tokensOwedY, { v: 0n })
  })

  it('remove position', async () => {
    await invariant.removePosition(deployer, 0n)

    assertThrowsAsync(invariant.getPosition(deployer, 0n))
    const positions = await invariant.getPositions(deployer)
    chai.expect(positions.length).to.equal(0)

    assertThrowsAsync(invariant.getTick(poolKey, lowerTickIndex))
    assertThrowsAsync(invariant.getTick(poolKey, upperTickIndex))

    const isLowerTickInitialized = await invariant.isTickInitialized(poolKey, lowerTickIndex)
    chai.expect(isLowerTickInitialized).to.equal(false)

    const isUpperTickInitialized = await invariant.isTickInitialized(poolKey, upperTickIndex)
    chai.expect(isUpperTickInitialized).to.equal(false)
  })

  it('transfer position', async () => {
    const recipient = BOB
    const recipientAddress = getAccountHashFromKey(recipient)

    await invariant.transferPosition(deployer, 0n, Key.Account, recipientAddress)

    assertThrowsAsync(invariant.getPosition(deployer, 0n))
    const position = await invariant.getPosition(recipient, 0n)

    const expectedPosition: Position = {
      poolKey,
      liquidity: { v: 100000000000n },
      lowerTickIndex,
      upperTickIndex,
      feeGrowthInsideX: { v: 0n },
      feeGrowthInsideY: { v: 0n },
      tokensOwedX: { v: 0n },
      tokensOwedY: { v: 0n },
      lastBlockNumber: position.lastBlockNumber
    }

    await positionEquals(position, expectedPosition)
  })

  it('claim fee', async () => {
    const swapper = BOB
    const swapperAddress = getAccountHashFromKey(swapper)
    const positionOwnerAddress = deployerAddress

    const amount = { v: 1000n }

    erc20.setContractHash(hashes.tokenX.loadHash)
    await erc20.mint(deployer, Key.Account, swapperAddress, amount.v)
    await erc20.approve(swapper, Key.Hash, hashes.invariant.packageHash, amount.v)

    const poolBefore = await invariant.getPool(poolKey)

    const targetSqrtPrice = { v: 15258932000000000000n }
    await invariant.swap(swapper, poolKey, true, amount, true, targetSqrtPrice)

    const poolAfter = await invariant.getPool(poolKey)
    erc20.setContractHash(hashes.tokenX.loadHash)
    const swapperTokenXAfter = await erc20.getBalanceOf(Key.Account, swapperAddress)
    erc20.setContractHash(hashes.tokenY.loadHash)
    const swapperTokenYAfter = await erc20.getBalanceOf(Key.Account, swapperAddress)

    chai.assert.equal(swapperTokenXAfter, 0n)
    chai.assert.equal(swapperTokenYAfter, 993n)

    erc20.setContractHash(hashes.tokenX.loadHash)
    const invariantTokenX = await erc20.getBalanceOf(Key.Hash, hashes.invariant.packageHash)
    erc20.setContractHash(hashes.tokenY.loadHash)
    const invariantTokenY = await erc20.getBalanceOf(Key.Hash, hashes.invariant.packageHash)

    chai.assert.equal(invariantTokenX, 1500n)
    chai.assert.equal(invariantTokenY, 7n)

    chai.assert.deepEqual(poolAfter.liquidity, poolBefore.liquidity)
    chai.assert.notDeepEqual(poolAfter.sqrtPrice, poolBefore.sqrtPrice)
    chai.assert.deepEqual(poolAfter.currentTickIndex, lowerTickIndex)
    chai.assert.deepEqual(poolAfter.feeGrowthGlobalX, { v: 50000000000000000000000n })
    chai.assert.deepEqual(poolAfter.feeGrowthGlobalY, { v: 0n })
    chai.assert.deepEqual(poolAfter.feeProtocolTokenX, { v: 1n })
    chai.assert.deepEqual(poolAfter.feeProtocolTokenY, { v: 0n })

    erc20.setContractHash(hashes.tokenX.loadHash)
    const positionOwnerBeforeX = await erc20.getBalanceOf(Key.Account, positionOwnerAddress)
    const invariantBeforeX = await erc20.getBalanceOf(Key.Hash, hashes.invariant.packageHash)

    await invariant.claimFee(deployer, 0n)

    erc20.setContractHash(hashes.tokenX.loadHash)
    const positionOwnerAfterX = await erc20.getBalanceOf(Key.Account, positionOwnerAddress)
    const invariantAfterX = await erc20.getBalanceOf(Key.Hash, hashes.invariant.packageHash)

    const position = await invariant.getPosition(deployer, 0n)
    const pool = await invariant.getPool(poolKey)
    const expectedTokensClaimed = 5n

    chai.assert.deepEqual(positionOwnerAfterX - expectedTokensClaimed, positionOwnerBeforeX)
    chai.assert.deepEqual(invariantAfterX + expectedTokensClaimed, invariantBeforeX)

    chai.assert.deepEqual(position.feeGrowthInsideX, pool.feeGrowthGlobalX)
    chai.assert.deepEqual(position.tokensOwedX, { v: 0n })
  })
})
