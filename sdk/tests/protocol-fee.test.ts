import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import { callWasm, initCasperClientAndService, loadWasm } from '../src/utils'

let wasm: typeof import('invariant-cspr-wasm')

const { client, service } = initCasperClientAndService(LOCAL_NODE_URL)
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

describe('protocol fee', () => {
  beforeAll(async () => {
    wasm = await loadWasm()
  })

  beforeEach(async () => {
    const [token0ContractPackageHash, token0ContractHash] = await Erc20.deploy(
      client,
      service,
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
      service,
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
      service,
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

    erc20 = await Erc20.load(client, service, Network.Local, token0ContractHash)
    await erc20.approve(ALICE, Key.Hash, invariantContractPackage, fee)
    erc20 = await Erc20.load(client, service, Network.Local, token1ContractHash)
    await erc20.approve(ALICE, Key.Hash, invariantContractPackage, fee)

    invariant = await Invariant.load(client, service, Network.Local, invariantContractHash)

    await invariant.addFeeTier(ALICE, fee, tickSpacing)

    await invariant.createPool(
      ALICE,
      token0ContractPackage,
      token1ContractPackage,
      fee,
      tickSpacing,
      1000000000000000000000000n,
      0n
    )

    await invariant.createPosition(
      ALICE,
      token0ContractPackage,
      token1ContractPackage,
      fee,
      tickSpacing,
      -10n,
      10n,
      10000000000000n,
      1000000000000000000000000n,
      1000000000000000000000000n
    )

    await invariant.swap(
      ALICE,
      token0ContractPackage,
      token1ContractPackage,
      fee,
      tickSpacing,
      true,
      4999n,
      true,
      999505344804856076727628n
    )
  })

  it('should withdraw protocol fee', async () => {
    erc20.setContractHash(token0Address)
    const token0Before = await erc20.balanceOf(Key.Account, aliceAddress)
    erc20.setContractHash(token1Address)
    const token1Before = await erc20.balanceOf(Key.Account, aliceAddress)

    // TODO: get pool before and assert protocol protocol fee

    await invariant.withdrawProtocolFee(
      ALICE,
      token0ContractPackage,
      token1ContractPackage,
      fee,
      tickSpacing
    )

    // TODO: get pool after and assert protocol protocol fee

    erc20.setContractHash(token0Address)
    const token0After = await erc20.balanceOf(Key.Account, aliceAddress)
    erc20.setContractHash(token1Address)
    const token1After = await erc20.balanceOf(Key.Account, aliceAddress)

    if (await callWasm(wasm.isTokenX, token0ContractPackage, token1ContractPackage)) {
      expect(token0After).toBe(token0Before + 1n)
      expect(token1After).toBe(token1Before)
    } else {
      expect(token0After).toBe(token0Before)
      expect(token1After).toBe(token1Before + 1n)
    }
  })

  //   it('should change fee receiver', async () => {
  //     await invariant.changeFeeReceiver(
  //       ALICE,
  //       token0ContractPackage,
  //       token1ContractPackage,
  //       fee,
  //       tickSpacing,
  //       bobAddress
  //     )

  //     erc20.setContractHash(token0Address)
  //     const token0Before = await erc20.balanceOf(Key.Account, bobAddress)
  //     erc20.setContractHash(token1Address)
  //     const token1Before = await erc20.balanceOf(Key.Account, bobAddress)

  //     // TODO: get pool before and assert protocol protocol fee

  //     const withdrawProtocolFeeResult = await invariant.withdrawProtocolFee(
  //       ALICE,
  //       token0ContractPackage,
  //       token1ContractPackage,
  //       fee,
  //       tickSpacing
  //     )
  //     expect(withdrawProtocolFeeResult.execution_results[0].result.Failure).toBeDefined()

  //     await invariant.withdrawProtocolFee(
  //       BOB,
  //       token0ContractPackage,
  //       token1ContractPackage,
  //       fee,
  //       tickSpacing
  //     )

  //     // TODO: get pool after and assert protocol protocol fee

  //     erc20.setContractHash(token0Address)
  //     const token0After = await erc20.balanceOf(Key.Account, bobAddress)
  //     erc20.setContractHash(token1Address)
  //     const token1After = await erc20.balanceOf(Key.Account, bobAddress)

  //     expect(await erc20.balanceOf(Key.Account, aliceAddress)).toBe(999945015n)
  //     expect(await erc20.balanceOf(Key.Account, aliceAddress)).toBe(999945015n)
  //     expect(token0After + token1After).toBe(100n)

  //     if (await callWasm(wasm.isTokenX, token0ContractPackage, token1ContractPackage)) {
  //       expect(token0After).toBe(token0Before + 1n)
  //       expect(token1After).toBe(token1Before)
  //     } else {
  //       expect(token0After).toBe(token0Before)
  //       expect(token1After).toBe(token1Before + 1n)
  //     }
  //   })
})
