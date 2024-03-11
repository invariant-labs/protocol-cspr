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

  const tokens = (await callWasm(wasm.isTokenX, token0ContractPackage, token1ContractPackage))
    ? {
        tokenX: {
          loadHash: token0ContractHash,
          packageHash: token0ContractPackage
        },
        tokenY: {
          loadHash: token1ContractHash,
          packageHash: token1ContractPackage
        }
      }
    : {
        tokenX: {
          loadHash: token1ContractHash,
          packageHash: token1ContractPackage
        },
        tokenY: {
          loadHash: token0ContractHash,
          packageHash: token0ContractPackage
        }
      }

  return {
    invariant: {
      loadHash: invariantContractHash,
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
