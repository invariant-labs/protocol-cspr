import { ALICE, LOCAL_NODE_URL } from '../src/consts'
import { Erc20 } from '../src/erc20'
import { Network } from '../src/network'
import { initCasperClientAndService } from '../src/utils'

describe('erc20', () => {
  it('should get metadata', async () => {
    const { client, service } = initCasperClientAndService(LOCAL_NODE_URL)

    const erc20Hash = await Erc20.deploy(
      client,
      service,
      Network.Local,
      ALICE,
      1000000000000n,
      'Coin',
      'COIN',
      6n,
      150000000000n
    )

    const erc20 = await Erc20.load(client, service, erc20Hash)
    expect(await erc20.name()).toEqual('Coin')
    expect(await erc20.symbol()).toEqual('COIN')
    expect(await erc20.decimals()).toEqual(6n)
  })
})
