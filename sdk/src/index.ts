import { ALICE, BOB, LOCAL_NODE_URL, TEST, TESTNET_NODE_URL } from './consts'
import { Key, Network } from './enums'
import { Erc20 } from './erc20'
import { Invariant } from './invariant'
import { callWasm, createAccountKeys, initCasperClient, loadWasm } from './utils'
const main = async () => {
  const createKeys = false
  const wasm = await loadWasm()

  if (createKeys) {
    createAccountKeys()
    console.log('Account keys generated')
    return
  }

  const isLocal = true

  let account = ALICE
  let accountAddress = account.publicKey.toAccountHashStr().replace('account-hash-', '')
  const dummy = BOB
  /* eslint-disable-next-line @typescript-eslint/no-unused-vars */
  const dummyAddress = dummy.publicKey.toAccountHashStr().replace('account-hash-', '')
  let network = Network.Local
  let nodeUrl = LOCAL_NODE_URL

  if (!isLocal) {
    account = TEST
    /* eslint-disable-next-line @typescript-eslint/no-unused-vars */
    accountAddress = account.publicKey.toAccountHashStr().replace('account-hash-', '')
    network = Network.Testnet
    nodeUrl = TESTNET_NODE_URL
  }
  console.log(account, network)

  const client = initCasperClient(nodeUrl)

  let invariantAddress = '6f9672545b2600f4f135124bc5fcce3eabcf1d43d828a9c9a227434e13aedc8d'
  let invariantContractPackage = 'f34deac596aeb27b7b9d9418922d9e72ed28bf723a21b1c399c040346ab27d38'
  let invariantContract = await Invariant.load(client, invariantAddress, network)

  const [invariantContractPackageHash, invariantContractHash] = await Invariant.deploy(
    client,
    network,
    account,
    0n,
    2446489177947n
  )
  invariantContractPackage = invariantContractPackageHash
  invariantContract = await Invariant.load(client, invariantContractHash, network)
  invariantAddress = invariantContract.contract.contractHash?.replace('hash-', '') ?? ''

  if (isLocal) {
    const [invariantContractPackageHash, invariantContractHash] = await Invariant.deploy(
      client,
      network,
      account,
      0n,
      600000000000n
    )
    invariantContractPackage = invariantContractPackageHash
    invariantContract = await Invariant.load(client, invariantContractHash, network)
    invariantAddress = invariantContract.contract.contractHash?.replace('hash-', '') ?? ''
  }

  let token0Address = 'a6e5a67c7834df44c1923c346dfa6cef0df4be4932cbd9102779819633b885d5'
  let token0ContractPackage = '8a52cb3f956a94dd89635701e2225275ddf145f26394acf2414653dbb0db8699'
  let token0Contract = await Erc20.load(client, network, token0Address)
  let token1Address = 'ff1e3e482ddb5c021386acd7af168917159f434d5302463b748693c8db1c4592'
  let token1ContractPackage = 'a9129e520e38ba142d81cdeebf05691b0e404206820792209ae188fbdc15428d'
  let token1Contract = await Erc20.load(client, network, token1Address)

  if (isLocal) {
    const [token0ContractPackageHash, token0ContractHash] = await Erc20.deploy(
      client,
      network,
      account,
      '0',
      1000000000000000n,
      '',
      '',
      0n,
      300000000000n
    )
    const [token1ContractPackageHash, token1ContractHash] = await Erc20.deploy(
      client,
      network,
      account,
      '1',
      1000000000000000n,
      '',
      '',
      0n,
      300000000000n
    )
    token0ContractPackage = token0ContractPackageHash
    token1ContractPackage = token1ContractPackageHash
    token0Contract = await Erc20.load(client, network, token0ContractHash)
    token1Contract = await Erc20.load(client, network, token1ContractHash)
    token0Address = token0Contract.contract.contractHash!
    token1Address = token1Contract.contract.contractHash!
  }

  const feeTier = await callWasm(wasm.newFeeTier, { v: 6000000000n }, 10n)
  const poolKey = await callWasm(
    wasm.newPoolKey,
    token0ContractPackage,
    token1ContractPackage,
    feeTier
  )

  const addFeeTierResult = await invariantContract.addFeeTier(account, feeTier)
  console.log('addFeeTier', addFeeTierResult.execution_results[0].result)

  const createPoolResult = await invariantContract.createPool(account, poolKey, {
    v: 1000000000000000000000000n
  })

  console.log('createPool', createPoolResult.execution_results[0].result)

  const approveResult1 = await token0Contract.approve(
    account,
    Key.Hash,
    invariantContractPackage,
    1000000000000000n
  )
  console.log('approve', approveResult1.execution_results[0].result)

  const approveResult2 = await token1Contract.approve(
    account,
    Key.Hash,
    invariantContractPackage,
    1000000000000000n
  )
  console.log('approve', approveResult2.execution_results[0].result)

  const createPositionResult = await invariantContract.createPosition(
    account,
    poolKey,
    -10n,
    10n,
    { v: 1000000000000000n },
    { v: 1000000000000000000000000n },
    { v: 1000000000000000000000000n }
  )
  console.log('createPosition', createPositionResult.execution_results[0].result)

  console.log(
    'token 0 invariant balance',
    await token0Contract.balanceOf(Key.Hash, invariantContractPackage)
  )
  console.log(
    'token 1 invariant balance',
    await token1Contract.balanceOf(Key.Hash, invariantContractPackage)
  )

  const swapResult = await invariantContract.swap(account, poolKey, true, { v: 10n }, true, {
    v: 0n
  })
  console.log('swap', swapResult.execution_results[0].result)

  console.log(
    'token 0 invariant balance',
    await token0Contract.balanceOf(Key.Hash, invariantContractPackage)
  )
  console.log(
    'token 1 invariant balance',
    await token1Contract.balanceOf(Key.Hash, invariantContractPackage)
  )

  console.log(await invariantContract.getInvariantConfig())
  console.log(await invariantContract.getPosition(account, 0n))
  console.log(await invariantContract.getFeeTiers())
  console.log(await invariantContract.getPool(poolKey))
  console.log(await invariantContract.getPools())
  console.log(await invariantContract.isTickInitialized(poolKey, 10n))
  console.log(await invariantContract.getPositions(account))
}

main()
