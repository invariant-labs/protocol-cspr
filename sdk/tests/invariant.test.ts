import type { Liquidity, Percentage } from 'invariant-cspr-wasm'
import { ALICE, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import { assertThrowsAsync, deployInvariantAndTokens, loadChai } from '../src/testUtils'
import { callWasm, initCasperClient, loadWasm } from '../src/utils'

let hashes: {
  invariant: { loadHash: string; packageHash: string }
  tokenX: { loadHash: string; packageHash: string }
  tokenY: { loadHash: string; packageHash: string }
}

describe('invariant test', () => {
  const client = initCasperClient(LOCAL_NODE_URL)
  const deployer = ALICE
  const network = Network.Local

  beforeEach(async () => {
    hashes = await deployInvariantAndTokens(client, deployer)
  })

  it('should change protocol fee', async () => {
    const chai = await loadChai()
    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    const newFee: Percentage = { v: 20000000000n }
    await invariant.changeProtocolFee(deployer, newFee)

    const { protocolFee } = await invariant.getInvariantConfig()
    chai.assert.deepEqual(protocolFee, newFee)
  })

  it('should add fee tier', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()
    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    const feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)

    await invariant.addFeeTier(deployer, feeTier)

    const feeTierExist = await invariant.feeTierExist(feeTier)
    chai.assert.exists(feeTierExist)
  })
  it('should remove fee tier', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()
    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    const feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)

    {
      await invariant.addFeeTier(deployer, feeTier)
      const feeTierExist = await invariant.feeTierExist(feeTier)
      chai.assert.exists(feeTierExist)
    }
    {
      await invariant.removeFeeTier(deployer, feeTier)
      const feeTierExist = await invariant.feeTierExist(feeTier)
      chai.assert.exists(!feeTierExist)
    }
  })
  it('should get tick and check if it is initliazed', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()
    const lowerTickIndex = -10n
    const initTickIndex = 0n
    const upperTickIndex = 10n
    const liquidityDelta: Liquidity = { v: 10000n }
    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    const feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)
    const poolKey = await callWasm(
      wasm.newPoolKey,
      hashes.tokenX.packageHash,
      hashes.tokenY.packageHash,
      feeTier
    )

    await invariant.addFeeTier(deployer, feeTier)
    const feeTierExist = await invariant.feeTierExist(feeTier)
    chai.assert.exists(feeTierExist)

    const initSqrtPrice = { v: 1000000000000000000000000n }
    await invariant.createPool(deployer, poolKey, initSqrtPrice)

    const erc20 = await Erc20.load(client, network, hashes.tokenX.loadHash)
    await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, 1000000000000n)
    await erc20.setContractHash(hashes.tokenY.loadHash)
    await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, 1000000000000n)

    const pool = await invariant.getPool(poolKey)
    await invariant.createPosition(
      deployer,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      liquidityDelta,
      pool.sqrtPrice,
      pool.sqrtPrice
    )

    const lowerTick = await invariant.getTick(poolKey, lowerTickIndex)
    assertThrowsAsync(invariant.getTick(poolKey, initTickIndex))
    const upperTick = await invariant.getTick(poolKey, upperTickIndex)

    const isLowerTickInitialized = await invariant.isTickInitialized(poolKey, lowerTickIndex)
    const isInitTickInitialized = await invariant.isTickInitialized(poolKey, initTickIndex)
    const isUpperTickInitialized = await invariant.isTickInitialized(poolKey, upperTickIndex)

    chai.assert.exists(isLowerTickInitialized)
    chai.assert.exists(!isInitTickInitialized)
    chai.assert.exists(isUpperTickInitialized)

    chai.assert.deepEqual(lowerTick, {
      index: -10n,
      sign: true,
      liquidityChange: liquidityDelta,
      liquidityGross: liquidityDelta,
      sqrtPrice: { v: 999500149965000000000000n },
      feeGrowthOutsideX: { v: 0n },
      feeGrowthOutsideY: { v: 0n },
      secondsOutside: lowerTick.secondsOutside
    })
    chai.assert.deepEqual(upperTick, {
      index: 10n,
      sign: false,
      liquidityChange: liquidityDelta,
      liquidityGross: liquidityDelta,
      sqrtPrice: { v: 1000500100010000000000000n },
      feeGrowthOutsideX: { v: 0n },
      feeGrowthOutsideY: { v: 0n },
      secondsOutside: upperTick.secondsOutside
    })
  })
  it('create pool', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()

    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    const feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)
    const poolKey = await callWasm(
      wasm.newPoolKey,
      hashes.tokenX.packageHash,
      hashes.tokenY.packageHash,
      feeTier
    )

    await invariant.addFeeTier(deployer, feeTier)
    const feeTierExist = await invariant.feeTierExist(feeTier)
    chai.assert.exists(feeTierExist)

    const initSqrtPrice = { v: 1000000000000000000000000n }
    await invariant.createPool(deployer, poolKey, initSqrtPrice)

    const pools = await invariant.getPools()
    const pool = await invariant.getPool(poolKey)
    const expectedPool = {
      liquidity: { v: 0n },
      sqrtPrice: { v: 1000000000000000000000000n },
      currentTickIndex: 0n,
      feeGrowthGlobalX: { v: 0n },
      feeGrowthGlobalY: { v: 0n },
      feeProtocolTokenX: { v: 0n },
      feeProtocolTokenY: { v: 0n },
      startTimestamp: pool.startTimestamp,
      lastTimestamp: pool.lastTimestamp,
      feeReceiver: pool.feeReceiver,
      oracleInitialized: false
    }

    chai.assert.exists(pools.length === 1)
    chai.assert.deepEqual(pool, expectedPool)
  })
  it('attempt to create pool with wront tick & sqrtPrice relationship', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()

    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    const feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)
    const poolKey = await callWasm(
      wasm.newPoolKey,
      hashes.tokenX.packageHash,
      hashes.tokenY.packageHash,
      feeTier
    )

    await invariant.addFeeTier(deployer, feeTier)
    const feeTierExist = await invariant.feeTierExist(feeTier)
    chai.assert.exists(feeTierExist)

    const initSqrtPrice = { v: 1000175003749000000000000n }
    assertThrowsAsync(
      invariant.createPool(deployer, poolKey, initSqrtPrice),
      wasm.InvariantError.InvalidInitTick
    )
  })
  it('create pool x/y and y/x', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()

    const invariant = await Invariant.load(client, hashes.invariant.loadHash, network)
    const feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)
    const poolKey = await callWasm(
      wasm.newPoolKey,
      hashes.tokenX.packageHash,
      hashes.tokenY.packageHash,
      feeTier
    )

    await invariant.addFeeTier(deployer, feeTier)
    const feeTierExist = await invariant.feeTierExist(feeTier)
    chai.assert.exists(feeTierExist)

    const initSqrtPrice = { v: 1000000000000000000000000n }
    await invariant.createPool(deployer, poolKey, initSqrtPrice)

    const pools = await invariant.getPools()
    const pool = await invariant.getPool(poolKey)
    const expectedPool = {
      liquidity: { v: 0n },
      sqrtPrice: { v: 1000000000000000000000000n },
      currentTickIndex: 0n,
      feeGrowthGlobalX: { v: 0n },
      feeGrowthGlobalY: { v: 0n },
      feeProtocolTokenX: { v: 0n },
      feeProtocolTokenY: { v: 0n },
      startTimestamp: pool.startTimestamp,
      lastTimestamp: pool.lastTimestamp,
      feeReceiver: pool.feeReceiver,
      oracleInitialized: false
    }

    chai.assert.exists(pools.length === 1)
    chai.assert.deepEqual(pool, expectedPool)

    const switchedPoolKey = {
      tokenX: poolKey.tokenY,
      tokenY: poolKey.tokenX,
      feeTier: poolKey.feeTier
    }

    assertThrowsAsync(
      invariant.createPool(deployer, switchedPoolKey, initSqrtPrice),
      wasm.InvariantError.PoolAlreadyExist
    )
  })
})
