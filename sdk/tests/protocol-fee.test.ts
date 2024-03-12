import { FeeTier, PoolKey } from 'invariant-cspr-wasm'
import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import { loadChai } from '../src/testUtils'
import { callWasm, initCasperClient, loadWasm } from '../src/utils'

let wasm: typeof import('invariant-cspr-wasm')
let chai: typeof import('chai')

const client = initCasperClient(LOCAL_NODE_URL)
let erc20: Erc20
let invariant: Invariant
let invariantContractPackage: string
let token0Address: string
let token1Address: string
let token0ContractPackage: string
let token1ContractPackage: string
const aliceAddress = ALICE.publicKey.toAccountHashStr().replace('account-hash-', '')
const bobAddress = BOB.publicKey.toAccountHashStr().replace('account-hash-', '')

const fee = 1000000000n
const tickSpacing = 1n

let feeTier: FeeTier
let poolKey: PoolKey

describe('protocol fee', () => {
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
    token0ContractPackage = token0ContractPackageHash
    token1ContractPackage = token1ContractPackageHash
    invariantContractPackage = invariantContractPackageHash

    erc20 = await Erc20.load(client, Network.Local, token0ContractHash)
    await erc20.approve(ALICE, Key.Hash, invariantContractPackage, fee)
    erc20 = await Erc20.load(client, Network.Local, token1ContractHash)
    await erc20.approve(ALICE, Key.Hash, invariantContractPackage, fee)

    invariant = await Invariant.load(client, Network.Local, invariantContractHash)

    feeTier = await callWasm(wasm.newFeeTier, { v: fee }, tickSpacing)
    poolKey = await callWasm(wasm.newPoolKey, token0ContractPackage, token1ContractPackage, feeTier)

    await invariant.addFeeTier(ALICE, feeTier)

    await invariant.createPool(ALICE, poolKey, { v: 1000000000000000000000000n })

    await invariant.createPosition(
      ALICE,
      poolKey,
      -10n,
      10n,
      { v: 10000000000000n },
      { v: 1000000000000000000000000n },
      { v: 1000000000000000000000000n }
    )

    await invariant.swap(ALICE, poolKey, true, { v: 4999n }, true, { v: 999505344804856076727628n })
  })

  it('should withdraw protocol fee', async () => {
    const feeTier = await callWasm(wasm.newFeeTier, { v: fee }, tickSpacing)
    const poolKey = await callWasm(
      wasm.newPoolKey,
      token0ContractPackage,
      token1ContractPackage,
      feeTier
    )

    erc20.setContractHash(token0Address)
    const token0Before = await erc20.balanceOf(Key.Account, aliceAddress)
    erc20.setContractHash(token1Address)
    const token1Before = await erc20.balanceOf(Key.Account, aliceAddress)

    const poolBefore = await invariant.getPool(poolKey)
    chai.assert.deepEqual(poolBefore.feeProtocolTokenX, { v: 1n })
    chai.assert.deepEqual(poolBefore.feeProtocolTokenY, { v: 0n })

    await invariant.withdrawProtocolFee(ALICE, poolKey)

    const poolAfter = await invariant.getPool(poolKey)
    chai.assert.deepEqual(poolAfter.feeProtocolTokenX, { v: 0n })
    chai.assert.deepEqual(poolAfter.feeProtocolTokenY, { v: 0n })

    erc20.setContractHash(token0Address)
    const token0After = await erc20.balanceOf(Key.Account, aliceAddress)
    erc20.setContractHash(token1Address)
    const token1After = await erc20.balanceOf(Key.Account, aliceAddress)

    if (await callWasm(wasm.isTokenX, token0ContractPackage, token1ContractPackage)) {
      chai.assert.equal(token0After, token0Before + 1n)
      chai.assert.equal(token1After, token1Before)
    } else {
      chai.assert.equal(token0After, token0Before)
      chai.assert.equal(token1After, token1Before + 1n)
    }
  })

  it('should change fee receiver', async () => {
    await invariant.changeFeeReceiver(ALICE, poolKey, Key.Account, bobAddress)

    erc20.setContractHash(token0Address)
    const token0Before = await erc20.balanceOf(Key.Account, bobAddress)
    erc20.setContractHash(token1Address)
    const token1Before = await erc20.balanceOf(Key.Account, bobAddress)

    const poolBefore = await invariant.getPool(poolKey)
    chai.assert.deepEqual(poolBefore.feeProtocolTokenX, { v: 1n })
    chai.assert.deepEqual(poolBefore.feeProtocolTokenY, { v: 0n })

    const withdrawProtocolFeeResult = await invariant.withdrawProtocolFee(ALICE, poolKey)
    chai.assert.notEqual(withdrawProtocolFeeResult.execution_results[0].result.Failure, undefined)

    await invariant.withdrawProtocolFee(BOB, poolKey)

    const poolAfter = await invariant.getPool(poolKey)
    chai.assert.deepEqual(poolAfter.feeProtocolTokenX, { v: 0n })
    chai.assert.deepEqual(poolAfter.feeProtocolTokenY, { v: 0n })

    erc20.setContractHash(token0Address)
    const token0After = await erc20.balanceOf(Key.Account, bobAddress)
    erc20.setContractHash(token1Address)
    const token1After = await erc20.balanceOf(Key.Account, bobAddress)

    if (await callWasm(wasm.isTokenX, token0ContractPackage, token1ContractPackage)) {
      chai.assert.equal(token0After, token0Before + 1n)
      chai.assert.equal(token1After, token1Before)
    } else {
      chai.assert.equal(token0After, token0Before)
      chai.assert.equal(token1After, token1Before + 1n)
    }
  })
})
