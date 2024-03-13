import { FeeTier, PoolKey, Position } from 'invariant-cspr-wasm'
import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import {
  assertThrowsAsync,
  deployInvariantAndTokens,
  loadChai,
  positionEquals
} from '../src/testUtils'
import { createFeeTier, createPoolKey, getAccountHashFromKey, initCasperClient } from '../src/utils'
import { getLiquidityByX, getLiquidityByY } from '../src/wasm'

let hashes: {
  invariant: { loadHash: string; packageHash: string }
  tokenX: { loadHash: string; packageHash: string }
  tokenY: { loadHash: string; packageHash: string }
}

describe('test get liquidity by x', () => {
  const client = initCasperClient(LOCAL_NODE_URL)
  const deployer = ALICE
  const positionOwner = BOB
  const positionOwnerHash = getAccountHashFromKey(positionOwner)
  const network = Network.Local
  const providedAmount = { v: 430000n }
  let feeTier: FeeTier
  let poolKey: PoolKey

  beforeEach(async () => {
    hashes = await deployInvariantAndTokens(client, deployer)

    feeTier = await createFeeTier({ v: 6000000000n }, 10n)
    poolKey = await createPoolKey(hashes.tokenX.packageHash, hashes.tokenY.packageHash, feeTier)

    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    await invariant.addFeeTier(deployer, feeTier)
    const initSqrtPrice = { v: 1005012269622000000000000n }
    await invariant.createPool(deployer, poolKey, initSqrtPrice)
  })

  it('test get liquidity by x', async () => {
    const chai = await loadChai()
    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)

    // Below range
    {
      const lowerTickIndex = -50n
      const upperTickIndex = 10n

      const pool = await invariant.getPool(poolKey)

      await assertThrowsAsync(
        getLiquidityByX(providedAmount, lowerTickIndex, upperTickIndex, pool.sqrtPrice, true)
      )
    }
    // In range
    {
      const lowerTickIndex = 80n
      const upperTickIndex = 120n
      const pool = await invariant.getPool(poolKey)

      const { l, amount } = await getLiquidityByX(
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      const erc20 = await Erc20.load(client, network, hashes.tokenX.loadHash)
      await erc20.mint(positionOwner, Key.Account, positionOwnerHash, providedAmount.v)
      await erc20.approve(positionOwner, Key.Hash, hashes.invariant.packageHash, providedAmount.v)
      await erc20.setContractHash(hashes.tokenY.loadHash)
      await erc20.mint(positionOwner, Key.Account, positionOwnerHash, amount.v)
      await erc20.approve(positionOwner, Key.Hash, hashes.invariant.packageHash, amount.v)

      await invariant.createPosition(
        positionOwner,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        pool.sqrtPrice
      )

      const position = await invariant.getPosition(positionOwner, 0n)
      const expectedPosition: Position = {
        poolKey,
        liquidity: l,
        lowerTickIndex,
        upperTickIndex,
        feeGrowthInsideX: { v: 0n },
        feeGrowthInsideY: { v: 0n },
        lastBlockNumber: 0n,
        tokensOwedX: { v: 0n },
        tokensOwedY: { v: 0n }
      }

      await positionEquals(position, expectedPosition)
    }
    // Above Range
    {
      const lowerTickIndex = 150n
      const upperTickIndex = 800n
      const pool = await invariant.getPool(poolKey)

      const { l, amount } = await getLiquidityByX(
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      chai.assert.equal(amount.v, 0n)

      const erc20 = await Erc20.load(client, network, hashes.tokenX.loadHash)
      await erc20.mint(positionOwner, Key.Account, positionOwnerHash, providedAmount.v)
      await erc20.approve(positionOwner, Key.Hash, hashes.invariant.packageHash, providedAmount.v)

      await invariant.createPosition(
        positionOwner,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        pool.sqrtPrice
      )

      const position = await invariant.getPosition(positionOwner, 1n)
      const expectedPosition: Position = {
        poolKey,
        liquidity: l,
        lowerTickIndex,
        upperTickIndex,
        feeGrowthInsideX: { v: 0n },
        feeGrowthInsideY: { v: 0n },
        lastBlockNumber: 0n,
        tokensOwedX: { v: 0n },
        tokensOwedY: { v: 0n }
      }
      await positionEquals(position, expectedPosition)
    }
  })
})

describe('test get liquidity by y', () => {
  const client = initCasperClient(LOCAL_NODE_URL)
  const deployer = ALICE
  const positionOwner = BOB
  const positionOwnerHash = getAccountHashFromKey(positionOwner)
  const network = Network.Local
  const providedAmount = { v: 47600000000n }
  let feeTier: FeeTier
  let poolKey: PoolKey

  beforeEach(async () => {
    hashes = await deployInvariantAndTokens(client, deployer)

    feeTier = await createFeeTier({ v: 6000000000n }, 10n)
    poolKey = await createPoolKey(hashes.tokenX.packageHash, hashes.tokenY.packageHash, feeTier)

    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)

    await invariant.addFeeTier(deployer, feeTier)
    const initSqrtPrice = { v: 367897834491000000000000n }
    await invariant.createPool(deployer, poolKey, initSqrtPrice)
  })

  it('test get liquidity by y', async () => {
    const chai = await loadChai()

    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    // Below range
    {
      const lowerTickIndex = -22000n
      const upperTickIndex = -21000n

      const pool = await invariant.getPool(poolKey)

      const { l, amount } = await getLiquidityByY(
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      chai.assert.equal(amount.v, 0n)

      const erc20 = await Erc20.load(client, network, hashes.tokenY.loadHash)
      await erc20.mint(positionOwner, Key.Account, positionOwnerHash, providedAmount.v)
      await erc20.approve(positionOwner, Key.Hash, hashes.invariant.packageHash, providedAmount.v)

      await invariant.createPosition(
        positionOwner,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        pool.sqrtPrice
      )

      const position = await invariant.getPosition(positionOwner, 0n)
      const expectedPosition: Position = {
        poolKey,
        liquidity: l,
        lowerTickIndex,
        upperTickIndex,
        feeGrowthInsideX: { v: 0n },
        feeGrowthInsideY: { v: 0n },
        lastBlockNumber: 0n,
        tokensOwedX: { v: 0n },
        tokensOwedY: { v: 0n }
      }

      await positionEquals(position, expectedPosition)
    }
    // In range
    {
      const lowerTickIndex = -25000n
      const upperTickIndex = -19000n
      const pool = await invariant.getPool(poolKey)

      const { l, amount } = await getLiquidityByY(
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      const erc20 = await Erc20.load(client, network, hashes.tokenY.loadHash)
      await erc20.mint(positionOwner, Key.Account, positionOwnerHash, providedAmount.v)
      await erc20.approve(positionOwner, Key.Hash, hashes.invariant.packageHash, providedAmount.v)
      await erc20.setContractHash(hashes.tokenX.loadHash)
      await erc20.mint(positionOwner, Key.Account, positionOwnerHash, amount.v)
      await erc20.approve(positionOwner, Key.Hash, hashes.invariant.packageHash, amount.v)

      await invariant.createPosition(
        positionOwner,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        pool.sqrtPrice
      )

      const position = await invariant.getPosition(positionOwner, 1n)
      const expectedPosition: Position = {
        poolKey,
        liquidity: l,
        lowerTickIndex,
        upperTickIndex,
        feeGrowthInsideX: { v: 0n },
        feeGrowthInsideY: { v: 0n },
        lastBlockNumber: 0n,
        tokensOwedX: { v: 0n },
        tokensOwedY: { v: 0n }
      }

      await positionEquals(position, expectedPosition)
    }
    // Above Range
    {
      const lowerTickIndex = -10000n
      const upperTickIndex = 0n
      const pool = await invariant.getPool(poolKey)

      await assertThrowsAsync(
        getLiquidityByY(providedAmount, lowerTickIndex, upperTickIndex, pool.sqrtPrice, true)
      )
    }
  })
})
