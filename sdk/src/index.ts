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

  // INVARIANT QUERIES HASH & CHANGE PROTOCOL FEE ENTRYPOINT: d262e503c5203e302ecfc8a31126a9c29783254116f6312d28200a44c1ce1c73
  // INVARIANT QUERIES HASH & CHANGE PROTOCOL FEE ENTRYPOINTS & PREINITIALIZED FEE TIER: 87573d38f1808d6eed4fe1b65eae56463fb5fc6eb3bc9e56add0ac78d69f1eca
  // INVARIANT QUERIES HASH & CHANGE PROTOCOL FEE ENTRYPOINTS & PREINITIALIZED FEE TIER & POOL: fb0a6c4c0d6b5a45b52fe7a05bbc3ffe87bfa4ea57f2b9722e179b4660a8b810
  // Mapping in Invariant with pool: 9195c4b7241c845bbc8bbe8801650fbd3c93d55e0c0145400edbdd5a7daa8a63 | RootHash: 582cc03e088113f1d83f7665a1b929a0ec3942c73de9db2efa74a740f18fe158
  // Maping in Invariant & nested mapping, both with pools: f930178220e1956abacdb2a39a8597025a2cf0e0d38878d08f57d0f70a5ca67f | RootHash: 03377d6489cf5e09cc239c571a1d4de3e0ae1ad64ad0d107cb9cebb6dc3253e0

  const invariantHash = 'f930178220e1956abacdb2a39a8597025a2cf0e0d38878d08f57d0f70a5ca67f'
  // const invariantHash = await Invariant.deploy(client, service, network, account, 0n, 288058232555n)
  // console.log('Invariant deployed:', invariantHash)

  const invariant = await Invariant.load(client, service, invariantHash)

  console.log('Invariant loaded')

  invariant.getFeeTiers()
  invariant.getPool()

  // {
  //   await invariant.changeProtocolFee(account, network, 200n)
  // const config = await invariant.getInvariantConfig()
  // console.log(config.invariantConfig.protocolFee.percentage)
  // }
  // {
  //   await invariant.changeProtocolFee(account, network, 99n)
  //   const config = await invariant.getInvariantConfig()
  //   console.log(config.invariantConfig.protocolFee.percentage)
  // }
}

main()
