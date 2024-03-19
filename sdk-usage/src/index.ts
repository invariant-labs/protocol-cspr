import {
  ALICE,
  BOB,
  Decimal,
  Erc20,
  Invariant,
  Key,
  LOCAL_NODE_URL,
  Network,
  Pool,
  Position,
  Tick,
  TokenAmount,
  calculateFee,
  calculateSqrtPriceAfterSlippage,
  getAccountHashFromKey,
  getLiquidityByY,
  initCasperClient,
  newFeeTier,
  newPoolKey,
  orderTokens,
  toDecimal
} from '@invariant-labs/cspr-sdk'

const main = async () => {
  // ###
  const client = initCasperClient(LOCAL_NODE_URL)

  const account = ALICE
  const accountAddress = getAccountHashFromKey(account)

  const receiver = BOB
  const receiverAddress = getAccountHashFromKey(receiver)

  const [invariantContractPackage, invariantContractHash] = await Invariant.deploy(
    client,
    Network.Local,
    account,
    { v: 0n },
    600000000000n
  )
  const [token0ContractPackage, token0ContractHash] = await Erc20.deploy(
    client,
    Network.Local,
    account,
    'erc20-1',
    1000000000000000n,
    'Coin',
    'COIN',
    12n,
    150000000000n
  )
  const [token1ContractPackage, token1ContractHash] = await Erc20.deploy(
    client,
    Network.Local,
    account,
    'erc20-2',
    1000000000000000n,
    'Coin',
    'COIN',
    12n,
    150000000000n
  )
  // ###

  // load invariant contract
  const invariant = await Invariant.load(client, invariantContractHash, Network.Local)

  // load token contract
  const erc20 = await Erc20.load(client, Network.Local, token0ContractHash)

  // set fee tier, make sture that fee tier with specified parameters exists
  const feeTier = await newFeeTier(await toDecimal(Decimal.Percentage, 1n, 2n), 1n) // fee: 0.01 = 1%, tick spacing: 1

  // if the fee tier does not exist, you have to add it
  const isAdded = await invariant.feeTierExist(feeTier)
  if (!isAdded) {
    await invariant.addFeeTier(account, feeTier)
  }

  // set initial square root of price of the pool, we set it to 1.00
  const initSqrtPrice = await toDecimal(Decimal.SqrtPrice, 1n, 0n)

  // set pool key, make sure that pool with speecified parameters does not exists
  const poolKey = await newPoolKey(token0ContractPackage, token1ContractPackage, feeTier)

  const createPoolResult = await invariant.createPool(account, poolKey, initSqrtPrice)

  // print transaction result
  console.log(createPoolResult.execution_results[0].result)

  // token y has 12 decimals and we want to add 8 actual tokens to our position
  const tokenYAmount: TokenAmount = { v: 8n * 10n ** 12n }

  // set lower and upper tick indexes, we want to create position in range [-10, 10]
  const lowerTickIndex = -10n
  const upperTickIndex = 10n

  // calculate amount of token x we need to give to create position
  const { amount: tokenXAmount, l: positionLiquidity } = await getLiquidityByY(
    tokenYAmount,
    lowerTickIndex,
    upperTickIndex,
    initSqrtPrice,
    true
  )

  // print amount of token x and y we need to give to create position based on parameters we passed
  console.log(tokenXAmount, tokenYAmount)

  const [tokenXContractHash, tokenYContractHash] = await orderTokens(
    token0ContractPackage,
    token1ContractPackage,
    token0ContractHash,
    token1ContractHash
  )

  // approve transfers of both tokens
  erc20.setContractHash(tokenXContractHash)
  await erc20.approve(account, Key.Hash, invariantContractPackage, tokenXAmount.v)
  erc20.setContractHash(tokenYContractHash)
  await erc20.approve(account, Key.Hash, invariantContractPackage, tokenYAmount.v)

  // ###
  const sqrtPriceLimit = await toDecimal(Decimal.SqrtPrice, 10n, 0n)
  // ###

  // create position
  const createPositionResult = await invariant.createPosition(
    account,
    poolKey,
    lowerTickIndex,
    upperTickIndex,
    positionLiquidity,
    initSqrtPrice,
    sqrtPriceLimit
  )
  console.log(createPositionResult.execution_results[0].result) // print transaction result

  // we want to swap 6 token0
  // token0 has 12 deciamls so we need to multiply it by 10^12
  const amount: TokenAmount = { v: 6n * 10n ** 12n }

  // approve token x transfer
  erc20.setContractHash(tokenXContractHash)
  await erc20.approve(account, Key.Hash, invariantContractPackage, amount.v)

  // ###
  const TARGET_SQRT_PRICE = await toDecimal(Decimal.SqrtPrice, 10n, 0n)
  // ###

  // slippage is a price change you are willing to accept,
  // for examples if current price is 1 and your slippage is 1%, then price limit will be 1.01
  const allowedSlippage = await toDecimal(Decimal.Percentage, 1n, 3n) // 0.001 = 0.1%

  // calculate sqrt price limit based on slippage
  const slippageLimit = await calculateSqrtPriceAfterSlippage(
    TARGET_SQRT_PRICE,
    allowedSlippage,
    false
  )

  const swapResult = await invariant.swap(account, poolKey, true, amount, true, slippageLimit)
  console.log(swapResult.execution_results[0].result) // print transaction result

  // query state
  const pool: Pool = await invariant.getPool(poolKey)
  const position: Position = await invariant.getPosition(account, 0n)
  const lowerTick: Tick = await invariant.getTick(poolKey, lowerTickIndex)
  const upperTick: Tick = await invariant.getTick(poolKey, upperTickIndex)

  // check amount of tokens that is available to claim
  const fees = await calculateFee(pool, position, lowerTick, upperTick)

  // print amount of unclaimed x and y tokens
  console.log(fees)

  // get balance of a specific token before claiming position fees and print it
  const accountBalanceBeforeClaim = await erc20.getBalanceOf(Key.Account, accountAddress)
  console.log(accountBalanceBeforeClaim)

  // specify position id
  const positionId = 0n
  // claim fee
  const claimFeeResult = await invariant.claimFee(account, positionId)
  // print transaction result
  console.log(claimFeeResult.execution_results[0].result)

  // get balance of a specific token after claiming position fees and print it
  const accountBalanceAfterClaim = await erc20.getBalanceOf(Key.Account, accountAddress)
  console.log(accountBalanceAfterClaim)

  const positionToTransfer = await invariant.getPosition(account, 0n)

  // transfer position from one account to another
  await invariant.transferPosition(account, 0n, Key.Account, receiverAddress)

  // get received position
  const receiverPosition = await invariant.getPosition(receiver, 0n)

  // ###
  await invariant.transferPosition(receiver, 0n, Key.Account, accountAddress)
  // ###

  // fetch user balances before removal
  const accountToken0BalanceBeforeRemove = await erc20.getBalanceOf(Key.Account, accountAddress)
  erc20.setContractHash(token1ContractHash)
  const accountToken1BalanceBeforeRemove = await erc20.getBalanceOf(Key.Account, accountAddress)
  console.log(accountToken0BalanceBeforeRemove, accountToken1BalanceBeforeRemove)

  // remove position
  const removePositionResult = await invariant.removePosition(account, 0n)
  console.log(removePositionResult.execution_results[0].result)

  // fetch user balances after removal
  erc20.setContractHash(token0ContractHash)
  const accountToken0BalanceAfterRemove = await erc20.getBalanceOf(Key.Account, accountAddress)
  erc20.setContractHash(token1ContractHash)
  const accountToken1BalanceAfterRemove = await erc20.getBalanceOf(Key.Account, accountAddress)

  // print balances
  console.log(accountToken0BalanceAfterRemove, accountToken1BalanceAfterRemove)
}

main()
