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

  // const invariantHash = await Invariant.deploy(client, service, network, account, 0n, 357986471289n)
  // // 2 500 000 000

  // console.log('Invariant deployed:', invariantHash)
  const invariant = await Invariant.load(
    client,
    service,
    'd262e503c5203e302ecfc8a31126a9c29783254116f6312d28200a44c1ce1c73'
  )

  console.log('Invariant loaded')

  // INVARIANT QUERIES HASH * CHANGE PROTOCOL FEE ENTRYPOINT: hash-d262e503c5203e302ecfc8a31126a9c29783254116f6312d28200a44c1ce1c73
  {
    await invariant.changeProtocolFee(account, network, 200n)
    const config = await invariant.getInvariantConfig()
    console.log(config.invariantConfig.protocolFee.percentage)
  }
  {
    await invariant.changeProtocolFee(account, network, 99n)
    const config = await invariant.getInvariantConfig()
    console.log(config.invariantConfig.protocolFee.percentage)
  }
}

main()
