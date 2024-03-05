import { dynamicImport } from 'tsimportlib'
import { ALICE, LOCAL_NODE_URL } from './consts'
import { Erc20 } from './erc20'
import { Network } from './network'
import { initCasperClientAndService } from './utils'

const loadWasm = async () => {
  return (await dynamicImport(
    'invariant-cspr-wasm',
    module
  )) as typeof import('invariant-cspr-wasm')
}

const main = async () => {
  const { getPercentageDenominator } = await loadWasm()
  console.log(getPercentageDenominator())

  const { client, service } = initCasperClientAndService(LOCAL_NODE_URL)
  const erc20Hash = await Erc20.deploy(
    client,
    service,
    Network.Local,
    ALICE,
    1000000000000n,
    'COIN',
    'Coin',
    6n,
    150000000000n
  )

  const erc20 = await Erc20.load(client, service, erc20Hash)
  console.log(await erc20.name())

  // const createKeys = false

  // if (createKeys) {
  //   createAccountKeys()
  //   return
  // }

  // const isLocal = true

  // let account
  // let network
  // let nodeUrl

  // if (isLocal) {
  //   account = ALICE
  //   network = Network.Local
  //   nodeUrl = LOCAL_NODE_URL
  // } else {
  //   account = TEST
  //   network = Network.Testnet
  //   nodeUrl = TESTNET_NODE_URL
  // }

  // const { client, service } = initCasperClientAndService(nodeUrl)

  // const erc20Hash = await Erc20.deploy(
  //   client,
  //   service,
  //   network,
  //   account,
  //   1000000000000n,
  //   'COIN',
  //   'Coin',
  //   6n,
  //   150000000000n
  // )

  // const erc20 = await Erc20.load(client, service, erc20Hash)
  // console.log(await erc20.name())

  // console.log(await erc20.balance_of(account.publicKey))
  // await erc20.transfer(account, network, BOB.publicKey, 2500000000n)
  // console.log(await erc20.balance_of(account.publicKey))

  // const invariantHash = await Invariant.deploy(
  //   client,
  //   service,
  //   network,
  //   account,
  //   0n,
  //   10000000000000n
  // )

  // const invariant = await Invariant.load(client, service, invariantHash)
  // await invariant.changeProtocolFee(account, network, 100n)
  // await invariant.addFeeTier(account, network, 100n, 100n)
}

main()
