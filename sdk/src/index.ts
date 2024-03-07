import { ALICE, BOB, LOCAL_NODE_URL, TEST, TESTNET_NODE_URL } from './consts'
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

  const isLocal = true

  let account = ALICE
  let accountAddress = account.publicKey.toAccountHashStr().replace('account-hash-', '')
  const dummy = BOB
  const dummyAddress = dummy.publicKey.toAccountHashStr().replace('account-hash-', '')
  let network = Network.Local
  let nodeUrl = LOCAL_NODE_URL

  if (!isLocal) {
    account = TEST
    accountAddress = account.publicKey.toAccountHashStr().replace('account-hash-', '')
    network = Network.Testnet
    nodeUrl = TESTNET_NODE_URL
  }

  const { client, service } = initCasperClientAndService(nodeUrl)

  let invariantAddress = '6f9672545b2600f4f135124bc5fcce3eabcf1d43d828a9c9a227434e13aedc8d'
  let invariantContract = await Invariant.load(client, service, invariantAddress)

  if (isLocal) {
    const invariantContractHash = await Invariant.deploy(
      client,
      service,
      network,
      account,
      0n,
      600000000000n
    )
    invariantContract = await Invariant.load(client, service, invariantContractHash)
    invariantAddress = invariantContract.contract.contractHash?.replace('hash-', '') ?? ''
  }

  let token0Address = 'a6e5a67c7834df44c1923c346dfa6cef0df4be4932cbd9102779819633b885d5'
  let token0Contract = await Erc20.load(client, service, token0Address)
  let token1Address = 'ff1e3e482ddb5c021386acd7af168917159f434d5302463b748693c8db1c4592'
  let token1Contract = await Erc20.load(client, service, token1Address)

  if (isLocal) {
    const token0ContractHash = await Erc20.deploy(
      client,
      service,
      network,
      account,
      '0',
      1000000000000000n,
      '',
      '',
      0n,
      300000000000n
    )
    const token1ContractHash = await Erc20.deploy(
      client,
      service,
      network,
      account,
      '1',
      1000000000000000n,
      '',
      '',
      0n,
      300000000000n
    )
    token0Contract = await Erc20.load(client, service, token0ContractHash)
    token1Contract = await Erc20.load(client, service, token1ContractHash)
    token0Address = token0Contract.contract.contractHash?.replace('hash-', '') ?? ''
    token1Address = token1Contract.contract.contractHash?.replace('hash-', '') ?? ''
  }

  console.log('balance', await token0Contract.balanceOf(account.publicKey))
  console.log('balance', await token1Contract.balanceOf(account.publicKey))

  const approveResult = await token0Contract.approve(
    account,
    network,
    dummy.publicKey,
    1000000000000000n
  )
  console.log('approve', approveResult.execution_results[0].result)

  console.log('allowance', await token0Contract.allowance(accountAddress, dummyAddress))

  await token0Contract.transferFrom(
    dummy,
    network,
    account.publicKey,
    dummy.publicKey,
    1000000000000000n
  )
  console.log('approve', approveResult.execution_results[0].result)

  console.log('balance', await token0Contract.balanceOf(account.publicKey))
  console.log('balance', await token0Contract.balanceOf(BOB.publicKey))
}

main()
