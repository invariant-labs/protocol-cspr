/* eslint-disable camelcase */
import { BigNumber } from '@ethersproject/bignumber'
import {
  CLByteArray,
  CLValueBuilder,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  Keys,
  RuntimeArgs,
  decodeBase16
} from 'casper-js-sdk'
import { DEFAULT_PAYMENT_AMOUNT, TESTNET_NODE_URL } from './consts'
import { Key, Network } from './enums'
import {
  bigintToByteArray,
  callWasm,
  decodeChunk,
  decodeFeeTiers,
  decodeInvariantConfig,
  decodePool,
  decodePoolKeys,
  decodePosition,
  decodePositionLength,
  decodeTick,
  encodePoolKey,
  getBitAtIndex,
  getDeploymentData,
  hash,
  integerSafeCast,
  loadWasm,
  sendTx
} from './utils'

const CONTRACT_NAME = 'invariant'

export class Invariant {
  client: CasperClient
  service: CasperServiceByJsonRPC
  contract: Contracts.Contract
  paymentAmount: bigint
  network: Network

  private constructor(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    network: Network,
    contractHash: string,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ) {
    this.client = client
    this.service = service
    this.network = network
    this.contract = new Contracts.Contract(this.client)
    this.contract.setContractHash(contractHash)
    this.paymentAmount = paymentAmount
  }

  static async deploy(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    network: Network,
    deployer: Keys.AsymmetricKey,
    fee: bigint = 0n,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ): Promise<[string, string]> {
    const contract = new Contracts.Contract(client)

    const wasm = await getDeploymentData(CONTRACT_NAME)

    const args = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string(CONTRACT_NAME),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string('init'),
      fee: CLValueBuilder.u128(BigNumber.from(fee))
    })

    const signedDeploy = contract.install(
      wasm,
      args,
      paymentAmount.toString(),
      deployer.publicKey,
      network.toString(),
      [deployer]
    )

    await service.deploy(signedDeploy)

    const deploymentResult = await service.waitForDeploy(signedDeploy, 100000)

    if (deploymentResult.execution_results[0].result.Failure) {
      // Shows the required deploy fee in case of failure
      console.log('----------')
      for (const v of deploymentResult.execution_results[0].result.Failure!.effect.transforms) {
        console.log(v)
      }
      console.log('----------')

      throw new Error(
        deploymentResult.execution_results[0].result.Failure.error_message?.toString()
      )
    }

    const stateRootHash = await service.getStateRootHash()
    const { Account } = await service.getBlockState(
      stateRootHash,
      deployer.publicKey.toAccountHashStr(),
      []
    )

    if (!Account) {
      throw new Error('Account not found in block state')
    }

    const contractPackageHash = Account.namedKeys.find((i: any) => i.name === CONTRACT_NAME)?.key

    if (!contractPackageHash) {
      throw new Error('Contract package not found in account named keys')
    }

    const { ContractPackage } = await service.getBlockState(stateRootHash, contractPackageHash, [])

    if (!ContractPackage) {
      throw new Error('Contract package not found in block state')
    }

    return [
      contractPackageHash.replace('hash-', ''),
      ContractPackage.versions[0].contractHash.replace('contract-', '')
    ]
  }

  static async load(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    network: Network,
    contractHash: string
  ) {
    return new Invariant(client, service, network, 'hash-' + contractHash)
  }

  async setContractHash(contractHash: string) {
    this.contract.setContractHash('hash-' + contractHash)
  }

  async addFeeTier(account: Keys.AsymmetricKey, fee: bigint, tickSpacing: bigint) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'add_fee_tier',
      {
        fee: CLValueBuilder.u128(BigNumber.from(fee)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(tickSpacing))
      }
    )
  }

  async removeFeeTier(account: Keys.AsymmetricKey, fee: bigint, tickSpacing: bigint) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'remove_fee_tier',
      {
        fee: CLValueBuilder.u128(BigNumber.from(fee)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(tickSpacing))
      }
    )
  }

  async createPool(
    account: Keys.AsymmetricKey,
    token0: string,
    token1: string,
    fee: bigint,
    tickSpacing: bigint,
    initSqrtPrice: bigint,
    initTick: bigint
  ) {
    const token0Key = new CLByteArray(decodeBase16(token0))
    const token1Key = new CLByteArray(decodeBase16(token1))

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'create_pool',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(fee)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(tickSpacing)),
        init_sqrt_price: CLValueBuilder.u128(BigNumber.from(initSqrtPrice)),
        init_tick: CLValueBuilder.i32(integerSafeCast(initTick))
      }
    )
  }

  async changeFeeReceiver(
    account: Keys.AsymmetricKey,
    token0: string,
    token1: string,
    fee: bigint,
    tickSpacing: bigint,
    newFeeReceiverHash: Key,
    newFeeReceiver: string
  ) {
    const token0Key = new CLByteArray(decodeBase16(token0))
    const token1Key = new CLByteArray(decodeBase16(token1))
    const newFeeReceiverBytes = new Uint8Array([
      newFeeReceiverHash,
      ...decodeBase16(newFeeReceiver)
    ])
    const newFeeReceiverKey = new CLByteArray(newFeeReceiverBytes)

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'change_fee_receiver',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(fee)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(tickSpacing)),
        fee_receiver: newFeeReceiverKey
      }
    )
  }

  async changeProtocolFee(account: Keys.AsymmetricKey, protocolFee: bigint) {
    const txArgs = RuntimeArgs.fromMap({
      protocol_fee: CLValueBuilder.u128(BigNumber.from(protocolFee))
    })

    const deploy = this.contract.callEntrypoint(
      'change_protocol_fee',
      txArgs,
      account.publicKey,
      this.network,
      DEFAULT_PAYMENT_AMOUNT.toString(),
      [account]
    )

    deploy.sign([account])
    await deploy.send(TESTNET_NODE_URL)
    await this.service.deploy(deploy)
    return await this.service.waitForDeploy(deploy, 100000)
  }

  async getInvariantConfig() {
    const key = hash('config')
    const stateRootHash = await this.service.getStateRootHash()

    const response = await this.service.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes

    return decodeInvariantConfig(rawBytes)
  }

  async getFeeTiers() {
    const key = hash('fee_tiers')
    const stateRootHash = await this.service.getStateRootHash()
    const response = await this.service.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes

    return decodeFeeTiers(rawBytes)
  }

  async feeTierExist(fee: bigint, tickSpacing: bigint): Promise<boolean> {
    const feeTiers = await this.getFeeTiers()
    return feeTiers.some(tier => tier.percentage === fee && tier.tickSpacing === tickSpacing)
  }

  async getPool(poolKey: any) {
    const buffor: number[] = []

    const poolKeyBytes = encodePoolKey(poolKey)
    buffor.push(...'pools'.split('').map(c => c.charCodeAt(0)))
    buffor.push('#'.charCodeAt(0))
    buffor.push(...'pools'.split('').map(c => c.charCodeAt(0)))
    buffor.push(...poolKeyBytes)

    const key = hash(new Uint8Array(buffor))

    const stateRootHash = await this.service.getStateRootHash()

    const response = await this.service.getDictionaryItemByName(
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
    account: Keys.AsymmetricKey,
    token0: string,
    token1: string,
    fee: bigint,
    tickSpacing: bigint,
    lowerTick: bigint,
    upperTick: bigint,
    liquidityDelta: bigint,
    slippageLimitLower: bigint,
    slippageLimitUpper: bigint
  ) {
    const token0Key = new CLByteArray(decodeBase16(token0))
    const token1Key = new CLByteArray(decodeBase16(token1))

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'create_position',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(fee)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(tickSpacing)),
        lower_tick: CLValueBuilder.i32(integerSafeCast(lowerTick)),
        upper_tick: CLValueBuilder.i32(integerSafeCast(upperTick)),
        liquidity_delta: CLValueBuilder.u256(BigNumber.from(liquidityDelta)),
        slippage_limit_lower: CLValueBuilder.u128(BigNumber.from(slippageLimitLower)),
        slippage_limit_upper: CLValueBuilder.u128(BigNumber.from(slippageLimitUpper))
      }
    )
  }

  async removePosition(account: Keys.AsymmetricKey, index: bigint) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'remove_position',
      {
        index: CLValueBuilder.u32(integerSafeCast(index))
      }
    )
  }

  async transferPosition(account: Keys.AsymmetricKey, index: bigint) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'transfer_position',
      {
        index: CLValueBuilder.u32(integerSafeCast(index))
      }
    )
  }

  async claimFee(account: Keys.AsymmetricKey, index: bigint) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'claim_fee',
      {
        index: CLValueBuilder.u32(integerSafeCast(index))
      }
    )
  }

  async getPosition(account: Keys.AsymmetricKey, index: bigint) {
    const stateRootHash = await this.service.getStateRootHash()
    const buffor: number[] = []
    const indexBytes = bigintToByteArray(index)
    buffor.push(...'positions'.split('').map(c => c.charCodeAt(0)))
    buffor.push('#'.charCodeAt(0))
    buffor.push(...'positions'.split('').map(c => c.charCodeAt(0)))
    // Value indicating that bytes are related to `AccountHash` not `ContractPackageHash`
    buffor.push(0)
    buffor.push(...account.accountHash())
    buffor.push(...indexBytes.concat(Array(4 - indexBytes.length).fill(0)))

    const key = hash(new Uint8Array(buffor))

    const response = await this.service.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodePosition(rawBytes)
  }

  async getTick(poolKey: any, index: bigint) {
    const stateRootHash = await this.service.getStateRootHash()
    const buffor: number[] = []
    const indexBytes = bigintToByteArray(index)
    const preparedIndexBytes = indexBytes.concat(Array(4 - indexBytes.length).fill(0))
    const poolKeyBytes = encodePoolKey(poolKey)

    buffor.push(...'ticks'.split('').map(c => c.charCodeAt(0)))
    buffor.push('#'.charCodeAt(0))
    buffor.push(...'ticks'.split('').map(c => c.charCodeAt(0)))
    buffor.push(...poolKeyBytes)
    buffor.push(...preparedIndexBytes)

    const key = hash(new Uint8Array(buffor))

    const response = await this.service.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodeTick(rawBytes)
  }

  private async getTickmapChunk(poolKey: any, chunkIndex: bigint) {
    const stateRootHash = await this.service.getStateRootHash()
    const indexBytes = bigintToByteArray(chunkIndex)
    const preparedIndexBytes = indexBytes.concat(Array(2 - indexBytes.length).fill(0))
    const poolKeyBytes = encodePoolKey(poolKey)
    const buffor: number[] = []

    buffor.push(...'tickmap'.split('').map(c => c.charCodeAt(0)))
    buffor.push('#'.charCodeAt(0))
    buffor.push(...'bitmap'.split('').map(c => c.charCodeAt(0)))
    buffor.push(...preparedIndexBytes)
    buffor.push(...poolKeyBytes)

    const key = hash(new Uint8Array(buffor))

    const response = await this.service.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodeChunk(rawBytes)
  }

  async getPositionsCount(account: Keys.AsymmetricKey) {
    const stateRootHash = await this.service.getStateRootHash()
    const buffor: number[] = []
    buffor.push(...'positions'.split('').map(c => c.charCodeAt(0)))
    buffor.push('#'.charCodeAt(0))
    buffor.push(...'positions_length'.split('').map(c => c.charCodeAt(0)))
    // Value indicating that bytes are related to `AccountHash` not `ContractPackageHash`
    buffor.push(0)
    buffor.push(...account.accountHash())

    const key = hash(new Uint8Array(buffor))

    const response = await this.service.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodePositionLength(rawBytes)
  }

  async getPositions(account: Keys.AsymmetricKey) {
    const positionsCount = await this.getPositionsCount(account)
    const positions = []
    for (let i = 0n; i < positionsCount; i++) {
      positions.push(await this.getPosition(account, i))
    }
    return positions
  }

  async swap(
    account: Keys.AsymmetricKey,
    token0: string,
    token1: string,
    fee: bigint,
    tickSpacing: bigint,
    xToY: boolean,
    amount: bigint,
    byAmountIn: boolean,
    sqrtPriceLimit: bigint
  ) {
    const token0Key = new CLByteArray(decodeBase16(token0))
    const token1Key = new CLByteArray(decodeBase16(token1))

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'swap',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(fee)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(tickSpacing)),
        x_to_y: CLValueBuilder.bool(xToY),
        amount: CLValueBuilder.u256(BigNumber.from(amount)),
        by_amount_in: CLValueBuilder.bool(byAmountIn),
        sqrt_price_limit: CLValueBuilder.u128(BigNumber.from(sqrtPriceLimit))
      }
    )
  }

  async withdrawProtocolFee(
    account: Keys.AsymmetricKey,
    token0: string,
    token1: string,
    fee: bigint,
    tickSpacing: bigint
  ) {
    const token0Key = new CLByteArray(decodeBase16(token0))
    const token1Key = new CLByteArray(decodeBase16(token1))

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      this.network,
      'withdraw_protocol_fee',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(BigNumber.from(fee)),
        tick_spacing: CLValueBuilder.u32(integerSafeCast(tickSpacing))
      }
    )
  }

  private async getPoolKeys() {
    const key = hash('pool_keys')
    const stateRootHash = await this.service.getStateRootHash()
    const response = await this.service.getDictionaryItemByName(
      stateRootHash,
      this.contract.contractHash!,
      'state',
      key,
      { rawData: true }
    )

    const rawBytes = (response.CLValue! as any).bytes
    return decodePoolKeys(rawBytes)
  }

  async isTickInitialized(poolKey: any, tickIndex: bigint) {
    const wasm = await loadWasm()
    const chunkIndex = await callWasm(wasm.tickToChunk, tickIndex, poolKey.feeTier.tickSpacing)
    const tickPosition = await callWasm(wasm.tickToPos, tickIndex, poolKey.feeTier.tickSpacing)
    const chunk = await this.getTickmapChunk(poolKey, chunkIndex)
    return getBitAtIndex(chunk, tickPosition)
  }

  async getPools() {
    const poolKeys = await this.getPoolKeys()
    const pools = await Promise.all(poolKeys.map(async poolKey => await this.getPool(poolKey)))
    return pools
  }
}
