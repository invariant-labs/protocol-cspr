import { ALICE, LOCAL_NODE_URL, TEST, TESTNET_NODE_URL } from './consts'
import { Invariant } from './invariant'
import { Network } from './network'
import { createAccountKeys, initCasperClientAndService } from './utils'

const main = async () => {
  const createKeys = false

  if (createKeys) {
    createAccountKeys()
    console.log('Account keys generated')
    return
  }

  const isLocal = false

  let account
  let network
  let nodeUrl

  if (isLocal) {
    account = ALICE
    network = Network.Local
    nodeUrl = LOCAL_NODE_URL
  } else {
    account = TEST
    network = Network.Testnet
    nodeUrl = TESTNET_NODE_URL
  }

  const { client, service } = initCasperClientAndService(nodeUrl)

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

  // const invariantHash = await Invariant.deploy(client, service, network, account, 0n, 537395207813n)
  // // 2 500 000 000

  // console.log('Invariant deployed:', invariantHash)
  // HASH => 06466707c7d88d3835e1691bc8958b0e9c044631c5fd29d076b90be6c1e8f1a4 - with only tmp as U256
  const invariant = await Invariant.load(
    client,
    service,
    'bac526f448c0d376c93517ebfca3f9ceb8916b868120a2a68e917a8c11295df1'
  )
  console.log('Invariant loaded', invariant)

  // hash-bac526f448c0d376c93517ebfca3f9ceb8916b868120a2a68e917a8c11295df1
  await invariant.queryFields()
  // await invariant.changeProtocolFee(account, network, 100n)
  // await invariant.addFeeTier(account, network, 100n, 100n)
}

main()
