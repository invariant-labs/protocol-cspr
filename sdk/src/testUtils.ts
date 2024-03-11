import { CasperClient, CasperServiceByJsonRPC, Keys } from 'casper-js-sdk'
import type { InvariantError, Position } from 'invariant-cspr-wasm'
import { Network } from './enums'
import { Erc20 } from './erc20'
import { Invariant } from './invariant'
import { callWasm, loadWasm } from './utils'

export const deployInvariantAndTokens = async (
  client: CasperClient,
  service: CasperServiceByJsonRPC,
  account: Keys.AsymmetricKey,
  initialFee: bigint = 0n,
  intialSupply: bigint = 1000000000000000n
) => {
  const wasm = await loadWasm()
  const [invariantContractPackageHash, invariantContractHash] = await Invariant.deploy(
    client,
    service,
    Network.Local,
    account,
    initialFee,
    600000000000n
  )
  const invariantContractPackage = invariantContractPackageHash
  const invariantContract = await Invariant.load(client, service, invariantContractHash)
  const invariantAddress = invariantContract.contract.contractHash?.replace('hash-', '') ?? ''

  const [token0ContractPackageHash, token0ContractHash] = await Erc20.deploy(
    client,
    service,
    Network.Local,
    account,
    '0',
    intialSupply,
    '',
    '',
    0n,
    300000000000n
  )
  const [token1ContractPackageHash, token1ContractHash] = await Erc20.deploy(
    client,
    service,
    Network.Local,
    account,
    '1',
    intialSupply,
    '',
    '',
    0n,
    300000000000n
  )
  const token0ContractPackage = token0ContractPackageHash
  const token1ContractPackage = token1ContractPackageHash
  const token0Contract = await Erc20.load(client, service, token0ContractHash)
  const token1Contract = await Erc20.load(client, service, token1ContractHash)
  const token0Address = token0Contract.contract.contractHash!
  const token1Address = token1Contract.contract.contractHash!

  const tokens = (await callWasm(wasm.isTokenX, token0ContractPackage, token1ContractPackage))
    ? {
        tokenX: {
          contract: token0Contract,
          address: token0Address,
          packageHash: token0ContractPackage
        },
        tokenY: {
          contract: token1Contract,
          address: token1Address,
          packageHash: token1ContractPackage
        }
      }
    : {
        tokenX: {
          contract: token1Contract,
          address: token1Address,
          packageHash: token1ContractPackage
        },
        tokenY: {
          contract: token0Contract,
          address: token0Address,
          packageHash: token0ContractPackage
        }
      }

  return {
    invariant: {
      contract: invariantContract,
      address: invariantAddress,
      packageHash: invariantContractPackage
    },
    ...tokens
  }
}

export const assertThrowsAsync = async (fn: Promise<any>, word?: InvariantError) => {
  try {
    await fn
  } catch (e: any) {
    if (word) {
      const err = e.toString()
      console.log(err)
      const regex = new RegExp(`${word}$`)
      if (!regex.test(err)) {
        console.log(err)
        throw new Error('Invalid Error message')
      }
    }
    return
  }
  throw new Error('Function did not throw error')
}

export const positionEquals = (position: Position, expectedPosition: Position) => {
  expect(position.poolKey.tokenX).toEqual(expectedPosition.poolKey.tokenX)
  expect(position.poolKey.tokenY).toEqual(expectedPosition.poolKey.tokenY)
  expect(position.poolKey.feeTier.fee.v).toEqual(expectedPosition.poolKey.feeTier.fee.v)
  expect(position.poolKey.feeTier.tickSpacing).toEqual(expectedPosition.poolKey.feeTier.tickSpacing)
  expect(position.lowerTickIndex).toEqual(expectedPosition.lowerTickIndex)
  expect(position.upperTickIndex).toEqual(expectedPosition.upperTickIndex)
  expect(position.liquidity.v).toEqual(expectedPosition.liquidity.v)
  expect(position.feeGrowthInsideX.v).toEqual(expectedPosition.feeGrowthInsideX.v)
  expect(position.feeGrowthInsideY.v).toEqual(expectedPosition.feeGrowthInsideY.v)
  expect(position.tokensOwedX.v).toEqual(expectedPosition.tokensOwedX.v)
  expect(position.tokensOwedY.v).toEqual(expectedPosition.tokensOwedY.v)
}
