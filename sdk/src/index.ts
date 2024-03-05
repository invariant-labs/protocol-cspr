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

  const invariantHash = 'fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810'
  // const invariantHash = await Invariant.deploy(client, service, network, account, 0n, 288058232555n)
  // console.log('Invariant deployed:', invariantHash)

  const invariant = await Invariant.load(client, service, invariantHash)

  console.log('Invariant loaded')

  const feeTiers = await invariant.getFeeTiers()
  const pool = await invariant.getPool()
  const config = await invariant.getInvariantConfig()

  console.log(pool)
  console.log(feeTiers)
  console.log(config)

  // {
  //   await invariant.changeProtocolFee(account, network, 200n)
  //   const config = await invariant.getInvariantConfig()
  //   console.log(config.invariantConfig.protocolFee.percentage)
  // }
  // {
  //   await invariant.changeProtocolFee(account, network, 99n)
  //   const config = await invariant.getInvariantConfig()
  //   console.log(config.invariantConfig.protocolFee.percentage)
  // }
}

main()
