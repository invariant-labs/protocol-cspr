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
import { Network } from './network'
import {
  decodeFeeTiers,
  decodeInvariantConfig,
  decodePool,
  encodePoolKey,
  getDeploymentData,
  hash,
  sendTx
} from './utils'

const CONTRACT_NAME = 'invariant'

export class Invariant {
  client: CasperClient
  service: CasperServiceByJsonRPC
  contract: Contracts.Contract
  paymentAmount: bigint

  private constructor(
    client: CasperClient,
    service: CasperServiceByJsonRPC,
    contractHash: string,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ) {
    this.client = client
    this.service = service
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
  ): Promise<string> {
    const contract = new Contracts.Contract(client)

    const wasm = await getDeploymentData(CONTRACT_NAME)

    const args = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string(CONTRACT_NAME),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string('init'),
      fee: CLValueBuilder.u128(Number(fee))
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

    return ContractPackage.versions[0].contractHash.replace('contract-', '')
  }

  static async load(client: CasperClient, service: CasperServiceByJsonRPC, contractHash: string) {
    return new Invariant(client, service, 'hash-' + contractHash)
  }

  async setContractHash(contractHash: string) {
    this.contract.setContractHash('hash-' + contractHash)
  }

  async addFeeTier(
    account: Keys.AsymmetricKey,
    network: Network,
    fee: bigint,
    tickSpacing: bigint
  ) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      network,
      'add_fee_tier',
      {
        fee: CLValueBuilder.u128(Number(fee)),
        tick_spacing: CLValueBuilder.u32(Number(tickSpacing))
      }
    )
  }

  async removeFeeTier(
    account: Keys.AsymmetricKey,
    network: Network,
    fee: bigint,
    tickSpacing: bigint
  ) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      network,
      'remove_fee_tier',
      {
        fee: CLValueBuilder.u128(Number(fee)),
        tick_spacing: CLValueBuilder.u32(Number(tickSpacing))
      }
    )
  }

  async createPool(
    account: Keys.AsymmetricKey,
    network: Network,
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
      network,
      'create_pool',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(Number(fee)),
        tick_spacing: CLValueBuilder.u32(Number(tickSpacing)),
        init_sqrt_price: CLValueBuilder.u128(BigNumber.from(initSqrtPrice)),
        init_tick: CLValueBuilder.i32(Number(initTick))
      }
    )
  }

  async changeFeeReceiver(
    account: Keys.AsymmetricKey,
    network: Network,
    token0: string,
    token1: string,
    fee: bigint,
    tickSpacing: bigint,
    newFeeReceiver: string
  ) {
    const token0Key = new CLByteArray(decodeBase16(token0))
    const token1Key = new CLByteArray(decodeBase16(token1))
    const feeReceiverKey = new CLByteArray(decodeBase16(newFeeReceiver))

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      network,
      'change_fee_receiver',
      {
        token_0: CLValueBuilder.key(token0Key),
        token_1: CLValueBuilder.key(token1Key),
        fee: CLValueBuilder.u128(Number(fee)),
        tick_spacing: CLValueBuilder.u32(Number(tickSpacing)),
        fee_receiver: CLValueBuilder.key(feeReceiverKey)
      }
    )
  }

  async changeProtocolFee(account: Keys.AsymmetricKey, network: Network, protocolFee: bigint) {
    const txArgs = RuntimeArgs.fromMap({
      protocol_fee: CLValueBuilder.u128(BigNumber.from(protocolFee))
    })

    const deploy = this.contract.callEntrypoint(
      'change_protocol_fee',
      txArgs,
      account.publicKey,
      network,
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
}
