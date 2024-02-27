import { ALICE, LOCAL_NODE_URL, TEST, TESTNET_NODE_URL } from './consts'
import { Invariant } from './invariant'
import { Network } from './network'
import { createAccountKeys, initCasperClientAndService } from './utils'

const main = async () => {
  const createKeys = false

  if (createKeys) {
    createAccountKeys()
    return
  }

  const isLocal = true

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

  const invariantHash = await Invariant.deploy(
    client,
    service,
    network,
    account,
    0n,
    10000000000000n
  )

  const invariant = await Invariant.load(client, service, invariantHash)
  console.log(invariant)
  await invariant.changeProtocolFee(account, network, 100n)
  const queryResult = await invariant.getProtocolFee(account, network)
  // const queryResult = await invariant.getProtocolFee()
  console.log(queryResult)
  console.log(queryResult.execution_results[0].result)
  console.log(queryResult.deploy)
  console.log(queryResult.deploy.payment)

  console.log()
}

main()
