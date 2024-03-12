import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { loadChai } from '../src/testUtils'
import { getAccountHashFromKey, initCasperClient } from '../src/utils'

const client = initCasperClient(LOCAL_NODE_URL)
let erc20: Erc20
const aliceAddress = getAccountHashFromKey(ALICE)
const bobAddress = getAccountHashFromKey(BOB)

describe('erc20', () => {
  beforeEach(async () => {
    const [, erc20ContractHash] = await Erc20.deploy(
      client,
      Network.Local,
      ALICE,
      'erc20',
      1000n,
      'Coin',
      'COIN',
      12n,
      150000000000n
    )

    erc20 = await Erc20.load(client, Network.Local, erc20ContractHash)
  })

  it('should set metadata', async () => {
    const chai = await loadChai()
    chai.assert.equal(await erc20.name(), 'Coin')
    chai.assert.equal(await erc20.symbol(), 'COIN')
    chai.assert.equal(await erc20.decimals(), 12n)
  })

  it('should mint tokens', async () => {
    const chai = await loadChai()
    await erc20.mint(ALICE, Key.Account, aliceAddress, 500n)
    chai.assert.equal(await erc20.balanceOf(Key.Account, aliceAddress), 1500n)
  })

  it('should transfer tokens', async () => {
    const chai = await loadChai()
    await erc20.transfer(ALICE, Key.Account, bobAddress, 250n)
    chai.assert.equal(await erc20.balanceOf(Key.Account, aliceAddress), 750n)
    chai.assert.equal(await erc20.balanceOf(Key.Account, bobAddress), 250n)
  })

  it('should change instance', async () => {
    const chai = await loadChai()
    const [, erc20ContractHash] = await Erc20.deploy(
      client,
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
    chai.assert.equal(await erc20.balanceOf(Key.Account, aliceAddress), 1000n)
  })

  it('should approve tokens', async () => {
    const chai = await loadChai()
    await erc20.approve(ALICE, Key.Account, bobAddress, 250n)
    chai.assert.equal(
      await erc20.allowance(Key.Account, aliceAddress, Key.Account, bobAddress),
      250n
    )
  })
})
