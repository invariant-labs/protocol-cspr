import { FeeTier, PoolKey, Position } from 'invariant-cspr-wasm'
import { ALICE, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { assertThrowsAsync, deployInvariantAndTokens, positionEquals } from '../src/testUtils'
import { callWasm, initCasperClientAndService, loadWasm } from '../src/utils'

let contracts: any = {
  invariant: {
    contract: '',
    address: '',
    packageHash: ''
  },
  tokenX: {
    contract: '',
    address: '',
    packageHash: ''
  },
  tokenY: {
    contract: '',
    address: '',
    packageHash: ''
  }
}

describe('math tests', () => {
  const { client, service } = initCasperClientAndService(LOCAL_NODE_URL)
  const account = ALICE
  const network = Network.Local
  const providedAmount = { v: 430000n }
  let feeTier: FeeTier | any
  let poolKey: PoolKey | any

  beforeEach(async () => {
    const wasm = await loadWasm()
    contracts = await deployInvariantAndTokens(client, service, account)
    feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)
    poolKey = await callWasm(
      wasm.newPoolKey,
      contracts.tokenX.packageHash,
      contracts.tokenY.packageHash,
      feeTier
    )

    const invariant = contracts.invariant.contract
    await invariant.addFeeTier(account, network, feeTier)
    const initSqrtPrice = { v: 1005012269622000000000000n }
    await invariant.createPool(account, network, poolKey, initSqrtPrice)
    const tokenX = contracts.tokenX.contract
    await tokenX.approve(
      account,
      network,
      Key.Hash,
      contracts.invariant.packageHash,
      1000000000000000n
    )
    const tokenY = contracts.tokenY.contract
    await tokenY.approve(
      account,
      network,
      Key.Hash,
      contracts.invariant.packageHash,
      1000000000000000n
    )
  })

  it('test get liquidity by x', async () => {
    const wasm = await loadWasm()
    const invariant = contracts.invariant.contract
    // Below range
    {
      const lowerTickIndex = -50n
      const upperTickIndex = 10n

      const pool = await invariant.getPool(poolKey)

      assertThrowsAsync(
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

      const { l } = await callWasm(
        wasm.getLiquidityByX,
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      await invariant.createPosition(
        account,
        network,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        pool.sqrtPrice
      )

      const position = await invariant.getPosition(account, 0n)
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

      positionEquals(position, expectedPosition)
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

      expect(amount.v).toBe(0n)

      await invariant.createPosition(
        account,
        network,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        pool.sqrtPrice
      )

      const position = await invariant.getPosition(account, 1n)
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
      positionEquals(position, expectedPosition)
    }
  })
})
