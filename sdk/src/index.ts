import { ALICE, LOCAL_NODE_URL, TEST, TESTNET_INVARIANT_HASH, TESTNET_NODE_URL } from './consts'
import { Invariant } from './invariant'
import { Network } from './network'
import { createAccountKeys, initCasperClientAndService, loadWasm } from './utils'

const main = async () => {
  const createKeys = false
  const wasm = await loadWasm()

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

  // c34b7847a3fe4d5d12e4975b4eddfed10d25f0cb165d740a4a74606172d7c472
  // da1b9f07767375414fc7649ac8719be5d7104f49bc8c030bd51c45b0dbb22908

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

  console.log('Init SDK!')
  {
    const sqrtPriceScale = wasm.getSqrtPriceScale()
    const sqrtPriceDenominator = wasm.getSqrtPriceDenominator()
    const amount = wasm.toTokenAmount(1000, wasm.getTokenAmountScale())
    console.log(amount)
    console.log(sqrtPriceScale, sqrtPriceDenominator)

    const sqrtPriceA = { v: '234878324943782000000000000' }
    const sqrtPriceB = { v: '87854456421658000000000000' }
    const liquidity = { v: '983983249092' }
    const resultDown = wasm.getDeltaX(sqrtPriceA, sqrtPriceB, liquidity, false)
    const resultUp = wasm.getDeltaX(sqrtPriceA, sqrtPriceB, liquidity, true)
    console.log(resultDown, resultUp)
  }
  {
    const fee = 55n
    const tickSpacing = 10n
    const token0 = 'c34b7847a3fe4d5d12e4975b4eddfed10d25f0cb165d740a4a74606172d7c472'
    const token1 = 'da1b9f07767375414fc7649ac8719be5d7104f49bc8c030bd51c45b0dbb22908'
    const initSqrtPrice = 10n ** 24n
    const initTick = 0n
    console.log(initSqrtPrice, initTick)
    console.log(token0, token1)
    const poolKey = {
      tokenX: token0,
      tokenY: token1,
      feeTier: {
        fee,
        tickSpacing
      }
    }
    // await invariant.addFeeTier(account, network, 55n, 10n)
    const feeTiers = await invariant.getFeeTiers()
    console.log(feeTiers)
    // await invariant.createPool(
    //   account,
    //   network,
    //   token0,
    //   token1,
    //   fee,
    //   tickSpacing,
    //   initSqrtPrice,
    //   initTick
    // )
    let pool = await invariant.getPool(poolKey)
    console.log(pool)
    // await invariant.changeFeeReceiver(
    //   account,
    //   network,
    //   token0,
    //   token1,
    //   fee,
    //   tickSpacing,
    //   'da1b9f07767375414fc7649ac8719be5d7104f49bc8c030bd51c45b0dbb22908'
    // )
    pool = await invariant.getPool(poolKey)
    console.log(pool)
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
