import { CasperClient, Keys } from 'casper-js-sdk'
import { dynamicImport } from 'tsimportlib'
import type { InvariantError, Percentage, Position } from 'wasm'
import { Erc20 } from './erc20'
import { Invariant } from './invariant'
import { Network } from './schema'
import { callWasm, loadWasm } from './utils'

export interface DeployedContractsHashes {
  invariant: { loadHash: string; packageHash: string }
  tokenX: { loadHash: string; packageHash: string }
  tokenY: { loadHash: string; packageHash: string }
}
export const deployInvariantAndTokens = async (
  client: CasperClient,
  account: Keys.AsymmetricKey,
  initialFee: Percentage = { v: 0n },
  intialSupply: bigint = 1000000000000000n
): Promise<DeployedContractsHashes> => {
  const wasm = await loadWasm()
  const [invariantContractPackageHash, invariantContractHash] = await Invariant.deploy(
    client,
    Network.Local,
    account,
    initialFee,
    600000000000n
  )
  const invariantContractPackage = invariantContractPackageHash

  const [token0ContractPackageHash, token0ContractHash] = await Erc20.deploy(
    client,
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

export const positionEquals = async (position: Position, expectedPosition: Position) => {
  const chai = await loadChai()
  chai.assert.equal(position.poolKey.tokenX, expectedPosition.poolKey.tokenX)
  chai.assert.equal(position.poolKey.tokenY, expectedPosition.poolKey.tokenY)
  chai.assert.equal(position.poolKey.feeTier.fee.v, expectedPosition.poolKey.feeTier.fee.v)
  chai.assert.equal(
    position.poolKey.feeTier.tickSpacing,
    expectedPosition.poolKey.feeTier.tickSpacing
  )
  chai.assert.equal(position.lowerTickIndex, expectedPosition.lowerTickIndex)
  chai.assert.equal(position.upperTickIndex, expectedPosition.upperTickIndex)
  chai.assert.equal(position.liquidity.v, expectedPosition.liquidity.v)
  chai.assert.equal(position.feeGrowthInsideX.v, expectedPosition.feeGrowthInsideX.v)
  chai.assert.equal(position.feeGrowthInsideY.v, expectedPosition.feeGrowthInsideY.v)
  chai.assert.equal(position.tokensOwedX.v, expectedPosition.tokensOwedX.v)
  chai.assert.equal(position.tokensOwedY.v, expectedPosition.tokensOwedY.v)
}

export const loadChai = async () => {
  return (await dynamicImport('chai', module)) as typeof import('chai')
}
