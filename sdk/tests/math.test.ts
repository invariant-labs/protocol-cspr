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
import { callWasm, initCasperClient, loadWasm } from '../src/utils'

let hashes: {
  invariant: { loadHash: string; packageHash: string }
  tokenX: { loadHash: string; packageHash: string }
  tokenY: { loadHash: string; packageHash: string }
}

describe('test get liquidity by x', () => {
  const client = initCasperClient(LOCAL_NODE_URL)
  const deployer = ALICE
  const positionOwner = BOB
  const positionOwnerHash = positionOwner.publicKey.toAccountHashStr().replace('account-hash-', '')
  const network = Network.Local
  const providedAmount = { v: 430000n }
  let feeTier: FeeTier
  let poolKey: PoolKey

  beforeEach(async () => {
    const wasm = await loadWasm()

    hashes = await deployInvariantAndTokens(client, deployer)

    feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)
    poolKey = await callWasm(
      wasm.newPoolKey,
      hashes.tokenX.packageHash,
      hashes.tokenY.packageHash,
      feeTier
    )

    const invariant = await Invariant.load(client, hashes.invariant.loadHash)
    await invariant.addFeeTier(deployer, network, feeTier)
    const initSqrtPrice = { v: 1005012269622000000000000n }
    await invariant.createPool(deployer, network, poolKey, initSqrtPrice)
  })

  it('test get liquidity by x', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()
    const invariant = await Invariant.load(client, hashes.invariant.loadHash)

    // Below range
    {
      const lowerTickIndex = -50n
      const upperTickIndex = 10n

      const pool = await invariant.getPool(poolKey)

      await assertThrowsAsync(
        callWasm(
          wasm.getLiquidityByX,
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )
      )
    }
    // In range
    {
      const lowerTickIndex = 80n
      const upperTickIndex = 120n
      const pool = await invariant.getPool(poolKey)

      const { l, amount } = await callWasm(
        wasm.getLiquidityByX,
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
        network,
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

      const { l, amount } = await callWasm(
        wasm.getLiquidityByX,
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
        network,
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
  const positionOwnerHash = positionOwner.publicKey.toAccountHashStr().replace('account-hash-', '')
  const network = Network.Local
  const providedAmount = { v: 47600000000n }
  let feeTier: FeeTier
  let poolKey: PoolKey

  beforeEach(async () => {
    const wasm = await loadWasm()
    hashes = await deployInvariantAndTokens(client, deployer)

    feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)
    poolKey = await callWasm(
      wasm.newPoolKey,
      hashes.tokenX.packageHash,
      hashes.tokenY.packageHash,
      feeTier
    )

    const invariant = await Invariant.load(client, hashes.invariant.loadHash)

    await invariant.addFeeTier(deployer, network, feeTier)
    const initSqrtPrice = { v: 367897834491000000000000n }
    await invariant.createPool(deployer, network, poolKey, initSqrtPrice)
  })

  it('test get liquidity by y', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()

    const invariant = await Invariant.load(client, hashes.invariant.loadHash)
    // Below range
    {
      const lowerTickIndex = -22000n
      const upperTickIndex = -21000n

      const pool = await invariant.getPool(poolKey)

      const { l, amount } = await callWasm(
        wasm.getLiquidityByY,
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
        network,
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

      const { l, amount } = await callWasm(
        wasm.getLiquidityByY,
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
        network,
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
        callWasm(
          wasm.getLiquidityByY,
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )
      )
    }
  })
})
