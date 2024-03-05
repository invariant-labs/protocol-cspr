import { ALICE, LOCAL_NODE_URL, TEST, TESTNET_INVARIANT_HASH, TESTNET_NODE_URL } from './consts'
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

  const invariantHash = TESTNET_INVARIANT_HASH
  // const invariantHash = await Invariant.deploy(
  //   client,
  //   service,
  //   network,
  //   account,
  //   0n,
  //   TESTNET_DEPLOY_AMOUNT
  // )
  // console.log('Invariant deployed:', invariantHash)

  const invariant = await Invariant.load(client, service, invariantHash)

  {
    await invariant.addFeeTier(account, network, 55n, 10n)
    let feeTiers = await invariant.getFeeTiers()
    console.log(feeTiers)
    await invariant.removeFeeTier(account, network, 55n, 10n)
    feeTiers = await invariant.getFeeTiers()
    console.log(feeTiers)
  }
  console.log('Invariant loaded')

  // const config = await invariant.getInvariantConfig()

  // console.log(feeTiers)
  // console.log(config)

  // const poolKey = {
  //   tokenX: '0101010101010101010101010101010101010101010101010101010101010101',
  //   tokenY: '0202020202020202020202020202020202020202020202020202020202020202',
  //   feeTier: {
  //     tickSpacing: 10n,
  //     fee: 100n
  //   }
  // }

  // const pool = await invariant.getPool(poolKey)
  // console.log(pool)

  // {
  //   await invariant.changeProtocolFee(account, network, 200n)
  //   const config = await invariant.getInvariantConfig()
  //   console.log(config)
  // }
}

main()
