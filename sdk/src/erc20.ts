/* eslint-disable camelcase */
import { Some } from '@casperlabs/ts-results'
import {
  CLByteArray,
  CLPublicKey,
  CLValueBuilder,
  CasperClient,
  CasperServiceByJsonRPC,
  Contracts,
  Keys,
  RuntimeArgs,
  decodeBase16
} from 'casper-js-sdk'
import { ALLOWANCES, BALANCES, DEFAULT_PAYMENT_AMOUNT } from './consts'
import { Network } from './network'
import { getDeploymentData, hash, hexToBytes, sendTx } from './utils'

const CONTRACT_NAME = 'erc20'

export class Erc20 {
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
    nameSuffix: string,
    initial_supply: bigint = 0n,
    name: string = '',
    symbol: string = '',
    decimals: bigint = 0n,
    paymentAmount: bigint = DEFAULT_PAYMENT_AMOUNT
  ): Promise<string> {
    const contract = new Contracts.Contract(client)

    const wasm = await getDeploymentData(CONTRACT_NAME)

    const args = RuntimeArgs.fromMap({
      odra_cfg_package_hash_key_name: CLValueBuilder.string(CONTRACT_NAME + nameSuffix),
      odra_cfg_allow_key_override: CLValueBuilder.bool(true),
      odra_cfg_is_upgradable: CLValueBuilder.bool(true),
      odra_cfg_constructor: CLValueBuilder.string('init'),
      initial_supply: CLValueBuilder.option(Some(CLValueBuilder.u256(Number(initial_supply)))),
      name: CLValueBuilder.string(name),
      symbol: CLValueBuilder.string(symbol),
      decimals: CLValueBuilder.u8(Number(decimals))
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

    const contractPackageHash = Account.namedKeys.find(
      (i: any) => i.name === CONTRACT_NAME + nameSuffix
    )?.key

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
    return new Erc20(client, service, 'hash-' + contractHash)
  }

  async setContractHash(contractHash: string) {
    this.contract.setContractHash('hash-' + contractHash)
  }

  async transfer(
    account: Keys.AsymmetricKey,
    network: Network,
    recipient: CLPublicKey,
    amount: bigint
  ) {
    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      network,
      'transfer',
      {
        recipient: CLValueBuilder.key(recipient),
        amount: CLValueBuilder.u256(Number(amount))
      }
    )
  }

  async name() {
    const response = await this.contract.queryContractDictionary('state', hash('name'))

    return response.data
  }

  async symbol() {
    const response = await this.contract.queryContractDictionary('state', hash('symbol'))

    return response.data
  }

  async decimals() {
    const response = await this.contract.queryContractDictionary('state', hash('decimals'))

    return BigInt(response.data)
  }

  async balance_of(address: CLPublicKey) {
    const accountHash = hexToBytes(address.toAccountHashStr().replace('account-hash-', ''))
    const balanceKey = new Uint8Array([...BALANCES, 0, ...accountHash])

    const response = await this.contract.queryContractDictionary('state', hash(balanceKey))

    return BigInt(response.data._hex)
  }

  async allowance(owner: string, spender: string) {
    const ownerHash = hexToBytes(owner)
    const spenderHash = hexToBytes(spender)
    const balanceKey = new Uint8Array([...ALLOWANCES, 0, ...ownerHash, 0, ...spenderHash])

    const response = await this.contract.queryContractDictionary('state', hash(balanceKey))

    return BigInt(response.data._hex)
  }

  async approve(account: Keys.AsymmetricKey, network: Network, spender: string, amount: bigint) {
    const spenderKey = new CLByteArray(decodeBase16(spender))

    return await sendTx(
      this.contract,
      this.service,
      this.paymentAmount,
      account,
      network,
      'approve',
      {
        spender: CLValueBuilder.key(spenderKey),
        amount: CLValueBuilder.u256(Number(amount))
      }
    )
  }
}
