/* eslint-disable camelcase */
import { BigNumber } from '@ethersproject/bignumber'
import {
  CLValueBuilder,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  Keys,
  RuntimeArgs
} from 'casper-js-sdk'
import { DEFAULT_PAYMENT_AMOUNT, TESTNET_NODE_URL } from './consts'
import { Network } from './network'
import { getDeploymentData, hash, sendTx } from './utils'

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

    console.log('Step 1')
    const signedDeploy = contract.install(
      wasm,
      args,
      paymentAmount.toString(),
      deployer.publicKey,
      network.toString(),
      [deployer]
    )

    console.log('Step 2')
    await service.deploy(signedDeploy)

    console.log('Step 3')
    const deploymentResult = await service.waitForDeploy(signedDeploy, 100000)

    console.log('Step 4')
    console.log(deploymentResult.execution_results[0])
    if (deploymentResult.execution_results[0].result.Failure) {
      {
        console.log(deploymentResult.execution_results[0].result.Failure.effect)
        for (const v of deploymentResult.execution_results[0].result.Failure.effect.transforms) {
          console.log(v)
        }
      }
      throw new Error(
        deploymentResult.execution_results[0].result.Failure.error_message?.toString()
      )
    }

    console.log('Step 5')
    const stateRootHash = await service.getStateRootHash()
    const { Account } = await service.getBlockState(
      stateRootHash,
      deployer.publicKey.toAccountHashStr(),
      []
    )

    console.log('Step 6')
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

  async getProtocolFee(account: Keys.AsymmetricKey, network: Network) {
    // const key = blake2b('config').toString()
    // const stateRootHash = await this.service.getStateRootHash()
    // const key = '7071474438b622de882472abc92f9d7e1fd3456e19b46f0117fe607b9d819679'
    const key = hash('tmp')
    const stateRootHash = await this.service.getStateRootHash()

    const response = await this.contract.queryContractDictionary(
      'state',
      key,
      stateRootHash,
      this.client
    )
    console.log(response)
    // return await sendTx(
    //   this.contract,
    //   this.service,
    //   this.paymentAmount,
    //   account,
    //   network,
    //   'get_protocol_fee',
    //   {}
    // )
  }

  async queryFields() {
    {
      const key = hash('tmp')
      const stateRootHash = await this.service.getStateRootHash()
      console.log(stateRootHash)
      // const response = await this.contract.queryContractDictionary(
      //   'state',
      //   key,
      //   stateRootHash,
      //   this.client
      // )
      // console.log(response)
    }
    {
      const key = hash('tmp_address')
      const stateRootHash = await this.service.getStateRootHash()
      console.log(stateRootHash)
      const response = await this.contract.queryContractDictionary(
        'state',
        key,
        stateRootHash,
        this.client
      )
      console.log(response)
    }
    // {
    //   const key = hash('tmp_percentage')
    //   const stateRootHash = await this.service.getStateRootHash()
    //   console.log(stateRootHash)
    //   const response = await this.contract.queryContractDictionary(
    //     'state',
    //     key,
    //     stateRootHash,
    //     this.client
    //   )
    //   console.log(response)
    // }
  }
}
