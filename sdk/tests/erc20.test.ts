import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { initCasperClientAndService } from '../src/utils'

const { client, service } = initCasperClientAndService(LOCAL_NODE_URL)
let erc20: Erc20
const aliceAddress = ALICE.publicKey.toAccountHashStr().replace('account-hash-', '')
const bobAddress = BOB.publicKey.toAccountHashStr().replace('account-hash-', '')

describe('erc20', () => {
  beforeEach(async () => {
    const [, erc20ContractHash] = await Erc20.deploy(
      client,
      service,
      Network.Local,
      ALICE,
      'erc20',
      1000n,
      'Coin',
      'COIN',
      12n,
      150000000000n
    )

    erc20 = await Erc20.load(client, service, Network.Local, erc20ContractHash)
  })

  it('should set metadata', async () => {
    expect(await erc20.name()).toEqual('Coin')
    expect(await erc20.symbol()).toEqual('COIN')
    expect(await erc20.decimals()).toEqual(12n)
  })

  it('should mint tokens', async () => {
    await erc20.mint(ALICE, Key.Account, aliceAddress, 500n)
    expect(await erc20.balanceOf(Key.Account, aliceAddress)).toEqual(1500n)
  })

  it('should transfer tokens', async () => {
    await erc20.transfer(ALICE, Key.Account, bobAddress, 250n)
    expect(await erc20.balanceOf(Key.Account, aliceAddress)).toEqual(750n)
    expect(await erc20.balanceOf(Key.Account, bobAddress)).toEqual(250n)
  })

  it('should change instance', async () => {
    const [, erc20ContractHash] = await Erc20.deploy(
      client,
      service,
      Network.Local,
      ALICE,
      'erc20',
      1000n,
      'Coin',
      'COIN',
      12n,
      150000000000n
    )

    await erc20.transfer(ALICE, Key.Account, bobAddress, 250n)
    erc20.setContractHash(erc20ContractHash)
    expect(await erc20.balanceOf(Key.Account, aliceAddress)).toEqual(1000n)
  })

  it('should approve tokens', async () => {
    await erc20.approve(ALICE, Key.Account, bobAddress, 250n)
    expect(await erc20.allowance(Key.Account, aliceAddress, Key.Account, bobAddress)).toEqual(250n)
  })
})
