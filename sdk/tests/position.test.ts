import { FeeTier, PoolKey } from 'invariant-cspr-wasm'
import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import { assertThrowsAsync, loadChai } from '../src/testUtils'
import { callWasm, getAccountHashFromKey, initCasperClient, loadWasm } from '../src/utils'

let wasm: typeof import('invariant-cspr-wasm')
let chai: typeof import('chai')

const client = initCasperClient(LOCAL_NODE_URL)
let erc20: Erc20
let invariant: Invariant
let invariantAddress: string
let invariantContractPackage: string
let token0Address: string
let token1Address: string
let token0ContractPackage: string
let token1ContractPackage: string
const aliceAddress = getAccountHashFromKey(ALICE)
const bobAddress = getAccountHashFromKey(BOB)

const fee = 6000000000n
const tickSpacing = 1n
const lowerTickIndex = -20n
const upperTickIndex = 10n

let feeTier: FeeTier
let poolKey: PoolKey

describe('position', () => {
  before(async () => {
    wasm = await loadWasm()
    chai = await loadChai()
  })

  beforeEach(async () => {
    const [token0ContractPackageHash, token0ContractHash] = await Erc20.deploy(
      client,
      Network.Local,
      ALICE,
      'erc20-1',
      1000000000n,
      'Coin',
      'COIN',
      12n,
      150000000000n
    )
    const [token1ContractPackageHash, token1ContractHash] = await Erc20.deploy(
      client,
      Network.Local,
      ALICE,
      'erc20-2',
      1000000000n,
      'Coin',
      'COIN',
      12n,
      150000000000n
    )
    const [invariantContractPackageHash, invariantContractHash] = await Invariant.deploy(
      client,
      Network.Local,
      ALICE,
      fee,
      600000000000n
    )

    token0Address = token0ContractHash
    token1Address = token1ContractHash
    invariantAddress = invariantContractPackageHash
    token0ContractPackage = token0ContractPackageHash
    token1ContractPackage = token1ContractPackageHash
    invariantContractPackage = invariantContractPackageHash

    erc20 = await Erc20.load(client, Network.Local, token0ContractHash)
    await erc20.approve(ALICE, Key.Hash, invariantContractPackage, fee)
    erc20 = await Erc20.load(client, Network.Local, token1ContractHash)
    await erc20.approve(ALICE, Key.Hash, invariantContractPackage, fee)

    invariant = await Invariant.load(client, invariantContractHash, Network.Local)

    feeTier = await callWasm(wasm.newFeeTier, { v: fee }, tickSpacing)
    poolKey = await callWasm(wasm.newPoolKey, token0ContractPackage, token1ContractPackage, feeTier)

    await invariant.addFeeTier(ALICE, feeTier)

    await invariant.createPool(ALICE, poolKey, { v: 1000000000000000000000000n })

    await invariant.createPosition(
      ALICE,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      { v: 100000000000n },
      { v: 1000000000000000000000000n },
      { v: 1000000000000000000000000n }
    )
  })

  it('create position', async () => {
    const position = await invariant.getPosition(ALICE, 0n)

    chai.assert.deepEqual(position.liquidity, { v: 100000000000n })
    chai.assert.deepEqual(position.lowerTickIndex, lowerTickIndex)
    chai.assert.deepEqual(position.upperTickIndex, upperTickIndex)
    chai.assert.deepEqual(position.feeGrowthInsideX, { v: 0n })
    chai.assert.deepEqual(position.feeGrowthInsideY, { v: 0n })
    chai.assert.deepEqual(position.tokensOwedX, { v: 0n })
    chai.assert.deepEqual(position.tokensOwedY, { v: 0n })
  })

  it('remove position', async () => {
    await invariant.removePosition(ALICE, 0n)

    assertThrowsAsync(invariant.getPosition(ALICE, 0n), wasm.InvariantError.PositionNotFound)
    const positions = await invariant.getPositions(ALICE)
    chai.expect(positions.length).to.equal(0)

    assertThrowsAsync(invariant.getTick(poolKey, lowerTickIndex), wasm.InvariantError.TickNotFound)
    assertThrowsAsync(invariant.getTick(poolKey, upperTickIndex), wasm.InvariantError.TickNotFound)

    const isLowerTickInitialized = await invariant.isTickInitialized(poolKey, lowerTickIndex)
    chai.expect(isLowerTickInitialized).to.equal(false)

    const isUpperTickInitialized = await invariant.isTickInitialized(poolKey, upperTickIndex)
    chai.expect(isUpperTickInitialized).to.equal(false)
  })

  it('transfer position', async () => {
    await invariant.transferPosition(ALICE, 0n, Key.Account, bobAddress)

    assertThrowsAsync(invariant.getPosition(ALICE, 0n), wasm.InvariantError.PositionNotFound)
    const position = await invariant.getPosition(BOB, 0n)

    chai.assert.deepEqual(position.liquidity, { v: 100000000000n })
    chai.assert.deepEqual(position.lowerTickIndex, lowerTickIndex)
    chai.assert.deepEqual(position.upperTickIndex, upperTickIndex)
    chai.assert.deepEqual(position.feeGrowthInsideX, { v: 0n })
    chai.assert.deepEqual(position.feeGrowthInsideY, { v: 0n })
    chai.assert.deepEqual(position.tokensOwedX, { v: 0n })
    chai.assert.deepEqual(position.tokensOwedY, { v: 0n })
  })

  it('claim fee', async () => {
    const [tokenX, tokenY] = (await callWasm(
      wasm.isTokenX,
      token0ContractPackage,
      token1ContractPackage
    ))
      ? [token0Address, token1Address]
      : [token1Address, token0Address]
    const amount = 1000n

    erc20.setContractHash(tokenX)
    await erc20.mint(ALICE, Key.Account, bobAddress, amount)
    await erc20.approve(BOB, Key.Hash, invariantContractPackage, amount)

    const poolBefore = await invariant.getPool(poolKey)

    const targetSqrtPrice = { v: 15258932000000000000n }
    await invariant.swap(BOB, poolKey, true, { v: amount }, true, targetSqrtPrice)

    const poolAfter = await invariant.getPool(poolKey)
    erc20.setContractHash(tokenX)
    const bobTokenXAfter = await erc20.balanceOf(Key.Account, bobAddress)
    erc20.setContractHash(tokenY)
    const bobTokenYAfter = await erc20.balanceOf(Key.Account, bobAddress)

    chai.assert.equal(bobTokenXAfter, 0n)
    chai.assert.equal(bobTokenYAfter, 993n)

    await erc20.setContractHash(tokenX)
    const invariantTokenX = await erc20.balanceOf(Key.Hash, invariantAddress)
    await erc20.setContractHash(tokenY)
    const invariantTokenY = await erc20.balanceOf(Key.Hash, invariantAddress)

    chai.assert.equal(invariantTokenX, 1500n)
    chai.assert.equal(invariantTokenY, 7n)

    chai.assert.deepEqual(poolAfter.liquidity, poolBefore.liquidity)
    chai.assert.notDeepEqual(poolAfter.sqrtPrice, poolBefore.sqrtPrice)
    chai.assert.deepEqual(poolAfter.currentTickIndex, lowerTickIndex)
    chai.assert.deepEqual(poolAfter.feeGrowthGlobalX, { v: 50000000000000000000000n })
    chai.assert.deepEqual(poolAfter.feeGrowthGlobalY, { v: 0n })
    chai.assert.deepEqual(poolAfter.feeProtocolTokenX, { v: 1n })
    chai.assert.deepEqual(poolAfter.feeProtocolTokenY, { v: 0n })

    await erc20.setContractHash(tokenX)
    const positionOwnerBeforeX = await erc20.balanceOf(Key.Account, aliceAddress)
    const invariantBeforeX = await erc20.balanceOf(Key.Hash, invariantAddress)

    await invariant.claimFee(ALICE, 0n)

    await erc20.setContractHash(tokenX)
    const positionOwnerAfterX = await erc20.balanceOf(Key.Account, aliceAddress)
    const invariantAfterX = await erc20.balanceOf(Key.Hash, invariantAddress)

    const position = await invariant.getPosition(ALICE, 0n)
    const pool = await invariant.getPool(poolKey)
    const expectedTokensClaimed = 5n

    chai.assert.deepEqual(positionOwnerAfterX - expectedTokensClaimed, positionOwnerBeforeX)
    chai.assert.deepEqual(invariantAfterX + expectedTokensClaimed, invariantBeforeX)

    chai.assert.deepEqual(position.feeGrowthInsideX, pool.feeGrowthGlobalX)
    chai.assert.deepEqual(position.tokensOwedX, { v: 0n })
  })
})
