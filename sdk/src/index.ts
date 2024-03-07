import { ALICE, LOCAL_NODE_URL, TEST, TESTNET_NODE_URL } from './consts'
import { Erc20 } from './erc20'
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

  // const invariantContractHash = await Invariant.deploy(
  //   client,
  //   service,
  //   network,
  //   account,
  //   0n,
  //   1000000000000n
  // )
  // console.log(invariantContractHash)
  const invariantContract = await Invariant.load(
    client,
    service,
    '6f9672545b2600f4f135124bc5fcce3eabcf1d43d828a9c9a227434e13aedc8d'
  )

  // const token0ContractHash = await Erc20.deploy(
  //   client,
  //   service,
  //   network,
  //   account,
  //   '0',
  //   1000n,
  //   '',
  //   '',
  //   0n,
  //   300000000000n
  // )
  // const token1ContractHash = await Erc20.deploy(
  //   client,
  //   service,
  //   network,
  //   account,
  //   '1',
  //   1000n,
  //   '',
  //   '',
  //   0n,
  //   300000000000n
  // )
  // console.log(token0ContractHash, token1ContractHash)
  const token0Contract = await Erc20.load(
    client,
    service,
    '18936cd633a90f38b1989dceb669059b9afafec93c0cef5ddc1f4f15f9f14168'
  )
  const token1Contract = await Erc20.load(
    client,
    service,
    'ff1e3e482ddb5c021386acd7af168917159f434d5302463b748693c8db1c4592'
  )

  // const token0Address = token0Contract.contract.contractHash?.replace('hash-', '') ?? ''
  // const token1Address = token1Contract.contract.contractHash?.replace('hash-', '') ?? ''
  // const token0Address = 'a9129e520e38ba142d81cdeebf05691b0e404206820792209ae188fbdc15428d'
  // const token1Address = '8b64d645c83ec910e58a900a80b62013794fe0d1f8d36a34ed3a8ad94e3d46e7'
  // const [tokenX, tokenY] =
  //   token0Address < token1Address ? [token0Address, token1Address] : [token1Address, token0Address]

  // const fee = 100n
  // const tickSpacing = 10n

  // const addFeeTierResult = await invariantContract.addFeeTier(account, network, fee, tickSpacing)
  // console.log('addFeeTier', addFeeTierResult.execution_results[0].result)

  // const feeTiers = await invariantContract.getFeeTiers()
  // console.log(feeTiers)

  // const createPoolResult = await invariantContract.createPool(
  //   account,
  //   network,
  //   tokenX,
  //   tokenY,
  //   fee,
  //   tickSpacing,
  //   1000000000000000000000000n,
  //   0n
  // )
  // console.log('createPool', createPoolResult.execution_results[0].result)

  // const poolBeforePosition = await invariantContract.getPool({
  //   tokenX,
  //   tokenY,
  //   feeTier: {
  //     fee,
  //     tickSpacing
  //   }
  // })
  // console.log(poolBeforePosition)

  const token0UserBalance = await token0Contract.balance_of(account.publicKey)
  console.log('token 0 balance', token0UserBalance)
  const token1UserBalance = await token1Contract.balance_of(account.publicKey)
  console.log('token 1 balance', token1UserBalance)

  const approveResult = await token0Contract.approve(
    account,
    network,
    '6f9672545b2600f4f135124bc5fcce3eabcf1d43d828a9c9a227434e13aedc8d',
    1000n
  )
  console.log('approve', approveResult.execution_results[0].result)

  const invariantAllowance = await token0Contract.allowance(
    '6796ab4158be14efcb3db532e3311123925a2a24f2add0d93eda0f396e4aee5f',
    '6f9672545b2600f4f135124bc5fcce3eabcf1d43d828a9c9a227434e13aedc8d'
  )
  console.log('allowance', invariantAllowance)

  // const createPositionResult = await invariantContract.createPosition(
  //   account,
  //   network,
  //   tokenX,
  //   tokenY,
  //   fee,
  //   tickSpacing,
  //   -10n,
  //   10n,
  //   10000n,
  //   1000000000000000000000000n,
  //   1000000000000000000000000n
  // )
  // console.log('createposition ', createPositionResult.execution_results[0].result)

  // const poolAfterPosition = await invariantContract.getPool({
  //   tokenX,
  //   tokenY,
  //   feeTier: {
  //     fee,
  //     tickSpacing
  //   }
  // })
  // console.log(poolAfterPosition)
}

main()
