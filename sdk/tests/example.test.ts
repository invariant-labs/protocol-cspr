/* eslint-disable @typescript-eslint/no-unused-vars */

import { Pool, Position, SqrtPrice, Tick, TokenAmount } from 'invariant-cspr-wasm'
import { ALICE, BOB, LOCAL_NODE_URL } from '../src/consts'
import { Key, Network } from '../src/enums'
import { Erc20 } from '../src/erc20'
import { Invariant } from '../src/invariant'
import { calculateFee, getAccountHashFromKey, initCasperClient } from '../src/utils'
import { getLiquidityByY, isTokenX, newFeeTier, newPoolKey } from '../src/wasm'

describe('sdk guide snippets', () => {
  it('sdk guide', async () => {
    const client = initCasperClient(LOCAL_NODE_URL)

    const account = ALICE
    const accountAddress = getAccountHashFromKey(account)

    const receiver = BOB
    const receiverAddress = getAccountHashFromKey(receiver)

    const [INVARIANT_CONTRACT_PACKAGE, INVARIANT_CONTRACT_HASH] = await Invariant.deploy(
      client,
      Network.Local,
      account,
      0n,
      600000000000n
    )
    const [TOKEN_0_CONTRACT_PACKAGE, TOKEN_0_CONTRACT_HASH] = await Erc20.deploy(
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
    const [TOKEN_1_CONTRACT_PACKAGE, TOKEN_1_CONTRACT_HASH] = await Erc20.deploy(
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

    // load invariant contract
    const invariant = await Invariant.load(client, INVARIANT_CONTRACT_HASH, Network.Local)

    // load token contract
    const erc20 = await Erc20.load(client, Network.Local, TOKEN_0_CONTRACT_HASH)

    // set fee tier, make sture that fee tier with specified parameters exists
    const feeTier = await newFeeTier({ v: 1000000000n }, 1n) // fee: 0.01 = 1%, tick spacing: 1

    // if the fee tier does not exist, you have to add it
    const isAdded = await invariant.feeTierExist(feeTier)
    if (!isAdded) {
      await invariant.addFeeTier(account, feeTier)
    }

    // set initial price of the pool, we set it to 1.00, remember its square root of price
    const initSqrtPrice = { v: 1000000000000000000000000n }

    // set pool key, make sure that pool with speecified parameters does not exists
    const poolKey = await newPoolKey(TOKEN_0_CONTRACT_PACKAGE, TOKEN_1_CONTRACT_PACKAGE, feeTier)

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

    const [TOKEN_X_CONTRACT_HASH, TOKEN_Y_CONTRACT_HASH] = (await isTokenX(
      TOKEN_0_CONTRACT_PACKAGE,
      TOKEN_1_CONTRACT_PACKAGE
    ))
      ? [TOKEN_0_CONTRACT_HASH, TOKEN_1_CONTRACT_HASH]
      : [TOKEN_1_CONTRACT_HASH, TOKEN_0_CONTRACT_HASH]

    // approve transfers of both tokens
    erc20.setContractHash(TOKEN_X_CONTRACT_HASH)
    await erc20.approve(account, Key.Hash, INVARIANT_CONTRACT_PACKAGE, tokenXAmount.v)
    erc20.setContractHash(TOKEN_Y_CONTRACT_HASH)
    await erc20.approve(account, Key.Hash, INVARIANT_CONTRACT_PACKAGE, tokenYAmount.v)

    // create position
    const createPositionResult = await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      positionLiquidity,
      initSqrtPrice,
      {
        v: 10000000000000000000000000n
      }
    )
    console.log(createPositionResult.execution_results[0].result) // print transaction result

    // we want to swap 6 token0
    // token0 has 12 deciamls so we need to multiply it by 10^12
    const amount: SqrtPrice = { v: 6n * 10n ** 12n }

    // approve token x transfer
    erc20.setContractHash(TOKEN_X_CONTRACT_HASH)
    await erc20.approve(account, Key.Hash, INVARIANT_CONTRACT_PACKAGE, amount.v)

    const swapResult = await invariant.swap(account, poolKey, true, amount, true, {
      v: 0n
    })
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
    const accountBalanceBeforeClaim = await erc20.balanceOf(Key.Account, accountAddress)
    console.log(accountBalanceBeforeClaim)

    // specify position id
    const positionId = 0n
    // claim fee
    const claimFeeResult = await invariant.claimFee(account, positionId)
    // print transaction result
    console.log(claimFeeResult.execution_results[0].result)

    // get balance of a specific token after claiming position fees and print it
    const accountBalanceAfterClaim = await erc20.balanceOf(Key.Account, accountAddress)
    console.log(accountBalanceAfterClaim)

    const positionToTransfer = await invariant.getPosition(account, 0n)

    // transfer position from one account to another
    await invariant.transferPosition(account, 0n, Key.Account, receiverAddress)

    // get received position
    const receiverPosition = await invariant.getPosition(receiver, 0n)

    await invariant.transferPosition(receiver, 0n, Key.Account, accountAddress)

    // fetch user balances before removal
    const accountToken0BalanceBeforeRemove = await erc20.balanceOf(Key.Account, accountAddress)
    erc20.setContractHash(TOKEN_1_CONTRACT_HASH)
    const accountToken1BalanceBeforeRemove = await erc20.balanceOf(Key.Account, accountAddress)
    console.log(accountToken0BalanceBeforeRemove, accountToken1BalanceBeforeRemove)

    // remove position
    const removePositionResult = await invariant.removePosition(account, 0n)
    console.log(removePositionResult.execution_results[0].result)

    // fetch user balances after removal
    erc20.setContractHash(TOKEN_0_CONTRACT_HASH)
    const accountToken0BalanceAfterRemove = await erc20.balanceOf(Key.Account, accountAddress)
    erc20.setContractHash(TOKEN_1_CONTRACT_HASH)
    const accountToken1BalanceAfterRemove = await erc20.balanceOf(Key.Account, accountAddress)

    // print balances
    console.log(accountToken0BalanceAfterRemove, accountToken1BalanceAfterRemove)
  })

  it('sdk guide - using erc20', async () => {
    const client = initCasperClient(LOCAL_NODE_URL)

    const account = ALICE
    const accountAddress = getAccountHashFromKey(account)

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

    // load token by passing its address (you can use existing one), it allows you to interact with it
    const erc20 = await Erc20.load(client, Network.Local, token0ContractHash)

    // interact with token 0
    const account0Balance = await erc20.balanceOf(Key.Account, accountAddress)
    console.log(account0Balance)

    // if you want to interact with different token,
    // simply set different contract address
    erc20.setContractHash(token1ContractHash)

    // now we can interact with token y
    const account1Balance = await erc20.balanceOf(Key.Account, accountAddress)
    console.log(account1Balance)

    // fetch token metadata for previously deployed token0
    erc20.setContractHash(token0ContractHash)
    const token0Name = await erc20.name()
    const token0Symbol = await erc20.symbol()
    const token0Decimals = await erc20.decimals()
    console.log(token0Name, token0Symbol, token0Decimals)

    // load diffrent token and load its metadata
    erc20.setContractHash(token1ContractHash)
    const token1Name = await erc20.name()
    const token1Symbol = await erc20.symbol()
    const token1Decimals = await erc20.decimals()
    console.log(token1Name, token1Symbol, token1Decimals)
  })
})
