import type { Percentage, Price, SqrtPrice } from 'invariant-cspr-wasm'
import { ALICE, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import { deployInvariantAndTokens, loadChai } from '../src/testUtils'
import {
  calculateFee,
  calculatePriceImpact,
  calculateSqrtPriceAfterSlippage,
  callWasm,
  initCasperClientAndService,
  loadWasm,
  priceToSqrtPrice,
  sqrtPriceToPrice
} from '../src/utils'

describe('utils', () => {
  it('calculatePriceImpact', async () => {
    const chai = await loadChai()
    // Incrasing price
    {
      // price change       120 -> 599
      // real price impact  79.96661101836...%
      const startingPrice: SqrtPrice = { v: 10954451150103322269139395n }
      const endingSqrtPrice: SqrtPrice = { v: 24474476501040834315678144n }
      const priceImpact: Percentage = await calculatePriceImpact(startingPrice, endingSqrtPrice)
      chai.assert.equal(priceImpact.v, 799666110183n)
    }
    // Decreasing price
    {
      // price change       0.367 -> 1.0001^(-221818)
      // real price impact  99.9999999365...%
      const startingPrice: SqrtPrice = { v: 605805249234438377196232n }
      const endingSqrtPrice: SqrtPrice = { v: 15258932449895975601n }
      const priceImpact: Percentage = await calculatePriceImpact(startingPrice, endingSqrtPrice)
      chai.assert.equal(priceImpact.v, 999999999365n)
    }
  })
  it('test calculateSqrtPriceAfterSlippage', async () => {
    const wasm = await loadWasm()
    const chai = await loadChai()
    // no slippage up
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 0n, 0n) }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      chai.assert.deepEqual(limitSqrt, sqrtPrice)
    }
    // no slippage down
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 0n, 0n) }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      chai.assert.deepEqual(limitSqrt, sqrtPrice)
    }
    // slippage 1% up
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 1n, 2n) }
      // sqrt(1) * sqrt(1 + 0.01) = 1.0049876
      const expected: SqrtPrice = { v: 1004987562112089027021926n }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      chai.assert.deepEqual(limitSqrt, expected)
    }
    // slippage 1% down
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 1n, 2n) }
      // sqrt(1) * sqrt(1 - 0.01) = 0.99498744
      const expected: SqrtPrice = { v: 994987437106619954734479n }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      chai.assert.deepEqual(limitSqrt, expected)
    }
    // slippage 0.5% up
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 5n, 3n) }
      // sqrt(1) * sqrt(1 - 0.005) = 1.00249688
      const expected: SqrtPrice = { v: 1002496882788171067537936n }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      chai.assert.deepEqual(limitSqrt, expected)
    }
    // slippage 0.5% down
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 5n, 3n) }
      // sqrt(1) * sqrt(1 - 0.005) = 0.997496867
      const expected: SqrtPrice = { v: 997496867163000166582694n }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      chai.assert.deepEqual(limitSqrt, expected)
    }
    // slippage 0.00003% up
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 3n, 7n) }
      // sqrt(1) * sqrt(1 + 0.0000003) = 1.00000015
      const expected: SqrtPrice = { v: 1000000149999988750001687n }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      chai.assert.deepEqual(limitSqrt, expected)
    }
    // slippage 0.00003% down
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 3n, 7n) }
      // sqrt(1) * sqrt(1 - 0.0000003) = 0.99999985
      const expected: SqrtPrice = { v: 999999849999988749998312n }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      chai.assert.deepEqual(limitSqrt, expected)
    }
    // slippage 100% up
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 1n, 0n) }
      // sqrt(1) * sqrt(1 + 1) = 1.414213562373095048801688...
      const expected: SqrtPrice = { v: 1414213562373095048801688n }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      chai.assert.deepEqual(limitSqrt, expected)
    }
    // slippage 100% down
    {
      const sqrtPrice: SqrtPrice = { v: await callWasm(wasm.toSqrtPrice, 1n, 0n) }
      const slippage: Percentage = { v: await callWasm(wasm.toPercentage, 1n, 0n) }
      // sqrt(1) * sqrt(1 - 1) = 0
      const expected: SqrtPrice = { v: 0n }
      const limitSqrt: SqrtPrice = await calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      chai.assert.deepEqual(limitSqrt, expected)
    }
  })
  it('sqrt price and price conversion', async () => {
    const chai = await loadChai()
    // 1.00 = sqrt(1.00)
    {
      const sqrtPrice: SqrtPrice = await priceToSqrtPrice({ v: 1000000000000000000000000n })
      const expectedSqrtPrice: SqrtPrice = { v: 1000000000000000000000000n }
      chai.assert.deepEqual(sqrtPrice, expectedSqrtPrice)
    }
    // 1.414213562373095048801688... = sqrt(2.00)
    {
      const sqrtPrice: SqrtPrice = await priceToSqrtPrice({ v: 2000000000000000000000000n })
      const expectedSqrtPrice: SqrtPrice = { v: 1414213562373095048801688n }
      chai.assert.deepEqual(sqrtPrice, expectedSqrtPrice)
    }
    // 0.5 = sqrt(0.25)
    {
      const sqrtPrice: SqrtPrice = await priceToSqrtPrice({ v: 250000000000000000000000n })
      const expectedSqrtPrice: SqrtPrice = { v: 500000000000000000000000n }
      chai.assert.deepEqual(sqrtPrice, expectedSqrtPrice)
    }
    // sqrt(1.00) = 1.00
    {
      const price: Price = await sqrtPriceToPrice({ v: 1000000000000000000000000n })
      const expectedPrice: Price = { v: 1000000000000000000000000n }
      chai.assert.deepEqual(price, expectedPrice)
    }
    // sqrt(1.414213562373095048801688...) = 2.00
    {
      const price: Price = await sqrtPriceToPrice({ v: 1414213562373095048801688n })
      const expectedPrice: Price = { v: 1999999999999999999999997n }
      chai.assert.deepEqual(price, expectedPrice)
    }
    // sqrt(0.25) = 0.5
    {
      const price: Price = await sqrtPriceToPrice({ v: 500000000000000000000000n })
      const expectedPrice: Price = { v: 250000000000000000000000n }
      chai.assert.deepEqual(price, expectedPrice)
    }
  })
  it('test calculate fee', async () => {
    const chai = await loadChai()
    const wasm = await loadWasm()

    const liquidityDelta = { v: 10000000000000n }
    const lowerTickIndex = -10n
    const upperTickIndex = 10n
    const swapAmount = { v: 4999n }
    const targetSqrtPrice = { v: 999505344804856076727628n }
    const approvalAmount = 1000000000n

    const { client, service } = initCasperClientAndService(LOCAL_NODE_URL)
    const deployer = ALICE
    const deployerAddress = deployer.publicKey.toAccountHashStr().replace('account-hash-', '')
    const network = Network.Local

    const hashes = await deployInvariantAndTokens(client, service, deployer)

    const feeTier = await callWasm(wasm.newFeeTier, { v: 10000000000n }, 1n)
    const poolKey = await callWasm(
      wasm.newPoolKey,
      hashes.tokenX.packageHash,
      hashes.tokenY.packageHash,
      feeTier
    )

    const invariant = await Invariant.load(client, service, hashes.invariant.loadHash)
    await invariant.addFeeTier(deployer, network, feeTier)
    const initSqrtPrice = { v: 1000000000000000000000000n }
    await invariant.createPool(deployer, network, poolKey, initSqrtPrice)

    const erc20 = await Erc20.load(client, service, network, hashes.tokenX.loadHash)
    await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, approvalAmount)
    await erc20.setContractHash(hashes.tokenY.loadHash)
    await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, approvalAmount)

    const pool = await invariant.getPool(poolKey)
    await invariant.createPosition(
      deployer,
      Network.Local,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      liquidityDelta,
      pool.sqrtPrice,
      pool.sqrtPrice
    )

    {
      await erc20.setContractHash(hashes.tokenX.loadHash)
      await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, approvalAmount)
      await erc20.setContractHash(hashes.tokenY.loadHash)
      await erc20.approve(deployer, Key.Hash, hashes.invariant.packageHash, approvalAmount)

      await invariant.swap(
        deployer,
        Network.Local,
        poolKey,
        true,
        swapAmount,
        true,
        targetSqrtPrice
      )
      const pool = await invariant.getPool(poolKey)
      const position = await invariant.getPosition(deployer, 0n)
      const lowerTick = await invariant.getTick(poolKey, lowerTickIndex)
      const upperTick = await invariant.getTick(poolKey, upperTickIndex)

      const [x, y] = await calculateFee(pool, position, lowerTick, upperTick)

      chai.assert.equal(y, { v: 0n })

      await erc20.setContractHash(hashes.tokenX.loadHash)
      const balanceXBeforeClaim = await erc20.balanceOf(Key.Account, deployerAddress)
      await erc20.setContractHash(hashes.tokenY.loadHash)
      const balanceYBeforeClaim = await erc20.balanceOf(Key.Account, deployerAddress)

      await invariant.claimFee(deployer, Network.Local, 0n)

      await erc20.setContractHash(hashes.tokenX.loadHash)
      const balanceXAfterClaim = await erc20.balanceOf(Key.Account, deployerAddress)
      await erc20.setContractHash(hashes.tokenY.loadHash)
      const balanceYAfterClaim = await erc20.balanceOf(Key.Account, deployerAddress)

      chai.assert.deepEqual(balanceXAfterClaim, balanceXBeforeClaim + x.v)
      chai.assert.deepEqual(balanceYAfterClaim, balanceYBeforeClaim)
    }
  })
})