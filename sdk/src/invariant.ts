/* eslint-disable camelcase */
import { BigNumber } from '@ethersproject/bignumber'
import {
  CLByteArray,
  CLValueBuilder,
  CasperClient,
  Contracts,
  Keys,
  RuntimeArgs,
  decodeBase16
} from 'casper-js-sdk'
import type {
  FeeTier,
  Liquidity,
  Percentage,
  Pool,
  PoolKey,
  Position,
  SqrtPrice,
  Tick,
  TokenAmount
} from 'wasm'
import { DEFAULT_PAYMENT_AMOUNT, INVARIANT_CONTRACT_NAME } from './consts'
import {
  decodeChunk,
  decodeFeeTiers,
  decodeInvariantConfig,
  decodePool,
  decodePoolKeys,
  decodePosition,
  decodePositionLength,
  decodeTick
} from './decoder'
import { bigintToByteArray, encodePoolKey, encodeString, hash } from './parser'
import { Key, Network } from './schema'
import {
  callWasm,
  extractContractHash,
  extractContractPackageHash,
  findContractPackageHash,
  getBitAtIndex,
  getDeploymentData,
  integerSafeCast,
  loadWasm,
  sendTx
} from './utils'

export class Invariant {
  client: CasperClient
  contract: Contracts.Contract
  network: Network
  paymentAmount: bigint
  wasm: any

  private constructor(
    client: CasperClient,
    contractHash: string,
    network: Network,
    wasm: any,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ) {
    this.client = client
    this.contract = new Contracts.Contract(this.client)
    this.contract.setContractHash(contractHash)
    this.network = network
    this.paymentAmount = paymentAmount
    this.wasm = wasm
  }

  static async deploy(
    client: CasperClient,
    network: Network,
    deployer: Keys.AsymmetricKey,
    fee: Percentage = { v: 0n },
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ): Promise<[string, string]> {
    const contract = new Contracts.Contract(client)

    const wasm = await getDeploymentData(INVARIANT_CONTRACT_NAME)

    const args = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string(INVARIANT_CONTRACT_NAME),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string('init'),
      fee: CLValueBuilder.u128(BigNumber.from(fee.v))
    })

    const signedDeploy = contract.install(
      wasm,
      args,
      paymentAmount.toString(),
      deployer.publicKey,
      network.toString(),
      [deployer]
    )

    await client.nodeClient.deploy(signedDeploy)

    const deploymentResult = await client.nodeClient.waitForDeploy(signedDeploy, 100000)

    if (deploymentResult.execution_results[0].result.Failure) {
      throw new Error(
        deploymentResult.execution_results[0].result.Failure.error_message?.toString()
      )
    }

    const stateRootHash = await client.nodeClient.getStateRootHash()
    const { Account } = await client.nodeClient.getBlockState(
      stateRootHash,
      deployer.publicKey.toAccountHashStr(),
      []
    )

    if (!Account) {
      throw new Error('Account not found in block state')
    }

    const contractPackageHash = findContractPackageHash(Account, INVARIANT_CONTRACT_NAME)

    if (!contractPackageHash) {
      throw new Error('Contract package not found in account named keys')
    }

    const { ContractPackage } = await client.nodeClient.getBlockState(
      stateRootHash,
      contractPackageHash,
      []
    )

    if (!ContractPackage) {
      throw new Error('Contract package not found in block state')
    }

    return [extractContractHash(contractPackageHash), extractContractPackageHash(ContractPackage)]
  }

  static async load(client: CasperClient, contractHash: string, network: Network) {
    const wasm = await loadWasm()
    return new Invariant(client, 'hash-' + contractHash, network, wasm)
  }

  async setContractHash(contractHash: string) {
    this.contract.setContractHash('hash-' + contractHash)
  }

  async addFeeTier(signer: Keys.AsymmetricKey, feeTier: FeeTier) {
    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'add_fee_tier',
      {
        fee: CLValueBuilder.u128(BigNumber.from(feeTier.fee.v)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(feeTier.tickSpacing))
      }
    )
  }

  async removeFeeTier(signer: Keys.AsymmetricKey, feeTier: FeeTier) {
    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'remove_fee_tier',
      {
        fee: CLValueBuilder.u128(BigNumber.from(feeTier.fee.v)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(feeTier.tickSpacing))
      }
    )
  }

  async createPool(signer: Keys.AsymmetricKey, poolKey: PoolKey, initSqrtPrice: SqrtPrice) {
    const token0Key = new CLByteArray(decodeBase16(poolKey.tokenX))
    const token1Key = new CLByteArray(decodeBase16(poolKey.tokenY))
    const initTick = await callWasm(
      this.wasm.calculateTick,
      initSqrtPrice,
      poolKey.feeTier.tickSpacing
    )

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'create_pool',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(poolKey.feeTier.fee.v)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(poolKey.feeTier.tickSpacing)),
        init_sqrt_price: CLValueBuilder.u128(BigNumber.from(initSqrtPrice.v)),
        init_tick: CLValueBuilder.i32(integerSafeCast(initTick))
      }
    )
  }

  async changeFeeReceiver(
    signer: Keys.AsymmetricKey,
    poolKey: PoolKey,
    newFeeReceiverHash: Key,
    newFeeReceiver: string
  ) {
    const token0Key = new CLByteArray(decodeBase16(poolKey.tokenX))
    const token1Key = new CLByteArray(decodeBase16(poolKey.tokenY))
    const newFeeReceiverBytes = new Uint8Array([
      newFeeReceiverHash,
      ...decodeBase16(newFeeReceiver)
    ])
    const newFeeReceiverKey = new CLByteArray(newFeeReceiverBytes)

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'change_fee_receiver',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(poolKey.feeTier.fee.v)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(poolKey.feeTier.tickSpacing)),
        fee_receiver: newFeeReceiverKey
      }
    )
  }

  async changeProtocolFee(signer: Keys.AsymmetricKey, protocolFee: Percentage) {
    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'change_protocol_fee',
      {
        protocol_fee: CLValueBuilder.u128(BigNumber.from(protocolFee.v))
      }
    )
  }

  async getInvariantConfig() {
    const key = hash('config')
    const stateRootHash = await this.client.nodeClient.getStateRootHash()

    const response = await this.client.nodeClient.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes

    return decodeInvariantConfig(rawBytes)
  }

  async getFeeTiers(): Promise<FeeTier[]> {
    const key = hash('fee_tiers')
    const stateRootHash = await this.client.nodeClient.getStateRootHash()
    const response = await this.client.nodeClient.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes

    return decodeFeeTiers(rawBytes)
  }

  async feeTierExist(feeTier: FeeTier): Promise<boolean> {
    const feeTiers = await this.getFeeTiers()
    return feeTiers.some(
      tier => tier.fee.v === feeTier.fee.v && tier.tickSpacing === feeTier.tickSpacing
    )
  }

  async getPool(poolKey: PoolKey): Promise<Pool> {
    const buffor: number[] = []

    const poolKeyBytes = encodePoolKey(poolKey)
    buffor.push(...encodeString('pools'))
    buffor.push(...encodeString('#'))
    buffor.push(...encodeString('pools'))
    buffor.push(...poolKeyBytes)

    const key = hash(new Uint8Array(buffor))

    const stateRootHash = await this.client.nodeClient.getStateRootHash()

    const response = await this.client.nodeClient.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes

    return decodePool(rawBytes)
  }

  async createPosition(
    signer: Keys.AsymmetricKey,
    poolKey: PoolKey,
    lowerTick: bigint,
    upperTick: bigint,
    liquidityDelta: Liquidity,
    slippageLimitLower: SqrtPrice,
    slippageLimitUpper: SqrtPrice
  ) {
    const token0Key = new CLByteArray(decodeBase16(poolKey.tokenX))
    const token1Key = new CLByteArray(decodeBase16(poolKey.tokenY))

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'create_position',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(poolKey.feeTier.fee.v)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(poolKey.feeTier.tickSpacing)),
        lower_tick: CLValueBuilder.i32(integerSafeCast(lowerTick)),
        upper_tick: CLValueBuilder.i32(integerSafeCast(upperTick)),
        liquidity_delta: CLValueBuilder.u256(BigNumber.from(liquidityDelta.v)),
        slippage_limit_lower: CLValueBuilder.u128(BigNumber.from(slippageLimitLower.v)),
        slippage_limit_upper: CLValueBuilder.u128(BigNumber.from(slippageLimitUpper.v))
      }
    )
  }

  async removePosition(signer: Keys.AsymmetricKey, index: bigint) {
    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'remove_position',
      {
        index: CLValueBuilder.u32(integerSafeCast(index))
      }
    )
  }

  async transferPosition(
    signer: Keys.AsymmetricKey,
    index: bigint,
    receiverHash: Key,
    receiver: string
  ) {
    const receiverBytes = new Uint8Array([receiverHash, ...decodeBase16(receiver)])
    const receiverKey = new CLByteArray(receiverBytes)

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'transfer_position',
      {
        index: CLValueBuilder.u32(integerSafeCast(index)),
        receiver: receiverKey
      }
    )
  }

  async claimFee(signer: Keys.AsymmetricKey, index: bigint) {
    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'claim_fee',
      {
        index: CLValueBuilder.u32(integerSafeCast(index))
      }
    )
  }

  async getPosition(signer: Keys.AsymmetricKey, index: bigint): Promise<Position> {
    const stateRootHash = await this.client.nodeClient.getStateRootHash()
    const buffor: number[] = []
    const indexBytes = bigintToByteArray(index)
    buffor.push(...encodeString('positions'))
    buffor.push(...encodeString('#'))
    buffor.push(...encodeString('positions'))
    buffor.push(...[Key.Account])
    buffor.push(...signer.accountHash())
    buffor.push(...indexBytes.concat(Array(4 - indexBytes.length).fill(0)))

    const key = hash(new Uint8Array(buffor))

    const response = await this.client.nodeClient.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodePosition(rawBytes)
  }

  async getTick(poolKey: PoolKey, index: bigint): Promise<Tick> {
    const stateRootHash = await this.client.nodeClient.getStateRootHash()
    const buffor: number[] = []

    const indexBytes = bigintToByteArray(index)
    const filler = index < 0n ? 255 : 0
    const preparedIndexBytes = indexBytes.concat(Array(4 - indexBytes.length).fill(filler))

    const poolKeyBytes = encodePoolKey(poolKey)

    buffor.push(...encodeString('ticks'))
    buffor.push(...encodeString('#'))
    buffor.push(...encodeString('ticks'))
    buffor.push(...poolKeyBytes)
    buffor.push(...preparedIndexBytes)

    const key = hash(new Uint8Array(buffor))

    const response = await this.client.nodeClient.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodeTick(rawBytes)
  }

  private async getTickmapChunk(poolKey: PoolKey, chunkIndex: bigint): Promise<bigint> {
    const stateRootHash = await this.client.nodeClient.getStateRootHash()
    const indexBytes = bigintToByteArray(chunkIndex)
    const preparedIndexBytes = indexBytes.concat(Array(2 - indexBytes.length).fill(0))
    const poolKeyBytes = encodePoolKey(poolKey)
    const buffor: number[] = []

    buffor.push(...encodeString('tickmap'))
    buffor.push(...encodeString('#'))
    buffor.push(...encodeString('bitmap'))
    buffor.push(...preparedIndexBytes)
    buffor.push(...poolKeyBytes)

    const key = hash(new Uint8Array(buffor))

    const response = await this.client.nodeClient.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodeChunk(rawBytes)
  }

  async getPositionsCount(signer: Keys.AsymmetricKey): Promise<bigint> {
    const stateRootHash = await this.client.nodeClient.getStateRootHash()
    const buffor: number[] = []
    buffor.push(...encodeString('positions'))
    buffor.push(...encodeString('#'))
    buffor.push(...encodeString('positions_length'))
    buffor.push(...[Key.Account])
    buffor.push(...signer.accountHash())

    const key = hash(new Uint8Array(buffor))

    const response = await this.client.nodeClient.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodePositionLength(rawBytes)
  }

  async getPositions(account: Keys.AsymmetricKey): Promise<Position[]> {
    const positionsCount = await this.getPositionsCount(account)
    const positions = await Promise.all(
      Array.from(
        { length: integerSafeCast(positionsCount) },
        async (_, i) => await this.getPosition(account, BigInt(i))
      )
    )
    return positions
  }

  async swap(
    signer: Keys.AsymmetricKey,
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice
  ) {
    const token0Key = new CLByteArray(decodeBase16(poolKey.tokenX))
    const token1Key = new CLByteArray(decodeBase16(poolKey.tokenY))

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'swap',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(poolKey.feeTier.fee.v)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(poolKey.feeTier.tickSpacing)),
        x_to_y: CLValueBuilder.bool(xToY),
        amount: CLValueBuilder.u256(BigNumber.from(amount.v)),
        by_amount_in: CLValueBuilder.bool(byAmountIn),
        sqrt_price_limit: CLValueBuilder.u128(BigNumber.from(sqrtPriceLimit.v))
      }
    )
  }

  async withdrawProtocolFee(signer: Keys.AsymmetricKey, poolKey: PoolKey) {
    const token0Key = new CLByteArray(decodeBase16(poolKey.tokenX))
    const token1Key = new CLByteArray(decodeBase16(poolKey.tokenY))

    return await sendTx(
      this.contract,
      this.client.nodeClient,
      this.paymentAmount,
      signer,
      this.network,
      'withdraw_protocol_fee',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(poolKey.feeTier.fee.v)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(poolKey.feeTier.tickSpacing))
      }
    )
  }

  private async getPoolKeys(): Promise<PoolKey[]> {
    const key = hash('pool_keys')
    const stateRootHash = await this.client.nodeClient.getStateRootHash()
    const response = await this.client.nodeClient.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodePoolKeys(rawBytes)
  }

  async isTickInitialized(poolKey: PoolKey, tickIndex: bigint): Promise<boolean> {
    const chunkIndex = await callWasm(this.wasm.tickToChunk, tickIndex, poolKey.feeTier.tickSpacing)
    const tickPosition = await callWasm(this.wasm.tickToPos, tickIndex, poolKey.feeTier.tickSpacing)
    const chunk = await this.getTickmapChunk(poolKey, chunkIndex)
    return getBitAtIndex(chunk, tickPosition)
  }

  async getPools(): Promise<Pool[]> {
    const poolKeys = await this.getPoolKeys()
    const pools = await Promise.all(poolKeys.map(async poolKey => await this.getPool(poolKey)))
    return pools
  }
}
